use crate::types::{
    Array16, Array32, Option16, Pair, SColor, SimpleSoundSpec, Wrapped16, ZLibCompressed,
};
use crate::wire::{
    deser::{Deserialize, DeserializeResult, Deserializer},
    ser::{Serialize, SerializeResult, Serializer},
};
use anyhow::bail;
use glam::Vec3;
use luanti_protocol_derive::{LuantiDeserialize, LuantiSerialize};

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct ItemdefCommand {
    #[wrap(ZLibCompressed<ItemdefList>)]
    pub item_def: ItemdefList,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct ItemdefList {
    pub itemdef_manager_version: u8,
    #[wrap(Array16<Wrapped16<ItemDef>>)]
    pub defs: Vec<ItemDef>,
    #[wrap(Array16<ItemAlias>)]
    pub aliases: Vec<ItemAlias>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct ItemDef {
    pub version: u8,
    pub item_type: ItemType,
    pub name: String,
    pub description: String,
    pub inventory_image: String,
    pub wield_image: String,
    pub wield_scale: Vec3,
    pub stack_max: i16,
    pub usable: bool,
    pub liquids_pointable: bool,
    pub tool_capabilities: Option16<ToolCapabilities>,
    #[wrap(Array16<Pair<String, i16>>)]
    pub groups: Vec<(String, i16)>,
    pub node_placement_prediction: String,
    pub sound_place: SimpleSoundSpec,
    pub sound_place_failed: SimpleSoundSpec,
    pub range: f32,
    pub palette_image: String,
    pub color: SColor,
    pub inventory_overlay: String,
    pub wield_overlay: String,
    pub short_description: Option<String>,
    pub sound_use: Option<SimpleSoundSpec>,
    pub sound_use_air: Option<SimpleSoundSpec>,
    pub place_param2: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct ItemAlias {
    pub name: String,
    pub convert_to: String,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct ToolCapabilities {
    pub version: u8,
    pub full_punch_interval: f32,
    pub max_drop_level: i16,
    // (name, tool group cap)
    #[wrap(Array32<Pair<String, ToolGroupCap>>)]
    pub group_caps: Vec<(String, ToolGroupCap)>,
    // (name, rating)
    #[wrap(Array32<Pair<String, i16>>)]
    pub damage_groups: Vec<(String, i16)>,
    pub punch_attack_uses: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct ToolGroupCap {
    pub uses: i16,
    pub maxlevel: i16,
    // (level, time)
    #[wrap(Array32<Pair<i16, f32>>)]
    pub times: Vec<(i16, f32)>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub enum ItemType {
    None,
    Node,
    Craft,
    Tool,
}
