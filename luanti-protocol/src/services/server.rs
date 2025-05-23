//!
//! For now, the `LuantiServer` is just a wrapper around a `LuantiSocket`,
//! and a `LuantiConnection` is just a wrapper around a `SocketPeer`.
//!
//! In the future it may provide its own abstraction above the Luanti Commands.

use log::error;
use log::info;
use log::warn;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::unbounded_channel;

use super::conn::LuantiConnection;
use super::socket::LuantiSocket;

pub struct LuantiServer {
    accept_rx: UnboundedReceiver<LuantiConnection>,
}

impl LuantiServer {
    #[must_use]
    pub fn new(server_address: SocketAddr) -> Self {
        let (accept_tx, accept_rx) = unbounded_channel();
        let runner = LuantiServerRunner {
            server_address,
            accept_tx,
        };
        tokio::spawn(async move {
            runner.run().await;
        });
        Self { accept_rx }
    }

    pub async fn accept(&mut self) -> LuantiConnection {
        self.accept_rx.recv().await.unwrap()
    }
}

struct LuantiServerRunner {
    server_address: SocketAddr,
    accept_tx: UnboundedSender<LuantiConnection>,
}

impl LuantiServerRunner {
    async fn run(self) {
        info!("LuantiServer listening on {}", self.server_address);
        let mut socket = loop {
            match LuantiSocket::new(self.server_address, true).await {
                Ok(socket) => break socket,
                Err(err) => {
                    warn!("LuantiServer: bind failed: {err}");
                    info!("Retrying in 5 seconds");
                    tokio::time::sleep(Duration::from_millis(5000)).await;
                }
            }
        };
        info!("LuantiServer started");
        #[expect(
            clippy::infinite_loop,
            reason = "// TODO implement a cancellation mechanism"
        )]
        loop {
            let peer = socket.accept().await.unwrap();
            info!("LuantiServer accepted connection");
            let conn = LuantiConnection::new(peer);
            if let Err(error) = self.accept_tx.send(conn) {
                error!("Unexpected send fail in LuantiServer: {error}");
            }
        }
    }
}
