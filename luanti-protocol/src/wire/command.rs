#![expect(
    clippy::min_ident_chars,
    reason = "//TODO rename the remaining fields within the macro"
)]

#[macro_use]
mod macros;
pub mod client_to_server;
pub mod server_to_client;

use super::channel_id::ChannelId;
use super::deser::Deserialize;
use super::deser::DeserializeResult;
use super::deser::Deserializer;
use super::ser::Serialize;
use super::ser::SerializeResult;
use super::ser::Serializer;
#[allow(clippy::wildcard_imports, reason = "greatly simplifies macros")]
use super::types::*;
use client_to_server::ToServerCommand;
use server_to_client::ToClientCommand;

#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    ToServer(ToServerCommand),
    ToClient(ToClientCommand),
}

pub trait CommandProperties {
    fn direction(&self) -> CommandDirection;
    fn default_channel(&self) -> ChannelId;
    fn default_reliability(&self) -> bool;
    fn command_name(&self) -> &'static str;
}

/// This only exists to make `audit_command` generic, but it
/// wasn't as clean as I hoped.
/// TODO(paradust): Factor this out.
pub trait CommandRef: CommandProperties + std::fmt::Debug {
    fn toserver_ref(&self) -> Option<&ToServerCommand>;
    fn toclient_ref(&self) -> Option<&ToClientCommand>;
}

pub fn serialize_commandref<Cmd: CommandRef, S: Serializer>(
    cmd: &Cmd,
    ser: &mut S,
) -> SerializeResult {
    if let Some(command) = cmd.toserver_ref() {
        ToServerCommand::serialize(command, ser)?;
    }
    if let Some(command) = cmd.toclient_ref() {
        ToClientCommand::serialize(command, ser)?;
    }
    Ok(())
}

impl CommandProperties for Command {
    fn direction(&self) -> CommandDirection {
        match self {
            Command::ToServer(_) => CommandDirection::ToServer,
            Command::ToClient(_) => CommandDirection::ToClient,
        }
    }

    fn default_channel(&self) -> ChannelId {
        match self {
            Command::ToServer(command) => command.default_channel(),
            Command::ToClient(command) => command.default_channel(),
        }
    }

    fn default_reliability(&self) -> bool {
        match self {
            Command::ToServer(command) => command.default_reliability(),
            Command::ToClient(command) => command.default_reliability(),
        }
    }

    fn command_name(&self) -> &'static str {
        match self {
            Command::ToServer(command) => command.command_name(),
            Command::ToClient(command) => command.command_name(),
        }
    }
}

impl CommandRef for Command {
    fn toserver_ref(&self) -> Option<&ToServerCommand> {
        match self {
            Command::ToServer(command) => Some(command),
            Command::ToClient(_) => None,
        }
    }

    fn toclient_ref(&self) -> Option<&ToClientCommand> {
        match self {
            Command::ToServer(_) => None,
            Command::ToClient(command) => Some(command),
        }
    }
}

impl CommandRef for ToClientCommand {
    fn toserver_ref(&self) -> Option<&ToServerCommand> {
        None
    }

    fn toclient_ref(&self) -> Option<&ToClientCommand> {
        Some(self)
    }
}

impl CommandRef for ToServerCommand {
    fn toserver_ref(&self) -> Option<&ToServerCommand> {
        Some(self)
    }

    fn toclient_ref(&self) -> Option<&ToClientCommand> {
        None
    }
}

impl Serialize for Command {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        match value {
            Command::ToServer(command) => ToServerCommand::serialize(command, ser),
            Command::ToClient(command) => ToClientCommand::serialize(command, ser),
        }
    }
}

impl Deserialize for Command {
    type Output = Option<Self>;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        Ok(match deser.direction() {
            CommandDirection::ToClient => ToClientCommand::deserialize(deser)?.map(Self::ToClient),
            CommandDirection::ToServer => ToServerCommand::deserialize(deser)?.map(Self::ToServer),
        })
    }
}
