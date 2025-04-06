use crate::wire::{
    deser::{Deserialize, DeserializeResult, Deserializer},
    ser::{Serialize, SerializeResult, Serializer},
};
use glam::{I16Vec2, I16Vec3, IVec2, IVec3, U8Vec4, UVec2, Vec2, Vec3};

impl Serialize for Vec2 {
    type Input = Self;

    fn serialize<S: Serializer>(value: &Self::Input, serializer: &mut S) -> SerializeResult {
        f32::serialize(&value.x, serializer)?;
        f32::serialize(&value.y, serializer)?;
        Ok(())
    }
}

impl Deserialize for Vec2 {
    type Output = Self;

    fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        Ok(Self {
            x: f32::deserialize(deserializer)?,
            y: f32::deserialize(deserializer)?,
        })
    }
}

impl Serialize for Vec3 {
    type Input = Self;

    fn serialize<S: Serializer>(value: &Self::Input, serializer: &mut S) -> SerializeResult {
        f32::serialize(&value.x, serializer)?;
        f32::serialize(&value.y, serializer)?;
        f32::serialize(&value.z, serializer)?;
        Ok(())
    }
}

impl Deserialize for Vec3 {
    type Output = Self;

    fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        Ok(Self {
            x: f32::deserialize(deserializer)?,
            y: f32::deserialize(deserializer)?,
            z: f32::deserialize(deserializer)?,
        })
    }
}

impl Serialize for IVec2 {
    type Input = Self;

    fn serialize<S: Serializer>(value: &Self::Input, serializer: &mut S) -> SerializeResult {
        i32::serialize(&value.x, serializer)?;
        i32::serialize(&value.y, serializer)?;
        Ok(())
    }
}

impl Deserialize for IVec2 {
    type Output = Self;

    fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        Ok(Self {
            x: i32::deserialize(deserializer)?,
            y: i32::deserialize(deserializer)?,
        })
    }
}

impl Serialize for IVec3 {
    type Input = Self;

    fn serialize<S: Serializer>(value: &Self::Input, serializer: &mut S) -> SerializeResult {
        i32::serialize(&value.x, serializer)?;
        i32::serialize(&value.y, serializer)?;
        i32::serialize(&value.z, serializer)?;
        Ok(())
    }
}

impl Deserialize for IVec3 {
    type Output = Self;

    fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        Ok(Self {
            x: i32::deserialize(deserializer)?,
            y: i32::deserialize(deserializer)?,
            z: i32::deserialize(deserializer)?,
        })
    }
}

impl Serialize for I16Vec3 {
    type Input = Self;

    fn serialize<S: Serializer>(value: &Self::Input, serializer: &mut S) -> SerializeResult {
        i16::serialize(&value.x, serializer)?;
        i16::serialize(&value.y, serializer)?;
        i16::serialize(&value.z, serializer)?;
        Ok(())
    }
}

impl Deserialize for I16Vec3 {
    type Output = Self;

    fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        Ok(Self {
            x: i16::deserialize(deserializer)?,
            y: i16::deserialize(deserializer)?,
            z: i16::deserialize(deserializer)?,
        })
    }
}

impl Serialize for I16Vec2 {
    type Input = Self;

    fn serialize<S: Serializer>(value: &Self::Input, serializer: &mut S) -> SerializeResult {
        i16::serialize(&value.x, serializer)?;
        i16::serialize(&value.y, serializer)?;
        Ok(())
    }
}

impl Deserialize for I16Vec2 {
    type Output = Self;

    fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        Ok(Self {
            x: i16::deserialize(deserializer)?,
            y: i16::deserialize(deserializer)?,
        })
    }
}

impl Serialize for UVec2 {
    type Input = Self;

    fn serialize<S: Serializer>(value: &Self::Input, serializer: &mut S) -> SerializeResult {
        u32::serialize(&value.x, serializer)?;
        u32::serialize(&value.y, serializer)?;
        Ok(())
    }
}

impl Deserialize for UVec2 {
    type Output = Self;

    fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        Ok(Self {
            x: u32::deserialize(deserializer)?,
            y: u32::deserialize(deserializer)?,
        })
    }
}

impl Serialize for U8Vec4 {
    type Input = Self;

    fn serialize<S: Serializer>(value: &Self::Input, serializer: &mut S) -> SerializeResult {
        u8::serialize(&value.x, serializer)?;
        u8::serialize(&value.y, serializer)?;
        u8::serialize(&value.z, serializer)?;
        u8::serialize(&value.w, serializer)?;
        Ok(())
    }
}

impl Deserialize for U8Vec4 {
    type Output = Self;

    fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        Ok(Self {
            x: u8::deserialize(deserializer)?,
            y: u8::deserialize(deserializer)?,
            z: u8::deserialize(deserializer)?,
            w: u8::deserialize(deserializer)?,
        })
    }
}
