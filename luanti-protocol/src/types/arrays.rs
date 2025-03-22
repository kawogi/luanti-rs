use std::marker::PhantomData;

use anyhow::bail;

use crate::wire::{
    deser::{Deserialize, DeserializeError, DeserializeResult, Deserializer},
    ser::{Serialize, SerializeResult, Serializer},
};

/// An array of items with no specified length.
/// The length is determined by buffer end.
#[derive(Debug, Clone, PartialEq)]
pub struct Array0<T>(PhantomData<T>);

impl<T: Serialize> Serialize for Array0<T>
where
    <T as Serialize>::Input: Sized,
{
    type Input = Vec<T::Input>;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        for item in value {
            <T as Serialize>::serialize(item, ser)?;
        }
        Ok(())
    }
}

impl<T: Deserialize> Deserialize for Array0<T> {
    type Output = Vec<T::Output>;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        let mut vec = Vec::new();
        while deser.remaining() > 0 {
            vec.push(<T as Deserialize>::deserialize(deser)?);
        }
        Ok(vec)
    }
}

/// An array of items with a u8 length prefix
#[derive(Debug, Clone, PartialEq)]
pub struct Array8<T>(PhantomData<T>);

impl<T: Serialize> Serialize for Array8<T>
where
    <T as Serialize>::Input: Sized,
{
    type Input = Vec<T::Input>;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        u8::serialize(&u8::try_from(value.len())?, ser)?;
        for item in value {
            <T as Serialize>::serialize(item, ser)?;
        }
        Ok(())
    }
}

impl<T: Deserialize> Deserialize for Array8<T> {
    type Output = Vec<T::Output>;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        let length = u8::deserialize(deser)? as usize;
        let mut vec = Vec::with_capacity(length);
        for _ in 0..length {
            vec.push(<T as Deserialize>::deserialize(deser)?);
        }
        Ok(vec)
    }
}

/// An array of items with a u16 length prefix
#[derive(Debug, Clone, PartialEq)]
pub struct Array16<T>(PhantomData<T>);

impl<T: Serialize> Serialize for Array16<T>
where
    <T as Serialize>::Input: Sized,
{
    type Input = Vec<T::Input>;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        u16::serialize(&u16::try_from(value.len())?, ser)?;
        for item in value {
            <T as Serialize>::serialize(item, ser)?;
        }
        Ok(())
    }
}

impl<T: Deserialize> Deserialize for Array16<T> {
    type Output = Vec<T::Output>;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        let length = u16::deserialize(deser)? as usize;
        let mut vec = Vec::with_capacity(length);
        for _ in 0..length {
            vec.push(<T as Deserialize>::deserialize(deser)?);
        }
        Ok(vec)
    }
}

/// An array of items with a u32 length prefix
#[derive(Debug, Clone, PartialEq)]
pub struct Array32<T>(PhantomData<T>);

impl<T: Serialize> Serialize for Array32<T>
where
    <T as Serialize>::Input: Sized,
{
    type Input = Vec<T::Input>;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        u32::serialize(&u32::try_from(value.len())?, ser)?;
        for item in value {
            <T as Serialize>::serialize(item, ser)?;
        }
        Ok(())
    }
}

impl<T: Deserialize> Deserialize for Array32<T> {
    type Output = Vec<T::Output>;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        let length = u32::deserialize(deser)? as usize;
        // Sanity check to prevent memory DoS
        if length > deser.remaining() {
            bail!(DeserializeError::InvalidValue(
                "Array32 length too long".into(),
            ));
        }
        let mut vec = Vec::with_capacity(length);
        for _ in 0..length {
            vec.push(<T as Deserialize>::deserialize(deser)?);
        }
        Ok(vec)
    }
}
