use std::fmt::{self, Display};

use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::wire::{
    deser::{Deserialize, DeserializeResult, Deserializer},
    ser::{Serialize, SerializeResult, Serializer},
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
#[repr(transparent)]
pub struct PeerId(u16);

impl PeerId {
    pub(crate) const NONE: Self = Self(0x0000);
    pub(crate) const SERVER: Self = Self(0x0001);

    pub(crate) fn is_none(self) -> bool {
        self == Self::NONE
    }

    pub(crate) fn is_server(self) -> bool {
        self == Self::SERVER
    }

    pub(crate) fn random() -> Self {
        Self(StdRng::from_os_rng().random_range(2..0xFFFF))
    }
}

impl Display for PeerId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl Deserialize for PeerId {
    type Output = Self;

    fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        u16::deserialize(deserializer).map(Self)
    }
}

impl Serialize for PeerId {
    type Input = Self;

    fn serialize<S: Serializer>(value: &Self::Input, serializer: &mut S) -> SerializeResult {
        u16::serialize(&value.0, serializer)
    }
}

// impl From<PeerId> for usize {
//     fn from(value: PeerId) -> Self {
//         value.into()
//     }
// }
