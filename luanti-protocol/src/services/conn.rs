//! `LuantiConnection`
//!
//!
//!
use std::net::SocketAddr;

use crate::commands::Command;
use crate::commands::client_to_server::ToServerCommand;
use crate::commands::server_to_client::AccessDeniedSpec;
use crate::commands::server_to_client::ToClientCommand;
use crate::peer::Peer;
use crate::wire::types::AccessDeniedCode;
use anyhow::Result;
use anyhow::bail;

/// This is owned by the driver
pub struct LuantiConnection {
    peer: Peer,
}

impl LuantiConnection {
    #[must_use]
    pub fn new(peer: Peer) -> Self {
        Self { peer }
    }

    #[must_use]
    pub fn remote_addr(&self) -> SocketAddr {
        self.peer.remote_addr()
    }

    /// Send a command to the client
    pub fn send(&self, command: ToClientCommand) -> Result<()> {
        self.peer.send(Command::ToClient(command))
    }

    pub fn send_access_denied(
        &self,
        code: AccessDeniedCode,
        reason: String,
        reconnect: bool,
    ) -> Result<()> {
        self.send(
            AccessDeniedSpec {
                code,
                reason,
                reconnect,
            }
            .into(),
        )
    }

    /// Await a command from the peer
    /// Returns (channel, reliable flag, Command)
    /// Returns None when the peer is disconnected
    pub async fn recv(&mut self) -> Result<ToServerCommand> {
        match self.peer.recv().await? {
            Command::ToServer(command) => Ok(command),
            Command::ToClient(_) => {
                bail!("Received wrong direction command from SocketPeer")
            }
        }
    }
}

/// This is owned by the `luanti_protocol`
pub struct LuantiConnectionRecord;
