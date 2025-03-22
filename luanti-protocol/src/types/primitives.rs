use anyhow::bail;

use crate::wire::{
    deser::{Deserialize, DeserializeResult, Deserializer},
    ser::{Serialize, SerializeResult, Serializer},
};

#[allow(non_camel_case_types, reason = "aligns with the original C++ codebase")]
pub type s8 = i8;

#[allow(non_camel_case_types, reason = "aligns with the original C++ codebase")]
pub type s16 = i16;

#[allow(non_camel_case_types, reason = "aligns with the original C++ codebase")]
pub type s32 = i32;

// Basic types
impl Serialize for bool {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        ser.write_bytes(u8::from(*value).to_be_bytes().as_slice())
    }
}

impl Deserialize for bool {
    type Output = Self;
    fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let byte = deserializer.take_n::<1>()?[0];
        Ok(match byte {
            0 => false,
            1 => true,
            _ => bail!("Invalid bool: {}", byte),
        })
    }
}

impl Serialize for u8 {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        ser.write_bytes(&value.to_be_bytes()[..])
    }
}

impl Deserialize for u8 {
    type Output = Self;
    fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        Ok(deserializer.take_n::<1>()?[0])
    }
}

impl Serialize for u16 {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        ser.write_bytes(&value.to_be_bytes()[..])
    }
}

impl Deserialize for u16 {
    type Output = Self;
    fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        Ok(u16::from_be_bytes(deserializer.take_n::<2>()?))
    }
}

impl Serialize for u32 {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        ser.write_bytes(&value.to_be_bytes()[..])
    }
}

impl Deserialize for u32 {
    type Output = Self;
    fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        Ok(u32::from_be_bytes(deserializer.take_n::<4>()?))
    }
}

impl Serialize for u64 {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        ser.write_bytes(&value.to_be_bytes()[..])
    }
}

impl Deserialize for u64 {
    type Output = Self;
    fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        Ok(u64::from_be_bytes(deserializer.take_n::<8>()?))
    }
}

impl Serialize for i8 {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        ser.write_bytes(&value.to_be_bytes()[..])
    }
}

impl Deserialize for i8 {
    type Output = Self;
    fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        Ok(deserializer.take(1)?[0] as i8)
    }
}

impl Serialize for i16 {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        ser.write_bytes(&value.to_be_bytes()[..])
    }
}

impl Deserialize for i16 {
    type Output = Self;

    fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        Ok(u16::from_be_bytes(deserializer.take_n::<2>()?) as i16)
    }
}

impl Serialize for i32 {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        ser.write_bytes(&value.to_be_bytes()[..])
    }
}

impl Deserialize for i32 {
    type Output = Self;
    fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        Ok(u32::from_be_bytes(deserializer.take_n::<4>()?) as i32)
    }
}

impl Serialize for f32 {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        ser.write_bytes(&value.to_be_bytes()[..])
    }
}

impl Deserialize for f32 {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        Ok(f32::from_be_bytes(deser.take_n::<4>()?))
    }
}
