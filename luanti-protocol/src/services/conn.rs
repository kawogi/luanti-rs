//! `LuantiConnection`
//!
//!
//!
use std::net::SocketAddr;

use crate::peer::Peer;
use crate::wire::command::AccessDeniedSpec;
use crate::wire::command::Command;
use crate::wire::command::ToClientCommand;
use crate::wire::command::ToServerCommand;
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

    pub fn send_access_denied(&self, code: AccessDeniedCode) -> Result<()> {
        self.send(AccessDeniedSpec { code }.into())
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
