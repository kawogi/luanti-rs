use crate::wire::{
    deser::{Deserialize, DeserializeResult, Deserializer},
    ser::{Serialize, SerializeResult, Serializer, VecSerializer},
};

/// Option is used for optional values at the end of a structure.
/// Once Option is used, all following must be Option as well.
impl<T: Serialize> Serialize for Option<T>
where
    <T as Serialize>::Input: Sized,
{
    type Input = Option<T::Input>;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        match value {
            Some(value) => <T as Serialize>::serialize(value, ser),
            None => Ok(()),
        }
    }
}

impl<T: Deserialize> Deserialize for Option<T> {
    type Output = Option<T::Output>;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        if deser.has_remaining() {
            Ok(Some(<T as Deserialize>::deserialize(deser)?))
        } else {
            Ok(None)
        }
    }
}

// An Optional value controlled by a u16 size parameter.
// Unlike Option, this can appear anywhere in the message.
#[derive(Debug, Clone, PartialEq)]
pub enum Option16<T> {
    None,
    Some(T),
}
impl<T: Serialize> Serialize for Option16<T>
where
    <T as Serialize>::Input: Sized,
{
    type Input = Option16<T::Input>;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        match value {
            Option16::None => u16::serialize(&0_u16, ser),
            Option16::Some(value) => {
                let mut buf = VecSerializer::new(ser.context(), 64);
                <T as Serialize>::serialize(value, &mut buf)?;
                let buf = buf.take();
                let num_bytes = u16::try_from(buf.len())?;
                u16::serialize(&num_bytes, ser)?;
                ser.write_bytes(&buf)?;
                Ok(())
            }
        }
    }
}

impl<T: Deserialize> Deserialize for Option16<T> {
    type Output = Option16<T::Output>;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        match u16::deserialize(deser)? {
            0 => Ok(Option16::None),
            num_bytes => {
                let mut buf = deser.slice(num_bytes as usize)?;
                Ok(Option16::Some(<T as Deserialize>::deserialize(&mut buf)?))
            }
        }
    }
}
