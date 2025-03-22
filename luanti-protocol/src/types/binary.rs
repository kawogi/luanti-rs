use std::marker::PhantomData;

use anyhow::bail;

use crate::wire::{
    deser::{Deserialize, DeserializeError, DeserializeResult, Deserializer},
    ser::{Serialize, SerializeResult, Serializer},
};

// Wrapped in a String (really a BinaryData16) with a 16-bit length
#[derive(Debug, Clone, PartialEq)]
pub struct Wrapped16<T> {
    phantom: PhantomData<T>,
}

impl<T: Serialize> Serialize for Wrapped16<T> {
    type Input = T::Input;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        let marker = ser.write_marker(2)?;
        <T as Serialize>::serialize(value, ser)?;
        let len: u16 = u16::try_from(ser.marker_distance(&marker))?;
        ser.set_marker(marker, &len.to_be_bytes()[..])?;
        Ok(())
    }
}

impl<T: Deserialize> Deserialize for Wrapped16<T> {
    type Output = T::Output;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        let len = u16::deserialize(deser)?;
        let mut restricted_deser = deser.slice(len as usize)?;
        <T as Deserialize>::deserialize(&mut restricted_deser)
    }
}

// Wrapped in a String (really a BinaryData16) with a 16-bit length
#[derive(Debug, Clone, PartialEq)]
pub struct Wrapped32<T> {
    phantom: PhantomData<T>,
}

impl<T: Serialize> Serialize for Wrapped32<T> {
    type Input = T::Input;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        let marker = ser.write_marker(4)?;
        <T as Serialize>::serialize(value, ser)?;
        let len: u32 = u32::try_from(ser.marker_distance(&marker))?;
        ser.set_marker(marker, &len.to_be_bytes()[..])?;
        Ok(())
    }
}

impl<T: Deserialize> Deserialize for Wrapped32<T> {
    type Output = T::Output;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        let len = u32::deserialize(deser)?;
        let mut restricted_deser = deser.slice(len as usize)?;
        <T as Deserialize>::deserialize(&mut restricted_deser)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryData16;

impl Serialize for BinaryData16 {
    type Input = Vec<u8>;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        u16::serialize(&u16::try_from(value.len())?, ser)?;
        ser.write_bytes(value)?;
        Ok(())
    }
}

impl Deserialize for BinaryData16 {
    type Output = Vec<u8>;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        let num_bytes = u16::deserialize(deser)? as usize;
        Ok(Vec::from(deser.take(num_bytes)?))
    }
}

/// Binary data preceded by a U32 size
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryData32;

impl Serialize for BinaryData32 {
    type Input = Vec<u8>;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        u32::serialize(&u32::try_from(value.len())?, ser)?;
        ser.write_bytes(value)?;
        Ok(())
    }
}

impl Deserialize for BinaryData32 {
    type Output = Vec<u8>;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        let num_bytes = u32::deserialize(deser)? as usize;
        Ok(Vec::from(deser.take(num_bytes)?))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FixedArray<const COUNT: usize, T>
where
    T: Serialize<Input = T>,
    T: Deserialize<Output = T>,
{
    phantom: PhantomData<T>,
}

impl<const COUNT: usize, T> Serialize for FixedArray<COUNT, T>
where
    T: Serialize<Input = T>,
    T: Deserialize<Output = T>,
{
    type Input = [T; COUNT];
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        for ent in value {
            <T as Serialize>::serialize(ent, ser)?;
        }
        Ok(())
    }
}

impl<const COUNT: usize, T> Deserialize for FixedArray<COUNT, T>
where
    T: Serialize<Input = T>,
    T: Deserialize<Output = T>,
{
    type Output = [T; COUNT];
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        let mut entries = Vec::with_capacity(COUNT);
        for _ in 0..COUNT {
            entries.push(<T as Deserialize>::deserialize(deser)?);
        }
        match entries.try_into() {
            Ok(entries) => Ok(entries),
            Err(_) => bail!(DeserializeError::InvalidValue("FixedArray bug".into())),
        }
    }
}
