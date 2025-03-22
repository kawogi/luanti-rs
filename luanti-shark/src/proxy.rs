//!
//! Luanti Proxy Server
//!
//! This heavily tests the code for serialization, deserialization,
//! packet splitting, and reliable retransmission.
//!
//! This is not just a simple packet forwarding proxy. Instead, it performs
//! split packet reconstruction and reliable tracking itself. Incoming
//! packets are deserialized to a stream of Commands (a strongly typed
//! representation of luanti data).
//!
//! To forward the Command to the other side, serialization, packet
//! splitting, and reliable tracking are performed in the opposite direction.
//!
//! If everything is correct, the proxied connection should be stable, and
//! durable to packet loss.
//!
//! As an added bonus, enabling verbose mode will print out the stream of
//! commands in both directions, in a human-readable format.
use anyhow::Result;

use log::debug;
use log::error;
use log::info;
use log::trace;
use luanti_protocol::CommandDirection;
use luanti_protocol::CommandRef;
use luanti_protocol::LuantiClient;
use luanti_protocol::LuantiConnection;
use luanti_protocol::LuantiServer;
use luanti_protocol::peer::PeerError;
use luanti_protocol::wire::command::server_to_client::ToClientCommand;
use std::net::SocketAddr;

pub(crate) struct LuantiProxy;

impl LuantiProxy {
    pub(crate) fn new(bind_addr: SocketAddr, forwarding_addr: SocketAddr, verbosity: u8) -> Self {
        let runner = LuantiProxyRunner {
            bind_addr,
            forwarding_addr,
            verbosity,
        };
        tokio::spawn(async move { runner.run().await });
        LuantiProxy {}
    }
}

struct LuantiProxyRunner {
    /// used to accept connection from clients
    bind_addr: SocketAddr,
    /// used to connect to the server
    forwarding_addr: SocketAddr,
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
                    debug!("forwarding connection to {addr}", addr = self.forwarding_addr);
                    // TODO(kawogi) this outgoing connection attempt blocks accepting new incoming connections
                    let client = LuantiClient::connect(self.forwarding_addr).await.expect("Connect failed");
                    debug!("successfully connected to {addr}", addr = self.forwarding_addr);
                    ProxyAdapterRunner::spawn(id, conn, client, self.verbosity);
                },
            }
        }
    }
}

pub(crate) struct ProxyAdapterRunner {
    id: u64,
    conn: LuantiConnection,
    client: LuantiClient,
    verbosity: u8,
}

impl ProxyAdapterRunner {
    pub(crate) fn spawn(id: u64, conn: LuantiConnection, client: LuantiClient, verbosity: u8) {
        let runner = ProxyAdapterRunner {
            id,
            conn,
            client,
            verbosity,
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
            tokio::select! {
                command = self.conn.recv() => {
                    trace!("conn.recv: {command:?}");
                    let command = command?;
                    self.maybe_show(&command);
                    self.client.send(command)?;
                },
                command = self.client.recv() => {
                    trace!("client.recv: {command:?}");
                    let command = command?;
                    self.maybe_show(&command);
                    self.conn.send(command)?;
                }
            }
        }
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
