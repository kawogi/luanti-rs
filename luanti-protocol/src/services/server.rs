//!
//! For now, the LuantiServer is just a wrapper around a LuantiSocket,
//! and a LuantiConnection is just a wrapper around a SocketPeer.
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
    pub fn new(bind_addr: SocketAddr) -> Self {
        let (accept_tx, accept_rx) = unbounded_channel();
        let runner = LuantiServerRunner {
            bind_addr: bind_addr,
            accept_tx: accept_tx,
        };
        tokio::spawn(async move {
            runner.run().await;
        });
        Self {
            accept_rx: accept_rx,
        }
    }

    pub async fn accept(&mut self) -> LuantiConnection {
        self.accept_rx.recv().await.unwrap()
    }
}

struct LuantiServerRunner {
    bind_addr: SocketAddr,
    accept_tx: UnboundedSender<LuantiConnection>,
}

impl LuantiServerRunner {
    async fn run(self) {
        info!("LuantiServer starting on {}", self.bind_addr.to_string());
        let mut socket = loop {
            match LuantiSocket::new(self.bind_addr, true).await {
                Ok(socket) => break socket,
                Err(err) => {
                    warn!("LuantiServer: bind failed: {}", err);
                    info!("Retrying in 5 seconds");
                    tokio::time::sleep(Duration::from_millis(5000)).await;
                }
            };
        };
        info!("LuantiServer started");
        loop {
            let t = socket.accept().await.unwrap();
            info!("LuantiServer accepted connection");
            let conn = LuantiConnection::new(t);
            match self.accept_tx.send(conn) {
                Ok(_) => (),
                Err(_) => error!("Unexpected send fail in LuantiServer"),
            }
        }
    }
}
