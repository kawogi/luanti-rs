use super::{s16, s32};
use crate::wire::{
    deser::{Deserialize, DeserializeResult, Deserializer},
    ser::{Serialize, SerializeResult, Serializer},
};
use luanti_protocol_derive::{LuantiDeserialize, LuantiSerialize};
use std::ops::{Div, Mul};

#[allow(non_camel_case_types, reason = "aligns with the original C++ codebase")]
#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct v2f {
    pub x: f32,
    pub y: f32,
}

impl v2f {
    #[must_use]
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[allow(non_camel_case_types, reason = "aligns with the original C++ codebase")]
#[derive(Debug, Clone, Copy, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct v3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl v3f {
    #[must_use]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    pub fn as_v3s32(&self) -> v3s32 {
        v3s32 {
            x: self.x.round() as i32,
            y: self.y.round() as i32,
            z: self.z.round() as i32,
        }
    }
}

impl Mul<f32> for v3f {
    type Output = v3f;
    fn mul(self, rhs: f32) -> Self::Output {
        v3f {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Div<f32> for v3f {
    type Output = v3f;
    fn div(self, rhs: f32) -> Self::Output {
        v3f {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

#[allow(non_camel_case_types, reason = "aligns with the original C++ codebase")]
#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct v2u32 {
    pub x: u32,
    pub y: u32,
}

impl v2u32 {
    #[must_use]
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

#[allow(non_camel_case_types, reason = "aligns with the original C++ codebase")]
#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct v2s16 {
    pub x: s16,
    pub y: s16,
}

impl v2s16 {
    #[must_use]
    pub fn new(x: s16, y: s16) -> Self {
        Self { x, y }
    }
}

#[allow(non_camel_case_types, reason = "aligns with the original C++ codebase")]
#[derive(Debug, Clone, Copy, PartialEq, LuantiSerialize, LuantiDeserialize)]
// TODO replace with types from the `glam` crate
pub struct v3s16 {
    pub x: s16,
    pub y: s16,
    pub z: s16,
}

impl v3s16 {
    #[must_use]
    pub fn new(x: s16, y: s16, z: s16) -> Self {
        Self { x, y, z }
    }
}

#[allow(non_camel_case_types, reason = "aligns with the original C++ codebase")]
#[derive(Debug, Clone, Copy, PartialEq, LuantiSerialize, LuantiDeserialize)]
// TODO replace with types from the `glam` crate
pub struct v2s32 {
    pub x: s32,
    pub y: s32,
}

impl v2s32 {
    #[must_use]
    pub fn new(x: s32, y: s32) -> Self {
        Self { x, y }
    }
}

#[allow(non_camel_case_types, reason = "aligns with the original C++ codebase")]
#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct v3s32 {
    pub x: s32,
    pub y: s32,
    pub z: s32,
}

impl v3s32 {
    #[must_use]
    pub fn as_v3f(&self) -> v3f {
        v3f {
            x: self.x as f32,
            y: self.y as f32,
            z: self.z as f32,
        }
    }
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct SColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl SColor {
    #[expect(
        clippy::min_ident_chars,
        reason = "those identifiers are well-known and clear from the context"
    )]
    #[must_use]
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}
