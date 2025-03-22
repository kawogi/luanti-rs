use anyhow::bail;
use luanti_protocol_derive::{LuantiDeserialize, LuantiSerialize};

use crate::wire::{
    deser::{Deserialize, DeserializeError, DeserializeResult, Deserializer},
    ser::{Serialize, SerializeResult, Serializer},
};

use super::{Array16, v3f};

#[derive(Debug, Clone, PartialEq)]
#[expect(
    clippy::large_enum_variant,
    reason = "// TODO consider `Box`ing variants"
)]
pub enum NodeBox {
    Regular,
    Fixed(NodeBoxFixed),
    Wallmounted(NodeBoxWallmounted),
    Leveled(NodeBoxLeveled),
    Connected(NodeBoxConnected),
}

impl Serialize for NodeBox {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        // Unused version number, always 6
        u8::serialize(&6, ser)?;

        let typ = match value {
            NodeBox::Regular => 0,
            NodeBox::Fixed(_) => 1,
            NodeBox::Wallmounted(_) => 2,
            NodeBox::Leveled(_) => 3,
            NodeBox::Connected(_) => 4,
        };
        u8::serialize(&typ, ser)?;
        match value {
            NodeBox::Regular => Ok(()),
            NodeBox::Fixed(value) => NodeBoxFixed::serialize(value, ser),
            NodeBox::Wallmounted(value) => NodeBoxWallmounted::serialize(value, ser),
            NodeBox::Leveled(value) => NodeBoxLeveled::serialize(value, ser),
            NodeBox::Connected(value) => NodeBoxConnected::serialize(value, ser),
        }
    }
}

impl Deserialize for NodeBox {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let ver = u8::deserialize(deser)?;
        if ver != 6 {
            bail!(DeserializeError::InvalidValue("Invalid NodeBox ver".into(),));
        }
        let typ = u8::deserialize(deser)?;
        match typ {
            0 => Ok(NodeBox::Regular),
            1 => Ok(NodeBox::Fixed(NodeBoxFixed::deserialize(deser)?)),
            2 => Ok(NodeBox::Wallmounted(NodeBoxWallmounted::deserialize(
                deser,
            )?)),
            3 => Ok(NodeBox::Leveled(NodeBoxLeveled::deserialize(deser)?)),
            4 => Ok(NodeBox::Connected(NodeBoxConnected::deserialize(deser)?)),
            _ => bail!(DeserializeError::InvalidValue(
                "Invalid NodeBox type".into(),
            )),
        }
    }
}

#[allow(non_camel_case_types, reason = "aligns with the original C++ codebase")]
#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct aabb3f {
    pub min_edge: v3f,
    pub max_edge: v3f,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct NodeBoxLeveled {
    #[wrap(Array16<aabb3f>)]
    pub fixed: Vec<aabb3f>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct NodeBoxFixed {
    #[wrap(Array16<aabb3f>)]
    pub fixed: Vec<aabb3f>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct NodeBoxWallmounted {
    pub wall_top: aabb3f,
    pub wall_bottom: aabb3f,
    pub wall_side: aabb3f,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct NodeBoxConnected {
    #[wrap(Array16<aabb3f>)]
    pub fixed: Vec<aabb3f>,
    #[wrap(Array16<aabb3f>)]
    pub connect_top: Vec<aabb3f>,
    #[wrap(Array16<aabb3f>)]
    pub connect_bottom: Vec<aabb3f>,
    #[wrap(Array16<aabb3f>)]
    pub connect_front: Vec<aabb3f>,
    #[wrap(Array16<aabb3f>)]
    pub connect_left: Vec<aabb3f>,
    #[wrap(Array16<aabb3f>)]
    pub connect_back: Vec<aabb3f>,
    #[wrap(Array16<aabb3f>)]
    pub connect_right: Vec<aabb3f>,
    #[wrap(Array16<aabb3f>)]
    pub disconnected_top: Vec<aabb3f>,
    #[wrap(Array16<aabb3f>)]
    pub disconnected_bottom: Vec<aabb3f>,
    #[wrap(Array16<aabb3f>)]
    pub disconnected_front: Vec<aabb3f>,
    #[wrap(Array16<aabb3f>)]
    pub disconnected_left: Vec<aabb3f>,
    #[wrap(Array16<aabb3f>)]
    pub disconnected_back: Vec<aabb3f>,
    #[wrap(Array16<aabb3f>)]
    pub disconnected_right: Vec<aabb3f>,
    #[wrap(Array16<aabb3f>)]
    pub disconnected: Vec<aabb3f>,
    #[wrap(Array16<aabb3f>)]
    pub disconnected_sides: Vec<aabb3f>,
}
