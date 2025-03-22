use std::fmt::{self, Display};

use anyhow::bail;

use crate::wire::deser::DeserializeError;

use super::{
    deser::{Deserialize, DeserializeResult, Deserializer},
    ser::{Serialize, Serializer},
};

/**
 * Channels used for Client -> Server communication
 *
 * - 2: Notifications back to the server (e.g. GOTBLOCKS)
 * - 1: Init and Authentication
 * - 0: everything else
 *
 * Packet order is only guaranteed inside a channel, so packets that operate on
 * the same objects are *required* to be in the same channel.
 */
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ChannelId {
    #[default]
    Default = Self::DEFAULT,
    Init = Self::INIT,
    Response = Self::RESPONSE,
}

impl ChannelId {
    const DEFAULT: u8 = 0;
    const INIT: u8 = 1;
    const RESPONSE: u8 = 2;

    #[must_use]
    pub fn all() -> [Self; 3] {
        [Self::Default, Self::Init, Self::Response]
    }
}

impl Display for ChannelId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChannelId::Default => formatter.write_str("default"),
            ChannelId::Init => formatter.write_str("init"),
            ChannelId::Response => formatter.write_str("response"),
        }
    }
}

impl Deserialize for ChannelId {
    type Output = Self;

    fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        match u8::deserialize(deserializer)? {
            Self::DEFAULT => Ok(Self::Default),
            Self::INIT => Ok(Self::Init),
            Self::RESPONSE => Ok(Self::Response),
            invalid => bail!(DeserializeError::InvalidChannel(invalid)),
        }
    }
}

impl Serialize for ChannelId {
    type Input = Self;

    fn serialize<S: Serializer>(
        value: &Self::Input,
        serializer: &mut S,
    ) -> super::ser::SerializeResult {
        u8::serialize(&(*value as u8), serializer)
    }
}

impl From<ChannelId> for usize {
    fn from(value: ChannelId) -> Self {
        (value as u8).into()
    }
}
