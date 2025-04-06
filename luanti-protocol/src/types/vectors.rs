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

// #[allow(non_camel_case_types, reason = "aligns with the original C++ codebase")]
// #[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
// pub struct v2f {
//     pub x: f32,
//     pub y: f32,
// }

// impl v2f {
//     #[must_use]
//     pub fn new(x: f32, y: f32) -> Self {
//         Self { x, y }
//     }
// }

// #[allow(non_camel_case_types, reason = "aligns with the original C++ codebase")]
// #[derive(Debug, Default, Clone, Copy, PartialEq, LuantiSerialize, LuantiDeserialize)]
// pub struct Vec3 {
//     pub x: f32,
//     pub y: f32,
//     pub z: f32,
// }

// impl Vec3 {
//     #[must_use]
//     pub fn new(x: f32, y: f32, z: f32) -> Self {
//         Self { x, y, z }
//     }

//     #[must_use]
//     pub fn as_v3s32(&self) -> v3s32 {
//         v3s32 {
//             x: self.x.round() as i32,
//             y: self.y.round() as i32,
//             z: self.z.round() as i32,
//         }
//     }
// }

// impl Mul<f32> for Vec3 {
//     type Output = Vec3;
//     fn mul(self, rhs: f32) -> Self::Output {
//         Vec3 {
//             x: self.x * rhs,
//             y: self.y * rhs,
//             z: self.z * rhs,
//         }
//     }
// }

// impl Div<f32> for Vec3 {
//     type Output = Vec3;
//     fn div(self, rhs: f32) -> Self::Output {
//         Vec3 {
//             x: self.x / rhs,
//             y: self.y / rhs,
//             z: self.z / rhs,
//         }
//     }
// }

// #[allow(non_camel_case_types, reason = "aligns with the original C++ codebase")]
// #[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
// pub struct UVec2 {
//     pub x: u32,
//     pub y: u32,
// }

// impl UVec2 {
//     #[must_use]
//     pub fn new(x: u32, y: u32) -> Self {
//         Self { x, y }
//     }
// }

// #[allow(non_camel_case_types, reason = "aligns with the original C++ codebase")]
// #[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
// pub struct I16Vec2 {
//     pub x: s16,
//     pub y: s16,
// }

// impl I16Vec2 {
//     #[must_use]
//     pub fn new(x: s16, y: s16) -> Self {
//         Self { x, y }
//     }
// }

// #[allow(non_camel_case_types, reason = "aligns with the original C++ codebase")]
// #[derive(Debug, Clone, Copy, PartialEq, LuantiSerialize, LuantiDeserialize)]
// // TODO replace with types from the `glam` crate
// pub struct I16Vec3 {
//     pub x: s16,
//     pub y: s16,
//     pub z: s16,
// }

// impl I16Vec3 {
//     #[must_use]
//     pub fn new(x: s16, y: s16, z: s16) -> Self {
//         Self { x, y, z }
//     }
// }

// #[allow(non_camel_case_types, reason = "aligns with the original C++ codebase")]
// #[derive(Debug, Clone, Copy, PartialEq, LuantiSerialize, LuantiDeserialize)]
// // TODO replace with types from the `glam` crate
// pub struct IVec2 {
//     pub x: s32,
//     pub y: s32,
// }

// impl IVec2 {
//     #[must_use]
//     pub fn new(x: s32, y: s32) -> Self {
//         Self { x, y }
//     }
// }

// #[allow(non_camel_case_types, reason = "aligns with the original C++ codebase")]
// #[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
// pub struct IVec3 {
//     pub x: s32,
//     pub y: s32,
//     pub z: s32,
// }

// impl IVec3 {
//     #[must_use]
//     pub fn as_v3f(&self) -> Vec3 {
//         Vec3 {
//             x: self.x as f32,
//             y: self.y as f32,
//             z: self.z as f32,
//         }
//     }
// }
