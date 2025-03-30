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
use log::warn;
use luanti_protocol::CommandDirection;
use luanti_protocol::CommandRef;
use luanti_protocol::LuantiConnection;
use luanti_protocol::LuantiServer;
use luanti_protocol::commands::client_to_server::InitSpec;
use luanti_protocol::commands::client_to_server::SrpBytesASpec;
use luanti_protocol::commands::client_to_server::ToServerCommand;
use luanti_protocol::commands::server_to_client::HelloSpec;
use luanti_protocol::commands::server_to_client::ToClientCommand;
use luanti_protocol::peer::PeerError;
use luanti_protocol::types::AuthMechsBitset;
use luanti_protocol::wire::packet::LATEST_PROTOCOL_VERSION;
use luanti_protocol::wire::packet::SER_FMT_VER_HIGHEST_WRITE;
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
            ToServerCommand::Init2(_init2_spec) => todo!(),
            ToServerCommand::ModchannelJoin(_modchannel_join_spec) => todo!(),
            ToServerCommand::ModchannelLeave(_modchannel_leave_spec) => todo!(),
            ToServerCommand::TSModchannelMsg(_tsmodchannel_msg_spec) => todo!(),
            ToServerCommand::Playerpos(_player_pos_command) => todo!(),
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
            ToServerCommand::ClientReady(_client_ready_spec) => todo!(),
            ToServerCommand::FirstSrp(_first_srp_spec) => todo!(),
            ToServerCommand::SrpBytesA(srp_bytes_aspec) => {
                self.handle_srp_bytes_a(*srp_bytes_aspec)
            }
            ToServerCommand::SrpBytesM(_srp_bytes_mspec) => todo!(),
            ToServerCommand::UpdateClientInfo(_update_client_info_spec) => todo!(),
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

    fn handle_srp_bytes_a(&self, _srp_bytes_aspec: SrpBytesASpec) -> Result<()> {
        todo!()
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
            2.. => trace!("{} {:#?}", prefix, command),
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
}
