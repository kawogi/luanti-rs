//! `LuantiConnection`
//!
//!
//!
use std::net::SocketAddr;

use crate::peer::peer::Peer;
use crate::wire::command::*;
use crate::wire::types::*;
use anyhow::Result;
use anyhow::bail;

/// This is owned by the driver
pub struct LuantiConnection {
    peer: Peer,
}

impl LuantiConnection {
    #[must_use]
    pub fn new(peer: Peer) -> Self {
        Self { peer: peer }
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
