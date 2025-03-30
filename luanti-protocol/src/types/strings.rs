use crate::wire::{
    deser::{Deserialize, DeserializeError, DeserializeResult, Deserializer},
    ser::{Serialize, SerializeResult, Serializer},
};
use anyhow::bail;
use std::ops::DerefMut;
use std::{marker::PhantomData, ops::Deref};

/// Rust String's must be valid UTF8. But Luanti's strings can contain arbitrary
/// binary data. The only way to store arbitrary bytes is with something like Vec<u8>,
/// which is not String-like. This provides a String-like alternative, that looks nice
/// in debug output.
#[derive(Clone, PartialEq)]
pub struct ByteString(pub Vec<u8>);

impl std::fmt::Debug for ByteString {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Format it as an escaped string
        std::fmt::Debug::fmt(&self.escape_ascii(), formatter)
    }
}

impl ByteString {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn escape_ascii(&self) -> String {
        self.0.escape_ascii().to_string()
    }
}

impl Deref for ByteString {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}

impl DerefMut for ByteString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut_slice()
    }
}

impl From<Vec<u8>> for ByteString {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl From<&[u8]> for ByteString {
    fn from(value: &[u8]) -> Self {
        Self(value.to_vec())
    }
}

/// str implements Serialize but not Deserialize
impl Serialize for str {
    type Input = Self;

    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        u16::serialize(&u16::try_from(value.len())?, ser)?;
        ser.write_bytes(value.as_bytes())
    }
}

impl Serialize for String {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        <str as Serialize>::serialize(value, ser)
    }
}

impl Deserialize for String {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let num_bytes = u16::deserialize(deser)? as usize;
        // very noisy; re-enable if there are protocol errors to be debugged
        // trace!(
        //     "String with {} bytes - {} bytes remaining",
        //     num_bytes,
        //     deser.remaining()
        // );
        match std::str::from_utf8(deser.take(num_bytes)?) {
            Ok(str) => Ok(str.into()),
            Err(error) => bail!(DeserializeError::InvalidValue(error.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LongString(PhantomData<String>);

impl Serialize for LongString {
    type Input = String;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        u32::serialize(&u32::try_from(value.len())?, ser)?;
        ser.write_bytes(value.as_bytes())
    }
}

impl Deserialize for LongString {
    type Output = String;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        let num_bytes = u32::deserialize(deser)? as usize;
        match std::str::from_utf8(deser.take(num_bytes)?) {
            Ok(str) => Ok(str.into()),
            Err(error) => bail!(DeserializeError::InvalidValue(error.to_string())),
        }
    }
}

/// Corresponds to `std::wstring` in C++ land
#[derive(Debug, Clone, PartialEq)]
pub struct WString(PhantomData<String>);

impl Serialize for WString {
    type Input = String;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        let enc: Vec<u16> = value.encode_utf16().collect();

        u16::serialize(&u16::try_from(enc.len())?, ser)?;
        // TODO: This could be made more efficient.
        let mut buf: Vec<u8> = vec![0; 2 * enc.len()];
        let mut index: usize = 0;
        for codepoint in enc {
            buf[index] = (codepoint >> 8) as u8;
            buf[index + 1] = codepoint as u8;
            index += 2;
        }
        ser.write_bytes(&buf)
    }
}

impl Deserialize for WString {
    type Output = String;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        let length = u16::deserialize(deser)? as usize;
        let raw = deser.take(2 * length)?;
        let mut seq: Vec<u16> = vec![0; length];
        for i in 0..length {
            seq[i] = u16::from_be_bytes(raw[2 * i..2 * i + 2].try_into().unwrap());
        }
        match String::from_utf16(&seq) {
            Ok(str) => Ok(str),
            Err(err) => bail!(DeserializeError::InvalidValue(err.to_string())),
        }
    }
}
