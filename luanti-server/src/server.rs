//! Minimal Server implementation serving as prototype
#![expect(
    clippy::todo,
    reason = "//TODO remove before completion of the prototype"
)]
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;

use log::debug;
use log::error;
use log::info;
use log::trace;
use log::warn;
use luanti_protocol::CommandDirection;
use luanti_protocol::CommandRef;
use luanti_protocol::LuantiConnection;
use luanti_protocol::LuantiServer;
use luanti_protocol::commands::client_to_server::ClientReadySpec;
use luanti_protocol::commands::client_to_server::Init2Spec;
use luanti_protocol::commands::client_to_server::InitSpec;
use luanti_protocol::commands::client_to_server::PlayerPosCommand;
use luanti_protocol::commands::client_to_server::SrpBytesASpec;
use luanti_protocol::commands::client_to_server::SrpBytesMSpec;
use luanti_protocol::commands::client_to_server::ToServerCommand;
use luanti_protocol::commands::client_to_server::UpdateClientInfoSpec;
use luanti_protocol::commands::server_to_client::AnnounceMediaSpec;
use luanti_protocol::commands::server_to_client::AuthAcceptSpec;
use luanti_protocol::commands::server_to_client::HelloSpec;
use luanti_protocol::commands::server_to_client::ItemdefCommand;
use luanti_protocol::commands::server_to_client::ItemdefList;
use luanti_protocol::commands::server_to_client::NodedefSpec;
use luanti_protocol::commands::server_to_client::SrpBytesSBSpec;
use luanti_protocol::commands::server_to_client::ToClientCommand;
use luanti_protocol::peer::PeerError;
use luanti_protocol::types::AuthMechsBitset;
use luanti_protocol::types::NodeDefManager;
use luanti_protocol::types::PlayerPos;
use luanti_protocol::types::v3f;
use luanti_protocol::wire::packet::LATEST_PROTOCOL_VERSION;
use luanti_protocol::wire::packet::SER_FMT_VER_HIGHEST_WRITE;
use rand::RngCore;
use sha2::Sha256;
use srp::groups::G_2048;
use srp::server::SrpServer;
use srp::server::SrpServerVerifier;
use std::net::SocketAddr;
use std::ops::RangeInclusive;

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

                    // TODO(kawogi) do actual stuff, like sending response, altering state …
                    // self.maybe_show(&client_command);
                    // self.conn.send(client_command)?;
                },
            }
        }
    }

    pub(crate) fn handle_client_command(&mut self, command: ToServerCommand) -> Result<()> {
        match command {
            ToServerCommand::Init(init_spec) => self.handle_init(*init_spec),
            ToServerCommand::Init2(init2_spec) => self.handle_init2(*init2_spec),
            ToServerCommand::ModchannelJoin(_modchannel_join_spec) => todo!(),
            ToServerCommand::ModchannelLeave(_modchannel_leave_spec) => todo!(),
            ToServerCommand::TSModchannelMsg(_tsmodchannel_msg_spec) => todo!(),
            ToServerCommand::Playerpos(player_pos_command) => {
                self.handle_player_pos(*player_pos_command)
            }
            ToServerCommand::Gotblocks(_gotblocks_spec) => todo!(),
            ToServerCommand::Deletedblocks(_deletedblocks_spec) => todo!(),
            ToServerCommand::InventoryAction(_inventory_action_spec) => todo!(),
            ToServerCommand::TSChatMessage(_tschat_message_spec) => todo!(),
            ToServerCommand::Damage(_damage_spec) => todo!(),
            ToServerCommand::Playeritem(_playeritem_spec) => todo!(),
            ToServerCommand::Respawn(_respawn_spec) => todo!(),
            ToServerCommand::Interact(_interact_spec) => todo!(),
            ToServerCommand::RemovedSounds(_removed_sounds_spec) => todo!(),
            ToServerCommand::NodemetaFields(_nodemeta_fields_spec) => todo!(),
            ToServerCommand::InventoryFields(_inventory_fields_spec) => todo!(),
            ToServerCommand::RequestMedia(_request_media_spec) => todo!(),
            ToServerCommand::HaveMedia(_have_media_spec) => todo!(),
            ToServerCommand::ClientReady(client_ready_spec) => {
                self.handle_client_ready(*client_ready_spec)
            }
            ToServerCommand::FirstSrp(_first_srp_spec) => todo!(),
            ToServerCommand::SrpBytesA(srp_bytes_aspec) => {
                self.handle_srp_bytes_a(*srp_bytes_aspec)
            }
            ToServerCommand::SrpBytesM(srp_bytes_mspec) => {
                self.handle_srp_bytes_mspec(*srp_bytes_mspec)
            }
            ToServerCommand::UpdateClientInfo(update_client_info_spec) => {
                self.handle_update_client_info(*update_client_info_spec)
            }
        }
    }

    /// This is the first command a client sends after a connect
    fn handle_init(&mut self, init_spec: InitSpec) -> Result<()> {
        if !matches!(self.state, ClientConnectionState::None) {
            warn!("ignoring unexpected Init-command");
            return Ok(());
        }

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

        self.state = ClientConnectionState::Initialized { player_name };

        self.conn.send(ToClientCommand::Hello(Box::new(HelloSpec {
            serialization_ver: serialization_version,
            compression_mode: 0, // unused field
            proto_ver: protocol_version,
            // TODO(kawogi) align those flags with the registered auth-providers (or transparently proxy them into a secure authentication or just don't implement all of them)
            auth_mechs: AuthMechsBitset::default(),
            username_legacy: String::new(), // always empty
        })))?;

        Ok(())
    }

    fn handle_srp_bytes_a(&mut self, srp_bytes_aspec: SrpBytesASpec) -> Result<()> {
        let ClientConnectionState::Initialized { player_name } = &self.state else {
            warn!("ignoring unexpected SrcBytesASpec-command");
            return Ok(());
        };

        // the client sends `A` earlier than usual because the required `g` is well-known
        // and doesn't need to be sent by the server
        let SrpBytesASpec { bytes_a, based_on } = srp_bytes_aspec;

        if based_on == 0 {
            // TODO(kawogi) respond with a proper auth rejection to the client
            // TODO(kawogi) maybe remove this specific test in favor of a general incompatibility if modern clients (based on their protocol version) prove to be unable to ever send this value
            bail!("server doesn't support legacy authentication");
        } else if based_on > 1 {
            // TODO(kawogi) respond with a proper auth rejection to the client
            bail!("server doesn't support `based_on={based_on}` for SRP authentication");
        }

        // authentication ignores casing
        // TODO(kawogi) this will be used by the final implementation
        // let auth_name = player_name.to_ascii_lowercase();

        let srp_server = SrpServer::<Sha256>::new(&G_2048);
        // let (username, a_pub) = get_client_request();

        // let (salt, v) = get_user(&username);
        // TODO(kawogi) look up salt and verifier for this user
        let mut srp_user_salt = [0_u8; 64];
        rand::rng().fill_bytes(&mut srp_user_salt);
        let mut srp_user_verifier = [0_u8; 64];
        rand::rng().fill_bytes(&mut srp_user_verifier);

        let mut srp_private_b = [0_u8; 256];
        rand::rng().fill_bytes(&mut srp_private_b);
        let srp_b_pub = srp_server.compute_public_ephemeral(&srp_private_b, &srp_user_verifier);

        let verifier = srp_server
            .process_reply(&srp_private_b, &srp_user_verifier, &bytes_a)
            .map_err(|error| anyhow!("{error}"))?;

        self.state = ClientConnectionState::Initialized2 {
            player_name: player_name.clone(),
            verifier,
        };

        self.conn
            .send(ToClientCommand::SrpBytesSB(Box::new(SrpBytesSBSpec {
                s: srp_user_salt.to_vec(),
                b: srp_b_pub,
            })))?;

        Ok(())
    }

    fn handle_srp_bytes_mspec(&mut self, srp_bytes_mspec: SrpBytesMSpec) -> Result<()> {
        let ClientConnectionState::Initialized2 {
            player_name,
            verifier,
        } = &self.state
        else {
            warn!("ignoring unexpected SrcBytesASpec-command");
            return Ok(());
        };

        let SrpBytesMSpec { bytes_m } = srp_bytes_mspec;

        match verifier.verify_client(&bytes_m) {
            Ok(()) => {
                info!("player '{player_name}' was sucessfully authenticated");
            }
            Err(error) => {
                warn!("failed to authenticate player '{player_name}': {error}");
                // FIXME(kawogi) remove this once there's a proper implementation in place
                warn!("MOCK-AUTHENTICATOR GRANTS ACCESS NONETHELESS!");
            }
        }

        self.state = ClientConnectionState::Authenticated;

        self.conn
            .send(ToClientCommand::AuthAccept(Box::new(AuthAcceptSpec {
                player_pos: v3f {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                // TODO(kawogi) load from actual map?
                map_seed: 0,
                // TODO(kawogi) what is this value?
                recommended_send_interval: 0.05,
                // TODO(kawogi) what is this value?
                sudo_auth_methods: 2,
            })))?;

        Ok(())
    }

    fn handle_init2(&self, init2_spec: Init2Spec) -> Result<()> {
        let ClientConnectionState::Authenticated = &self.state else {
            warn!("ignoring unexpected Init2Spec-command");
            return Ok(());
        };

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

        self.conn
            .send(ToClientCommand::Itemdef(Box::new(ItemdefCommand {
                item_def: itemdef_list,
            })))?;

        self.conn
            .send(ToClientCommand::Nodedef(Box::new(NodedefSpec {
                node_def: node_def_manager,
            })))?;

        self.conn.send(ToClientCommand::AnnounceMedia(Box::new(
            AnnounceMediaSpec {
                files: vec![],
                remote_servers: String::new(),
            },
        )))?;

        Ok(())
    }

    #[expect(
        clippy::unnecessary_wraps,
        reason = "//TODO(kawogi) for symmetry with other handlers, but should be reviewed"
    )]
    fn handle_client_ready(
        &self,
        client_ready_spec: ClientReadySpec,
    ) -> std::result::Result<(), anyhow::Error> {
        let ClientConnectionState::Authenticated = &self.state else {
            warn!("ignoring unexpected ClientReadySpec-command");
            return Ok(());
        };

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

        Ok(())
    }

    #[expect(
        clippy::unnecessary_wraps,
        clippy::needless_pass_by_value,
        reason = "//TODO(kawogi) for symmetry with other handlers, but should be reviewed"
    )]
    fn handle_player_pos(
        &self,
        player_pos_command: PlayerPosCommand,
    ) -> std::result::Result<(), anyhow::Error> {
        let ClientConnectionState::Authenticated = &self.state else {
            warn!("ignoring unexpected PlayerPosCommand-command");
            return Ok(());
        };

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
    fn handle_update_client_info(
        &self,
        update_client_info_spec: UpdateClientInfoSpec,
    ) -> Result<()> {
        let ClientConnectionState::Authenticated = &self.state else {
            warn!("ignoring unexpected UpdateClientInfo-command");
            return Ok(());
        };

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
    Initialized {
        /// name of the player
        player_name: String,
    },
    Initialized2 {
        /// name of the player
        player_name: String,
        verifier: SrpServerVerifier<Sha256>,
    },
    Authenticated,
}
