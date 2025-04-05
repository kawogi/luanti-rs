//! Minimal Server implementation serving as prototype

use crate::authentication::Authenticator;
use crate::client_connection::ClientConnection;
use log::info;
use luanti_protocol::LuantiServer;
use std::net::SocketAddr;
use tokio::task::JoinHandle;

/// A server providing access to a single Luanti world
pub(crate) struct LuantiWorldServer {
    /// used to accept connection from clients
    bind_addr: SocketAddr,
    verbosity: u8,
    runner: Option<JoinHandle<()>>,
}

impl LuantiWorldServer {
    pub(crate) fn new(bind_addr: SocketAddr, verbosity: u8) -> Self {
        Self {
            bind_addr,
            verbosity,
            runner: None,
        }
    }

    pub(crate) fn start(&mut self, authenticator: impl Authenticator + 'static) {
        assert!(self.runner.is_none(), "server is already running");

        let bind_addr = self.bind_addr;
        let verbosity = self.verbosity;
        let runner = tokio::spawn(async move {
            Self::accept_connections(bind_addr, authenticator, verbosity).await;
        });
        self.runner.replace(runner);
    }

    async fn accept_connections<Auth: Authenticator + 'static>(
        bind_addr: SocketAddr,
        authenticator: Auth,
        verbosity: u8,
    ) {
        let mut server = LuantiServer::new(bind_addr);
        let mut connection_id = 1;
        loop {
            tokio::select! {
                connection = server.accept() => {
                    let id = connection_id;
                    connection_id += 1;
                    info!("[P{}] New client connected from {:?}", id, connection.remote_addr());
                    ClientConnection::spawn(id, connection, authenticator.clone(), verbosity);
                },
            }
        }
    }
}
