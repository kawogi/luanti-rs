use std::net::SocketAddr;

use anyhow::bail;

use super::socket::LuantiSocket;
use crate::peer::Peer;

#[allow(
    clippy::wildcard_imports,
    reason = "commands are expected to be used in bulk"
)]
use crate::wire::command::*;

pub struct LuantiClient {
    server: Peer,
}

impl LuantiClient {
    pub async fn connect(server_address: SocketAddr) -> anyhow::Result<Self> {
        let bind_addr = if server_address.is_ipv4() {
            "0.0.0.0:0".parse()?
        } else {
            "[::]:0".parse()?
        };
        let mut socket = LuantiSocket::new(bind_addr, false).await?;

        // Send a null packet to server.
        // It should answer back, establishing a peer ids.
        let server = socket.add_server(server_address).await;

        Ok(Self { server })
    }

    /// If this fails, the client has disconnected.
    pub async fn recv(&mut self) -> anyhow::Result<ToClientCommand> {
        match self.server.recv().await? {
            Command::ToClient(cmd) => Ok(cmd),
            Command::ToServer(_) => bail!("Invalid packet direction"),
        }
    }

    /// If this fails, the client has disconnected.
    pub fn send(&mut self, command: ToServerCommand) -> anyhow::Result<()> {
        self.server.send(Command::ToServer(command))
    }
}
