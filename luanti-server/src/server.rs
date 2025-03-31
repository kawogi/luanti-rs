//! Minimal Server implementation serving as prototype
#![expect(
    clippy::todo,
    reason = "//TODO remove before completion of the prototype"
)]
use anyhow::Result;
use anyhow::bail;

use log::debug;
use log::error;
use log::info;
use log::trace;
use luanti_protocol::CommandDirection;
use luanti_protocol::CommandRef;
use luanti_protocol::LuantiConnection;
use luanti_protocol::LuantiServer;
use luanti_protocol::commands::CommandProperties;
use luanti_protocol::commands::client_to_server::ClientReadySpec;
use luanti_protocol::commands::client_to_server::Init2Spec;
use luanti_protocol::commands::client_to_server::InitSpec;
use luanti_protocol::commands::client_to_server::PlayerPosCommand;
use luanti_protocol::commands::client_to_server::ToServerCommand;
use luanti_protocol::commands::client_to_server::UpdateClientInfoSpec;
use luanti_protocol::commands::server_to_client::AnnounceMediaSpec;
use luanti_protocol::commands::server_to_client::HelloSpec;
use luanti_protocol::commands::server_to_client::ItemdefCommand;
use luanti_protocol::commands::server_to_client::ItemdefList;
use luanti_protocol::commands::server_to_client::NodedefSpec;
use luanti_protocol::commands::server_to_client::ToClientCommand;
use luanti_protocol::peer::PeerError;
use luanti_protocol::types::AuthMechsBitset;
use luanti_protocol::types::NodeDefManager;
use luanti_protocol::types::PlayerPos;
use luanti_protocol::wire::packet::LATEST_PROTOCOL_VERSION;
use luanti_protocol::wire::packet::SER_FMT_VER_HIGHEST_WRITE;
use std::mem;
use std::net::SocketAddr;
use std::ops::RangeInclusive;

use crate::auth::SrpAuthState;
use crate::auth::SrpUserAuthData;

const SUPPORTED_PROTOCOL_VERSIONS: RangeInclusive<u16> =
    LATEST_PROTOCOL_VERSION..=LATEST_PROTOCOL_VERSION;
const SUPPORTED_SERIALIZATION_VERSIONS: RangeInclusive<u8> =
    SER_FMT_VER_HIGHEST_WRITE..=SER_FMT_VER_HIGHEST_WRITE;

/// A server providing access to a single Luanti world
pub(crate) struct LuantiWorldServer;

impl LuantiWorldServer {
    pub(crate) fn new(bind_addr: SocketAddr, verbosity: u8) -> Self {
        let runner = LuantiProxyRunner {
            bind_addr,
            verbosity,
        };
        tokio::spawn(async move { runner.run().await });
        LuantiWorldServer {}
    }
}

struct LuantiProxyRunner {
    /// used to accept connection from clients
    bind_addr: SocketAddr,
    verbosity: u8,
}

impl LuantiProxyRunner {
    async fn run(self) {
        let mut server = LuantiServer::new(self.bind_addr);
        let mut next_id: u64 = 1;
        loop {
            tokio::select! {
                conn = server.accept() => {
                    let id = next_id;
                    next_id += 1;
                    info!("[P{}] New client connected from {:?}", id, conn.remote_addr());
                    ProxyAdapterRunner::spawn(id, conn, self.verbosity);
                },
            }
        }
    }
}

pub(crate) struct ProxyAdapterRunner {
    id: u64,
    conn: LuantiConnection,
    verbosity: u8,
    // /// the accepted serialization version
    // serialization_version: u8,
    // /// the accepted protocol version
    // protocol_version: u16,
    // /// the accepted compression modes
    // supp_compr_modes: u16,
    state: ClientConnectionState,
}

impl ProxyAdapterRunner {
    pub(crate) fn spawn(id: u64, conn: LuantiConnection, verbosity: u8) {
        let runner = ProxyAdapterRunner {
            id,
            conn,
            verbosity,
            // serialization_version: 0,
            // protocol_version: 0,
            // supp_compr_modes: 0,
            state: ClientConnectionState::default(),
        };
        tokio::spawn(async move { runner.run().await });
    }

    pub(crate) async fn run(mut self) {
        debug!("starting proxy runner");
        match self.run_inner().await {
            Ok(()) => (),
            Err(err) => {
                let show_err = if let Some(err) = err.downcast_ref::<PeerError>() {
                    !matches!(err, PeerError::PeerSentDisconnect)
                } else {
                    true
                };
                if show_err {
                    error!("[{}] Disconnected: {:?}", self.id, err);
                } else {
                    info!("[{}] Disconnected", self.id);
                }
            }
        }
    }

    pub(crate) async fn run_inner(&mut self) -> Result<()> {
        loop {
            // TODO(kawogi) review whether this select remains useful after completion of the server's base implementation
            tokio::select! {
                command = self.conn.recv() => {
                    trace!("conn.recv: {command:?}");
                    let command = command?;
                    self.maybe_show(&command);
                    self.handle_client_command(command)?;
                },
            }
        }
    }

    pub(crate) fn handle_client_command(&mut self, command: ToServerCommand) -> Result<()> {
        self.state = match mem::take(&mut self.state) {
            ClientConnectionState::None => match command {
                ToServerCommand::Init(init_spec) => self.handle_init(*init_spec)?,
                unexpected => {
                    bail!(
                        "unexpected command for init state: {}",
                        unexpected.command_name()
                    );
                }
            },
            ClientConnectionState::Authenticating(srp_auth_state) => {
                match srp_auth_state.handle_client_message(command, &self.conn)? {
                    SrpAuthState::Authenticated { display_name, name } => {
                        ClientConnectionState::Authenticated { display_name, name }
                    }
                    authenticating => ClientConnectionState::Authenticating(authenticating),
                }
            }
            ClientConnectionState::Authenticated { display_name, name } => match command {
                ToServerCommand::Init2(init2_spec) => {
                    Self::handle_init2(display_name, name, *init2_spec, &self.conn)?
                }
                unexpected => {
                    bail!(
                        "unexpected command for authenticated state: {}",
                        unexpected.command_name()
                    );
                }
            },
            ClientConnectionState::Loading { display_name, name } => match command {
                ToServerCommand::ClientReady(client_ready_spec) => {
                    Self::handle_client_ready(display_name, name, *client_ready_spec)?
                }
                unexpected => {
                    bail!(
                        "unexpected command for authenticated state: {}",
                        unexpected.command_name()
                    );
                }
            },
            ClientConnectionState::Running { display_name, name } => {
                match command {
                    ToServerCommand::Playerpos(player_pos_command) => {
                        Self::handle_player_pos(*player_pos_command)?;
                    }
                    ToServerCommand::UpdateClientInfo(update_client_info_spec) => {
                        Self::handle_update_client_info(*update_client_info_spec)?;
                    }
                    ToServerCommand::ModchannelJoin(_modchannel_join_spec) => todo!(),
                    ToServerCommand::ModchannelLeave(_modchannel_leave_spec) => todo!(),
                    ToServerCommand::TSModchannelMsg(_tsmodchannel_msg_spec) => todo!(),
                    ToServerCommand::Gotblocks(_gotblocks_spec) => todo!(),
                    ToServerCommand::Deletedblocks(_deletedblocks_spec) => {
                        todo!()
                    }
                    ToServerCommand::InventoryAction(_inventory_action_spec) => todo!(),
                    ToServerCommand::TSChatMessage(_tschat_message_spec) => {
                        todo!()
                    }
                    ToServerCommand::Damage(_damage_spec) => todo!(),
                    ToServerCommand::Playeritem(_playeritem_spec) => todo!(),
                    ToServerCommand::Respawn(_respawn_spec) => todo!(),
                    ToServerCommand::Interact(_interact_spec) => todo!(),
                    ToServerCommand::RemovedSounds(_removed_sounds_spec) => {
                        todo!()
                    }
                    ToServerCommand::NodemetaFields(_nodemeta_fields_spec) => todo!(),
                    ToServerCommand::InventoryFields(_inventory_fields_spec) => todo!(),
                    ToServerCommand::RequestMedia(_request_media_spec) => {
                        todo!()
                    }
                    ToServerCommand::HaveMedia(_have_media_spec) => todo!(),
                    ToServerCommand::FirstSrp(_first_srp_spec) => todo!(),
                    unexpected => {
                        bail!(
                            "unexpected command for authenticated state: {}",
                            unexpected.command_name()
                        );
                    }
                }
                ClientConnectionState::Running { display_name, name }
            }
        };

        Ok(())
    }

    /// This is the first command a client sends after a connect
    fn handle_init(&mut self, init_spec: InitSpec) -> Result<ClientConnectionState> {
        let InitSpec {
            serialization_ver_max,
            supp_compr_modes: _unused,
            min_net_proto_version,
            max_net_proto_version,
            player_name,
        } = init_spec;

        info!("New player tries to connect: {player_name}");
        debug!("Client max serialization version: {serialization_ver_max}");
        debug!("Client protocol versions: {min_net_proto_version}..{max_net_proto_version}");

        let protocol_version = {
            // intersect version ranges
            let min_version = (*SUPPORTED_PROTOCOL_VERSIONS.start()).max(min_net_proto_version);
            let max_version = (*SUPPORTED_PROTOCOL_VERSIONS.end()).min(max_net_proto_version);
            if min_version > max_version {
                bail!(
                    "unsupported protocol version. Only {min}..{max} is supported, but {min_net_proto_version}..{max_net_proto_version} was requested",
                    min = SUPPORTED_PROTOCOL_VERSIONS.start(),
                    max = SUPPORTED_PROTOCOL_VERSIONS.end(),
                );
            }
            max_version
        };
        debug!("negotiated protocol version {protocol_version}");

        let serialization_version = {
            // intersect version ranges
            let min_version = *SUPPORTED_SERIALIZATION_VERSIONS.start();
            let max_version = (*SUPPORTED_SERIALIZATION_VERSIONS.end()).min(serialization_ver_max);
            if min_version > max_version {
                bail!(
                    "unsupported serialization version. Only {min}..{max} is supported, but 0..{max_net_proto_version} was requested",
                    min = SUPPORTED_PROTOCOL_VERSIONS.start(),
                    max = SUPPORTED_PROTOCOL_VERSIONS.end(),
                );
            }
            max_version
        };
        debug!("negotiated serialization_version version {serialization_version}");

        // TODO(kawogi) Verify that the technical protocol switch is transparently performed by the underlying connection implementation

        // TODO(kawogi) verify player name to be valid and registered
        let user_data = SrpUserAuthData::new_fake(player_name);

        self.conn.send(HelloSpec {
            serialization_ver: serialization_version,
            compression_mode: 0, // unused field
            proto_ver: protocol_version,
            // TODO(kawogi) align those flags with the registered auth-providers (or transparently proxy them into a secure authentication or just don't implement all of them)
            auth_mechs: AuthMechsBitset::default(),
            username_legacy: String::new(), // always empty
        })?;

        Ok(ClientConnectionState::Authenticating(SrpAuthState::new(
            user_data,
        )))
    }

    fn handle_init2(
        display_name: String,
        name: String,
        init2_spec: Init2Spec,
        conn: &LuantiConnection,
    ) -> Result<ClientConnectionState> {
        let Init2Spec { lang } = init2_spec;
        info!(
            "Client language: '{lang}'",
            lang = lang.unwrap_or("<none>".into())
        );

        let itemdef_list = ItemdefList {
            itemdef_manager_version: 0,
            defs: vec![],
            aliases: vec![],
        };

        let node_def_manager = NodeDefManager {
            content_features: vec![],
        };

        conn.send(ItemdefCommand {
            item_def: itemdef_list,
        })?;

        conn.send(NodedefSpec {
            node_def: node_def_manager,
        })?;

        conn.send(AnnounceMediaSpec {
            files: vec![],
            remote_servers: String::new(),
        })?;

        Ok(ClientConnectionState::Loading { display_name, name })
    }

    #[expect(
        clippy::unnecessary_wraps,
        reason = "//TODO(kawogi) for symmetry with other handlers, but should be reviewed"
    )]
    fn handle_client_ready(
        display_name: String,
        name: String,
        client_ready_spec: ClientReadySpec,
    ) -> Result<ClientConnectionState> {
        let ClientReadySpec {
            major_ver: _,
            minor_ver: _,
            patch_ver: _,
            reserved: _,
            full_ver,
            formspec_ver,
        } = client_ready_spec;

        info!(
            "Client ready: v{full_ver}, formspec v{}",
            formspec_ver
                .as_ref()
                .map_or("<none>".into(), ToString::to_string)
        );

        Ok(ClientConnectionState::Running { display_name, name })
    }

    #[expect(
        clippy::unnecessary_wraps,
        clippy::needless_pass_by_value,
        reason = "//TODO(kawogi) for symmetry with other handlers, but should be reviewed"
    )]
    fn handle_player_pos(
        player_pos_command: PlayerPosCommand,
    ) -> std::result::Result<(), anyhow::Error> {
        let PlayerPosCommand {
            player_pos:
                PlayerPos {
                    position,
                    speed,
                    pitch,
                    yaw,
                    keys_pressed,
                    fov,
                    wanted_range,

                    camera_inverted,
                    movement_speed,
                    movement_direction,
                },
        } = player_pos_command;

        debug!(
            "player moved: pos:({px},{py},{pz}) speed:({sx},{sy},{sz}) pitch:{pitch} yaw:{yaw} keys:{keys_pressed} fov:{fov} range:{wanted_range} cam_inv:{camera_inverted} mov_speed:{movement_speed} mov_dir:{movement_direction} ",
            px = position.x,
            py = position.y,
            pz = position.z,
            sx = speed.x,
            sy = speed.y,
            sz = speed.z,
        );

        Ok(())
    }

    #[expect(
        clippy::unnecessary_wraps,
        reason = "//TODO(kawogi) for symmetry with other handlers, but should be reviewed"
    )]
    fn handle_update_client_info(update_client_info_spec: UpdateClientInfoSpec) -> Result<()> {
        let UpdateClientInfoSpec {
            render_target_size,
            real_gui_scaling,
            real_hud_scaling,
            max_fs_size,
            touch_controls,
        } = update_client_info_spec;

        debug!(
            "updated client info: render size:({render_x},{render_y}) gui scaling:({real_gui_scaling}) hud scaling:{real_hud_scaling} fs size:({fs_x},{fs_y}) touch:{touch_controls}",
            render_x = render_target_size.x,
            render_y = render_target_size.y,
            fs_x = max_fs_size.x,
            fs_y = max_fs_size.y,
        );

        Ok(())
    }

    pub(crate) fn is_bulk_command<Cmd: CommandRef>(command: &Cmd) -> bool {
        matches!(
            command.toclient_ref(),
            Some(ToClientCommand::Blockdata(_) | ToClientCommand::Media(_))
        )
    }

    pub(crate) fn maybe_show<Cmd: CommandRef>(&self, command: &Cmd) {
        let dir = match command.direction() {
            CommandDirection::ToClient => "S->C",
            CommandDirection::ToServer => "C->S",
        };
        let prefix = format!("[{}] {} ", self.id, dir);
        let mut verbosity = self.verbosity;
        if verbosity == 2 && Self::is_bulk_command(command) {
            // Show the contents of smaller commands, but skip the huge ones
            verbosity = 1;
        }
        match verbosity {
            0 => (),
            1 => trace!("{} {}", prefix, command.command_name()),
            2.. => trace!("{prefix} {command:#?}"),
        }
    }
}

#[derive(Default)]
enum ClientConnectionState {
    /// The initial state after establishing the connection before any kind of communication happened.
    /// We're waiting for an Init-command to arrive
    #[default]
    None,
    Authenticating(SrpAuthState),
    Authenticated {
        /// The (non-technical) name that has been provided by the user. This might contain special
        /// characters or have mixed casing. This may be used as display name.
        display_name: String,
        /// Technical name (key) of the user's record. This is usually a normalized version of the
        /// `display_name`
        name: String,
    },
    Loading {
        /// The (non-technical) name that has been provided by the user. This might contain special
        /// characters or have mixed casing. This may be used as display name.
        display_name: String,
        /// Technical name (key) of the user's record. This is usually a normalized version of the
        /// `display_name`
        name: String,
    },
    Running {
        /// The (non-technical) name that has been provided by the user. This might contain special
        /// characters or have mixed casing. This may be used as display name.
        display_name: String,
        /// Technical name (key) of the user's record. This is usually a normalized version of the
        /// `display_name`
        name: String,
    },
}
