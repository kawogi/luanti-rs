use std::{
    fmt::{self, Display},
    ops::Add,
};

use crate::wire::{
    deser::{Deserialize, DeserializeResult, Deserializer},
    ser::{Serialize, SerializeResult, Serializer},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct WrappingSequenceNumber(u16);

impl WrappingSequenceNumber {
    pub(crate) const INITIAL: Self = Self(0xffdc);
}

impl Display for WrappingSequenceNumber {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl Deserialize for WrappingSequenceNumber {
    type Output = Self;

    fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        u16::deserialize(deserializer).map(Self)
    }
}

impl Serialize for WrappingSequenceNumber {
    type Input = Self;

    fn serialize<S: Serializer>(value: &Self::Input, serializer: &mut S) -> SerializeResult {
        u16::serialize(&value.0, serializer)
    }
}

impl From<u16> for WrappingSequenceNumber {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<WrappingSequenceNumber> for u16 {
    fn from(value: WrappingSequenceNumber) -> Self {
        value.0
    }
}

impl Add<u16> for WrappingSequenceNumber {
    type Output = Self;

    fn add(self, rhs: u16) -> Self::Output {
        Self(self.0.wrapping_add(rhs))
    }
}
