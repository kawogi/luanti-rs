use crate::wire::{
    deser::{Deserialize, DeserializeResult, Deserializer},
    ser::{Serialize, SerializeResult, Serializer},
};
use luanti_protocol_derive::{LuantiDeserialize, LuantiSerialize};

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct AccessDeniedCommand {
    pub code: AccessDeniedCode,
    pub reason: String,
    pub reconnect: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AccessDeniedCode {
    WrongPassword,
    UnexpectedData,
    Singleplayer,
    WrongVersion,
    WrongCharsInName,
    WrongName,
    TooManyUsers,
    EmptyPassword,
    AlreadyConnected,
    ServerFail,
    CustomString(String),
    Shutdown(String, bool), // custom message (or blank), should_reconnect
    Crash(String, bool),    // custom message (or blank), should_reconnect
}

impl Serialize for AccessDeniedCode {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        #![allow(clippy::enum_glob_use, reason = "improves readability")]
        use AccessDeniedCode::*;
        match value {
            WrongPassword => u8::serialize(&0, ser),
            UnexpectedData => u8::serialize(&1, ser),
            Singleplayer => u8::serialize(&2, ser),
            WrongVersion => u8::serialize(&3, ser),
            WrongCharsInName => u8::serialize(&4, ser),
            WrongName => u8::serialize(&5, ser),
            TooManyUsers => u8::serialize(&6, ser),
            EmptyPassword => u8::serialize(&7, ser),
            AlreadyConnected => u8::serialize(&8, ser),
            ServerFail => u8::serialize(&9, ser),
            CustomString(msg) => {
                u8::serialize(&10, ser)?;
                String::serialize(msg, ser)?;
                Ok(())
            }
            Shutdown(msg, reconnect) => {
                u8::serialize(&11, ser)?;
                String::serialize(msg, ser)?;
                bool::serialize(reconnect, ser)?;
                Ok(())
            }
            Crash(msg, reconnect) => {
                u8::serialize(&12, ser)?;
                String::serialize(msg, ser)?;
                bool::serialize(reconnect, ser)?;
                Ok(())
            }
        }
    }
}

impl Deserialize for AccessDeniedCode {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        #![allow(clippy::enum_glob_use, reason = "improves readability")]
        use AccessDeniedCode::*;
        let deny_code = u8::deserialize(deser)?;
        match deny_code {
            0 => Ok(WrongPassword),
            1 => Ok(UnexpectedData),
            2 => Ok(Singleplayer),
            3 => Ok(WrongVersion),
            4 => Ok(WrongCharsInName),
            5 => Ok(WrongName),
            6 => Ok(TooManyUsers),
            7 => Ok(EmptyPassword),
            8 => Ok(AlreadyConnected),
            9 => Ok(ServerFail),
            10 => Ok(CustomString(String::deserialize(deser)?)),
            11 => Ok(Shutdown(
                String::deserialize(deser)?,
                (u8::deserialize(deser)? & 1) != 0,
            )),
            12 => Ok(Crash(
                String::deserialize(deser)?,
                (u8::deserialize(deser)? & 1) != 0,
            )),
            _ => Ok(CustomString(String::deserialize(deser)?)),
        }
    }
}

impl AccessDeniedCode {
    #[must_use]
    pub fn to_str(&self) -> &str {
        #![allow(clippy::enum_glob_use, reason = "improves readability")]
        use AccessDeniedCode::*;
        match self {
            WrongPassword => "Invalid password",
            UnexpectedData => {
                "Your client sent something the server didn't expect.  Try reconnecting or updating your client."
            }
            Singleplayer => {
                "The server is running in simple singleplayer mode.  You cannot connect."
            }
            WrongVersion => {
                "Your client's version is not supported.\nPlease contact the server administrator."
            }
            WrongCharsInName => "Player name contains disallowed characters",
            WrongName => "Player name not allowed",
            TooManyUsers => "Too many users",
            EmptyPassword => "Empty passwords are disallowed.  Set a password and try again.",
            AlreadyConnected => {
                "Another client is connected with this name.  If your client closed unexpectedly, try again in a minute."
            }
            ServerFail => "Internal server error",
            CustomString(msg) => {
                if msg.is_empty() {
                    "unknown"
                } else {
                    msg
                }
            }
            Shutdown(msg, _) => {
                if msg.is_empty() {
                    "Server shutting down"
                } else {
                    msg
                }
            }
            Crash(msg, _) => {
                if msg.is_empty() {
                    "The server has experienced an internal error.  You will now be disconnected."
                } else {
                    msg
                }
            }
        }
    }
}
