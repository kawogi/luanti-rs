use anyhow::bail;
use glam::{IVec2, Vec2, Vec3};
use luanti_protocol_derive::{LuantiDeserialize, LuantiSerialize};

use crate::wire::{
    deser::{Deserialize, DeserializeError, DeserializeResult, Deserializer},
    ser::{Serialize, SerializeResult, Serializer},
};

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct HudchangeCommand {
    pub server_id: u32,
    pub stat: HudStat,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HudStat {
    Pos(Vec2),
    Name(String),
    Scale(Vec2),
    Text(String),
    Number(u32),
    Item(u32),
    Dir(u32),
    Align(Vec2),
    Offset(Vec2),
    WorldPos(Vec3),
    Size(IVec2),
    ZIndex(u32),
    Text2(String),
    Style(u32),
}

impl Serialize for HudStat {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        #![allow(clippy::enum_glob_use, reason = "improves readability")]
        use HudStat::*;
        match value {
            Pos(value) => {
                u8::serialize(&0, ser)?;
                Vec2::serialize(value, ser)?;
            }
            Name(value) => {
                u8::serialize(&1, ser)?;
                String::serialize(value, ser)?;
            }
            Scale(value) => {
                u8::serialize(&2, ser)?;
                Vec2::serialize(value, ser)?;
            }
            Text(value) => {
                u8::serialize(&3, ser)?;
                String::serialize(value, ser)?;
            }
            Number(value) => {
                u8::serialize(&4, ser)?;
                u32::serialize(value, ser)?;
            }
            Item(value) => {
                u8::serialize(&5, ser)?;
                u32::serialize(value, ser)?;
            }
            Dir(value) => {
                u8::serialize(&6, ser)?;
                u32::serialize(value, ser)?;
            }
            Align(value) => {
                u8::serialize(&7, ser)?;
                Vec2::serialize(value, ser)?;
            }
            Offset(value) => {
                u8::serialize(&8, ser)?;
                Vec2::serialize(value, ser)?;
            }
            WorldPos(value) => {
                u8::serialize(&9, ser)?;
                Vec3::serialize(value, ser)?;
            }
            Size(value) => {
                u8::serialize(&10, ser)?;
                IVec2::serialize(value, ser)?;
            }
            ZIndex(value) => {
                u8::serialize(&11, ser)?;
                u32::serialize(value, ser)?;
            }
            Text2(value) => {
                u8::serialize(&12, ser)?;
                String::serialize(value, ser)?;
            }
            Style(value) => {
                u8::serialize(&13, ser)?;
                u32::serialize(value, ser)?;
            }
        }
        Ok(())
    }
}

impl Deserialize for HudStat {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        #![allow(clippy::enum_glob_use, reason = "improves readability")]
        use HudStat::*;
        let stat = u8::deserialize(deser)?;
        match stat {
            0 => Ok(Pos(Vec2::deserialize(deser)?)),
            1 => Ok(Name(String::deserialize(deser)?)),
            2 => Ok(Scale(Vec2::deserialize(deser)?)),
            3 => Ok(Text(String::deserialize(deser)?)),
            4 => Ok(Number(u32::deserialize(deser)?)),
            5 => Ok(Item(u32::deserialize(deser)?)),
            6 => Ok(Dir(u32::deserialize(deser)?)),
            7 => Ok(Align(Vec2::deserialize(deser)?)),
            8 => Ok(Offset(Vec2::deserialize(deser)?)),
            9 => Ok(WorldPos(Vec3::deserialize(deser)?)),
            10 => Ok(Size(IVec2::deserialize(deser)?)),
            11 => Ok(ZIndex(u32::deserialize(deser)?)),
            12 => Ok(Text2(String::deserialize(deser)?)),
            13 => Ok(Style(u32::deserialize(deser)?)),
            _ => bail!(DeserializeError::InvalidValue(String::from(
                "HudStat invalid stat",
            ))),
        }
    }
}
