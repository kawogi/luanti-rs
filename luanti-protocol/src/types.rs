//! Luanti data types used inside of Commands / Packets.
//!
//! Derive macros `LuantiSerialize` and `LuantiDeserialize` are used to
//! produce ser/deser methods for many of the structs below. The order of
//! the fields inside the struct determines the order in which they are
//! serialized/deserialized, so be careful modifying anything below.
//! Their serialized representation must stay the same.
//!
//! NOTE: The derive macros currently do not work on structs with generic parameters.
//!
//! TODO(paradust): Having an assert!-like macro that generates Serialize/Deserialize
//! errors instead of aborts may be helpful for cleaning this up.

#![expect(
    clippy::pub_underscore_fields,
    clippy::used_underscore_binding,
    reason = "required for de-/serialization macros"
)]
#![expect(
    clippy::min_ident_chars,
    reason = "those identifiers are well-known and clear from the context"
)]

mod active_object;
mod arrays;
mod binary;
mod compressed;
mod node_box;
mod options;
mod primitives;
mod strings;
mod vectors;

use crate::itos;
use crate::wire::deser::Deserialize;
use crate::wire::deser::DeserializeError;
use crate::wire::deser::DeserializeResult;
use crate::wire::deser::Deserializer;
use crate::wire::packet::LATEST_PROTOCOL_VERSION;
use crate::wire::packet::SER_FMT_HIGHEST_READ;
use crate::wire::ser::Serialize;
use crate::wire::ser::SerializeResult;
use crate::wire::ser::Serializer;
use crate::wire::ser::VecSerializer;
use crate::wire::util::compress_zlib;
use crate::wire::util::decompress_zlib;
use crate::wire::util::deserialize_json_string_if_needed;
use crate::wire::util::next_word;
use crate::wire::util::serialize_json_string_if_needed;
use crate::wire::util::skip_whitespace;
use crate::wire::util::split_by_whitespace;
use crate::wire::util::stoi;
use crate::wire::util::zstd_compress;
use crate::wire::util::zstd_decompress;
pub use active_object::*;
use anyhow::anyhow;
use anyhow::bail;
pub use arrays::*;
pub use binary::*;
pub use compressed::*;
use luanti_protocol_derive::LuantiDeserialize;
use luanti_protocol_derive::LuantiSerialize;
pub use node_box::*;
pub use options::*;
pub use primitives::*;
use std::marker::PhantomData;
pub use strings::*;
pub use vectors::*;

pub type CommandId = u8;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandDirection {
    ToClient,
    ToServer,
}

impl CommandDirection {
    #[must_use]
    pub fn for_send(remote_is_server: bool) -> Self {
        if remote_is_server {
            CommandDirection::ToServer
        } else {
            CommandDirection::ToClient
        }
    }

    #[must_use]
    pub fn for_receive(remote_is_server: bool) -> Self {
        Self::for_send(remote_is_server).flip()
    }

    #[must_use]
    pub fn flip(&self) -> Self {
        match self {
            CommandDirection::ToClient => CommandDirection::ToServer,
            CommandDirection::ToServer => CommandDirection::ToClient,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProtocolContext {
    pub dir: CommandDirection,
    pub protocol_version: u16,
    pub ser_fmt: u8,
}

impl ProtocolContext {
    #[must_use]
    pub fn latest_for_receive(remote_is_server: bool) -> Self {
        Self {
            dir: CommandDirection::for_receive(remote_is_server),
            protocol_version: LATEST_PROTOCOL_VERSION,
            ser_fmt: SER_FMT_HIGHEST_READ,
        }
    }

    #[must_use]
    pub fn latest_for_send(remote_is_server: bool) -> Self {
        Self {
            dir: CommandDirection::for_send(remote_is_server),
            protocol_version: LATEST_PROTOCOL_VERSION,
            ser_fmt: SER_FMT_HIGHEST_READ,
        }
    }
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct AddedObject {
    pub id: u16,
    pub typ: u8,
    #[wrap(Wrapped32<GenericInitData>)]
    pub init_data: GenericInitData,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct MediaFileData {
    pub name: String,
    #[wrap(BinaryData32)]
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct MediaAnnouncement {
    pub name: String,
    pub sha1_base64: String,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct SkyColor {
    pub day_sky: SColor,
    pub day_horizon: SColor,
    pub dawn_sky: SColor,
    pub dawn_horizon: SColor,
    pub night_sky: SColor,
    pub night_horizon: SColor,
    pub indoors: SColor,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct SunParams {
    pub visible: bool,
    pub texture: String,
    pub tonemap: String,
    pub sunrise: String,
    pub sunrise_visible: bool,
    pub scale: f32,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct MoonParams {
    pub visible: bool,
    pub texture: String,
    pub tonemap: String,
    pub scale: f32,
}
#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct StarParams {
    pub visible: bool,
    pub count: u32,
    pub starcolor: SColor,
    pub scale: f32,
    pub day_opacity: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct MinimapMode {
    pub typ: u16,
    pub label: String,
    pub size: u16,
    pub texture: String,
    pub scale: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerPos {
    pub position: v3f,     // serialized as v3s32, *100.0f
    pub speed: v3f,        // serialized as v3s32, *100.0f
    pub pitch: f32,        // serialized as s32, *100.0f
    pub yaw: f32,          // serialized as s32, *100.0f
    pub keys_pressed: u32, // bitset
    pub fov: f32,          // serialized as u8, *80.0f
    pub wanted_range: u8,
}

impl Serialize for PlayerPos {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        let s_position = (value.position * 100_f32).as_v3s32();
        let s_speed = (value.speed * 100_f32).as_v3s32();
        let s_pitch = (value.pitch * 100_f32).round() as s32;
        let s_yaw = (value.yaw * 100_f32).round() as s32;
        let s_fov = (value.fov * 80_f32).round() as u8;

        v3s32::serialize(&s_position, ser)?;
        v3s32::serialize(&s_speed, ser)?;
        i32::serialize(&s_pitch, ser)?;
        i32::serialize(&s_yaw, ser)?;
        u32::serialize(&value.keys_pressed, ser)?;
        u8::serialize(&s_fov, ser)?;
        u8::serialize(&value.wanted_range, ser)?;
        Ok(())
    }
}

impl Deserialize for PlayerPos {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let s_position = v3s32::deserialize(deser)?;
        let s_speed = v3s32::deserialize(deser)?;
        let s_pitch = s32::deserialize(deser)?;
        let s_yaw = s32::deserialize(deser)?;
        let keys_pressed = u32::deserialize(deser)?;
        let s_fov = u8::deserialize(deser)?;
        let wanted_range = u8::deserialize(deser)?;
        Ok(PlayerPos {
            position: s_position.as_v3f() / 100_f32,
            speed: s_speed.as_v3f() / 100_f32,
            pitch: (s_pitch as f32) / 100_f32,
            yaw: (s_yaw as f32) / 100_f32,
            keys_pressed,
            fov: f32::from(s_fov) / 80_f32,
            wanted_range,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pair<T1, T2>(PhantomData<(T1, T2)>);

impl<T1: Serialize, T2: Serialize> Serialize for Pair<T1, T2>
where
    <T1 as Serialize>::Input: Sized,
    <T2 as Serialize>::Input: Sized,
{
    type Input = (T1::Input, T2::Input);
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        <T1 as Serialize>::serialize(&value.0, ser)?;
        <T2 as Serialize>::serialize(&value.1, ser)?;
        Ok(())
    }
}

impl<T1: Deserialize, T2: Deserialize> Deserialize for Pair<T1, T2> {
    type Output = (T1::Output, T2::Output);
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        Ok((
            <T1 as Deserialize>::deserialize(deser)?,
            <T2 as Deserialize>::deserialize(deser)?,
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimapModeList {
    pub mode: u16,
    pub vec: Vec<MinimapMode>,
}

impl Serialize for MinimapModeList {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        // The length of the list is a u16 which precedes `mode`,
        // which makes the layout not fit into any usual pattern.
        u16::serialize(&u16::try_from(value.vec.len())?, ser)?;
        u16::serialize(&value.mode, ser)?;
        for mode in &value.vec {
            MinimapMode::serialize(mode, ser)?;
        }
        Ok(())
    }
}

impl Deserialize for MinimapModeList {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let count = u16::deserialize(deser)?;
        let mode = u16::deserialize(deser)?;
        let mut vec: Vec<MinimapMode> = Vec::with_capacity(count as usize);
        for _ in 0..count {
            vec.push(MinimapMode::deserialize(deser)?);
        }
        Ok(MinimapModeList { mode, vec })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuthMechsBitset {
    pub legacy_password: bool,
    pub srp: bool,
    pub first_srp: bool,
}

impl Serialize for AuthMechsBitset {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        let mut bits: u32 = 0;
        if value.legacy_password {
            bits |= 1;
        }
        if value.srp {
            bits |= 2;
        }
        if value.first_srp {
            bits |= 4;
        }
        u32::serialize(&bits, ser)
    }
}

impl Deserialize for AuthMechsBitset {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let value = u32::deserialize(deser)?;
        Ok(AuthMechsBitset {
            legacy_password: (value & 1) != 0,
            srp: (value & 2) != 0,
            first_srp: (value & 4) != 0,
        })
    }
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct SimpleSoundSpec {
    pub name: String,
    pub gain: f32,
    pub pitch: f32,
    pub fade: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileDef {
    pub name: String,
    pub animation: TileAnimationParams,
    // These are stored in a single u8 flags
    pub backface_culling: bool,
    pub tileable_horizontal: bool,
    pub tileable_vertical: bool,
    // The flags also determine which of these is present
    pub color_rgb: Option<(u8, u8, u8)>,
    pub scale: u8,
    pub align_style: AlignStyle,
}

const TILE_FLAG_BACKFACE_CULLING: u16 = 1 << 0;
const TILE_FLAG_TILEABLE_HORIZONTAL: u16 = 1 << 1;
const TILE_FLAG_TILEABLE_VERTICAL: u16 = 1 << 2;
const TILE_FLAG_HAS_COLOR: u16 = 1 << 3;
const TILE_FLAG_HAS_SCALE: u16 = 1 << 4;
const TILE_FLAG_HAS_ALIGN_STYLE: u16 = 1 << 5;

impl Serialize for TileDef {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        u8::serialize(&6, ser)?; // tiledef version
        String::serialize(&value.name, ser)?;
        TileAnimationParams::serialize(&value.animation, ser)?;
        let mut flags: u16 = 0;
        if value.backface_culling {
            flags |= TILE_FLAG_BACKFACE_CULLING;
        }
        if value.tileable_horizontal {
            flags |= TILE_FLAG_TILEABLE_HORIZONTAL;
        }
        if value.tileable_vertical {
            flags |= TILE_FLAG_TILEABLE_VERTICAL;
        }
        if value.color_rgb.is_some() {
            flags |= TILE_FLAG_HAS_COLOR;
        }
        if value.scale != 0 {
            flags |= TILE_FLAG_HAS_SCALE;
        }
        if value.align_style != AlignStyle::Node {
            flags |= TILE_FLAG_HAS_ALIGN_STYLE;
        }
        u16::serialize(&flags, ser)?;
        if let Some(color) = &value.color_rgb {
            u8::serialize(&color.0, ser)?;
            u8::serialize(&color.1, ser)?;
            u8::serialize(&color.2, ser)?;
        }
        if value.scale != 0 {
            u8::serialize(&value.scale, ser)?;
        }
        if value.align_style != AlignStyle::Node {
            AlignStyle::serialize(&value.align_style, ser)?;
        }
        Ok(())
    }
}

impl Deserialize for TileDef {
    type Output = Self;
    fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let version: u8 = u8::deserialize(deserializer)?;
        if version != 6 {
            bail!(DeserializeError::InvalidValue(
                "Invalid TileDef version".into(),
            ));
        }
        let name = String::deserialize(deserializer)?;
        let animation = TileAnimationParams::deserialize(deserializer)?;
        let flags = u16::deserialize(deserializer)?;
        #[expect(clippy::if_then_some_else_none, reason = "`?`-operator prohibits this")]
        let color = if (flags & TILE_FLAG_HAS_COLOR) != 0 {
            Some((
                u8::deserialize(deserializer)?,
                u8::deserialize(deserializer)?,
                u8::deserialize(deserializer)?,
            ))
        } else {
            None
        };
        let scale = if (flags & TILE_FLAG_HAS_SCALE) != 0 {
            u8::deserialize(deserializer)?
        } else {
            0
        };
        let align_style = if (flags & TILE_FLAG_HAS_ALIGN_STYLE) != 0 {
            AlignStyle::deserialize(deserializer)?
        } else {
            AlignStyle::Node
        };

        Ok(Self {
            name,
            animation,
            backface_culling: (flags & TILE_FLAG_BACKFACE_CULLING) != 0,
            tileable_horizontal: (flags & TILE_FLAG_TILEABLE_HORIZONTAL) != 0,
            tileable_vertical: (flags & TILE_FLAG_TILEABLE_VERTICAL) != 0,
            color_rgb: color,
            scale,
            align_style,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TileAnimationParams {
    None,
    VerticalFrames {
        aspect_w: u16,
        aspect_h: u16,
        length: f32,
    },
    Sheet2D {
        frames_w: u8,
        frames_h: u8,
        frame_length: f32,
    },
}

// TileAnimationType
const TAT_NONE: u8 = 0;
const TAT_VERTICAL_FRAMES: u8 = 1;
const TAT_SHEET_2D: u8 = 2;

impl Serialize for TileAnimationParams {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        let typ = match value {
            TileAnimationParams::None => TAT_NONE,
            TileAnimationParams::VerticalFrames { .. } => TAT_VERTICAL_FRAMES,
            TileAnimationParams::Sheet2D { .. } => TAT_SHEET_2D,
        };
        u8::serialize(&typ, ser)?;
        match value {
            TileAnimationParams::None => {}
            TileAnimationParams::VerticalFrames {
                aspect_w,
                aspect_h,
                length,
            } => {
                u16::serialize(aspect_w, ser)?;
                u16::serialize(aspect_h, ser)?;
                f32::serialize(length, ser)?;
            }
            TileAnimationParams::Sheet2D {
                frames_w,
                frames_h,
                frame_length,
            } => {
                u8::serialize(frames_w, ser)?;
                u8::serialize(frames_h, ser)?;
                f32::serialize(frame_length, ser)?;
            }
        };
        Ok(())
    }
}

impl Deserialize for TileAnimationParams {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let typ = u8::deserialize(deser)?;
        match typ {
            TAT_NONE => Ok(TileAnimationParams::None),
            TAT_VERTICAL_FRAMES => Ok(TileAnimationParams::VerticalFrames {
                aspect_w: u16::deserialize(deser)?,
                aspect_h: u16::deserialize(deser)?,
                length: f32::deserialize(deser)?,
            }),
            TAT_SHEET_2D => Ok(TileAnimationParams::Sheet2D {
                frames_w: u8::deserialize(deser)?,
                frames_h: u8::deserialize(deser)?,
                frame_length: f32::deserialize(deser)?,
            }),
            _ => bail!(DeserializeError::InvalidValue(format!(
                "Invalid TileAnimationParams type {} at: {:?}",
                typ, deser.data
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub enum AlignStyle {
    Node,
    World,
    UserDefined,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub enum DrawType {
    Normal,
    AirLike,
    Liquid,
    FlowingLiquid,
    GlassLike,
    AllFaces,
    AllFacesOptional,
    TorchLike,
    SignLike,
    PlantLike,
    FenceLike,
    RailLike,
    NodeBox,
    GlassLikeFramed,
    FireLike,
    GlassLikeFramedOptional,
    Mesh,
    PlantLikeRooted,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
#[expect(clippy::struct_excessive_bools, reason = "this is mandated by the API")]
pub struct ContentFeatures {
    pub version: u8,
    pub name: String,
    #[wrap(Array16<Pair<String, s16>>)]
    pub groups: Vec<(String, s16)>,
    pub param_type: u8,
    pub param_type_2: u8,
    pub drawtype: DrawType,
    pub mesh: String,
    pub visual_scale: f32,
    // this was an attempt to be tiledef length, but then they added an extra 6 tiledefs without fixing it
    pub unused_six: u8,
    #[wrap(FixedArray<6, TileDef>)]
    pub tiledef: [TileDef; 6],
    #[wrap(FixedArray<6, TileDef>)]
    pub tiledef_overlay: [TileDef; 6],
    #[wrap(Array8<TileDef>)]
    pub tiledef_special: Vec<TileDef>,
    pub alpha_for_legacy: u8,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub palette_name: String,
    pub waving: u8,
    pub connect_sides: u8,
    #[wrap(Array16<u16>)]
    pub connects_to_ids: Vec<u16>,
    pub post_effect_color: SColor,
    pub leveled: u8,
    pub light_propagates: u8,
    pub sunlight_propagates: u8,
    pub light_source: u8,
    pub is_ground_content: bool,
    pub walkable: bool,
    pub pointable: bool,
    pub diggable: bool,
    pub climbable: bool,
    pub buildable_to: bool,
    pub rightclickable: bool,
    pub damage_per_second: u32,
    pub liquid_type_bc: u8,
    pub liquid_alternative_flowing: String,
    pub liquid_alternative_source: String,
    pub liquid_viscosity: u8,
    pub liquid_renewable: bool,
    pub liquid_range: u8,
    pub drowning: u8,
    pub floodable: bool,
    pub node_box: NodeBox,
    pub selection_box: NodeBox,
    pub collision_box: NodeBox,
    pub sound_footstep: SimpleSoundSpec,
    pub sound_dig: SimpleSoundSpec,
    pub sound_dug: SimpleSoundSpec,
    pub legacy_facedir_simple: bool,
    pub legacy_wallmounted: bool,
    pub node_dig_prediction: Option<String>,
    pub leveled_max: Option<u8>,
    pub alpha: Option<AlphaMode>,
    pub move_resistance: Option<u8>,
    pub liquid_move_physics: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub enum AlphaMode {
    Blend,
    Clip,
    Opaque,
    LegacyCompat,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeDefManager {
    pub content_features: Vec<(u16, ContentFeatures)>,
}

/// The way this structure is encoded is really unusual, in order to
/// allow the `ContentFeatures` to be extended in the future without
/// changing the encoding.
impl Serialize for NodeDefManager {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        // Version
        u8::serialize(&1, ser)?;
        let count: u16 = u16::try_from(value.content_features.len())?;
        u16::serialize(&count, ser)?;
        // The serialization of content_features is wrapped in a String32
        // Write a marker so we can write the size later
        let string32_wrapper = ser.write_marker(4)?;
        for (index, features) in &value.content_features {
            u16::serialize(index, ser)?;
            // The contents of each feature is wrapped in a String16.
            let string16_wrapper = ser.write_marker(2)?;
            ContentFeatures::serialize(features, ser)?;
            let len: u16 = u16::try_from(ser.marker_distance(&string16_wrapper))?;
            ser.set_marker(string16_wrapper, &len.to_be_bytes()[..])?;
        }
        let len: u32 = u32::try_from(ser.marker_distance(&string32_wrapper))?;
        ser.set_marker(string32_wrapper, &len.to_be_bytes()[..])?;
        Ok(())
    }
}

impl Deserialize for NodeDefManager {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let version = u8::deserialize(deser)?;
        if version != 1 {
            bail!(DeserializeError::InvalidValue(
                "Bad NodeDefManager version".into(),
            ));
        }
        let count: u16 = u16::deserialize(deser)?;
        let string32_wrapper_len: u32 = u32::deserialize(deser)?;
        // Shadow deser with a restricted deserializer
        let mut deser = deser.slice(string32_wrapper_len as usize)?;
        let mut content_features: Vec<(u16, ContentFeatures)> = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let i = u16::deserialize(&mut deser)?;
            let string16_wrapper_len: u16 = u16::deserialize(&mut deser)?;
            let mut inner_deser = deser.slice(string16_wrapper_len as usize)?;
            let features = ContentFeatures::deserialize(&mut inner_deser)?;
            content_features.push((i, features));
        }
        Ok(Self { content_features })
    }
}

// A "block" is 16x16x16 "nodes"
const MAP_BLOCKSIZE: u16 = 16;

// Number of nodes in a block
const NODECOUNT: u16 = MAP_BLOCKSIZE * MAP_BLOCKSIZE * MAP_BLOCKSIZE;

#[derive(Debug, Clone, PartialEq)]
pub struct MapBlock {
    pub is_underground: bool,
    pub day_night_diff: bool,
    pub generated: bool,
    pub lighting_complete: Option<u16>,
    pub nodes: MapNodesBulk,
    pub node_metadata: NodeMetadataList, // m_node_metadata.serialize(os, version, disk);
}

impl Serialize for MapBlock {
    /// `MapBlock` is a bit of a nightmare, because the compression algorithm
    /// and where the compression is applied (to the whole struct, or to
    /// parts of it) depends on the serialization format version.
    ///
    /// For now, only `ser_fmt` >= 28 is supported.
    /// For ver 28, only the nodes and nodemeta are compressed using zlib.
    /// For >= 29, the entire thing is compressed using zstd.
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, serializer: &mut S) -> SerializeResult {
        let ver = serializer.context().ser_fmt;
        let mut tmp_ser = VecSerializer::new(serializer.context(), 0x8000);
        let ser = &mut tmp_ser;
        let header = MapBlockHeader {
            is_underground: value.is_underground,
            day_night_diff: value.day_night_diff,
            generated: value.generated,
            lighting_complete: value.lighting_complete,
        };
        MapBlockHeader::serialize(&header, ser)?;
        if ver >= 29 {
            MapNodesBulk::serialize(&value.nodes, ser)?;
        } else {
            // Serialize and compress using zlib
            let mut inner = VecSerializer::new(ser.context(), 0x8000);
            MapNodesBulk::serialize(&value.nodes, &mut inner)?;
            let compressed = compress_zlib(&inner.take());
            ser.write_bytes(&compressed)?;
        }
        if ver >= 29 {
            NodeMetadataList::serialize(&value.node_metadata, ser)?;
        } else {
            // Serialize and compress using zlib
            let mut inner = VecSerializer::new(ser.context(), 0x8000);
            NodeMetadataList::serialize(&value.node_metadata, &mut inner)?;
            let compressed = compress_zlib(&inner.take());
            ser.write_bytes(&compressed)?;
        }
        if ver >= 29 {
            // The whole thing is zstd compressed
            let tmp = tmp_ser.take();
            zstd_compress(&tmp, |chunk| serializer.write_bytes(chunk))?;
        } else {
            // Just write it directly
            let tmp = tmp_ser.take();
            serializer.write_bytes(&tmp)?;
        }
        Ok(())
    }
}

///
/// This is a helper for `MapBlock` ser/deser
/// Not exposed publicly.
#[derive(Debug)]
struct MapBlockHeader {
    pub is_underground: bool,
    pub day_night_diff: bool,
    pub generated: bool,
    pub lighting_complete: Option<u16>,
}

impl Serialize for MapBlockHeader {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        let mut flags: u8 = 0;
        if value.is_underground {
            flags |= 0x1;
        }
        if value.day_night_diff {
            flags |= 0x2;
        }
        if !value.generated {
            flags |= 0x8;
        }
        u8::serialize(&flags, ser)?;
        if ser.context().ser_fmt >= 27 {
            if let Some(lighting_complete) = value.lighting_complete {
                u16::serialize(&lighting_complete, ser)?;
            } else {
                bail!("lighting_complete must be set for ver >= 27");
            }
        }
        u8::serialize(&2, ser)?; // content_width == 2
        u8::serialize(&2, ser)?; // params_width == 2
        Ok(())
    }
}

impl Deserialize for MapBlockHeader {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let flags = u8::deserialize(deser)?;
        if flags != (flags & (0x1 | 0x2 | 0x8)) {
            bail!(DeserializeError::InvalidValue(
                "Invalid MapBlock flags".into(),
            ));
        }
        #[expect(clippy::if_then_some_else_none, reason = "`?`-operator prohibits this")]
        let lighting_complete = if deser.context().ser_fmt >= 27 {
            Some(u16::deserialize(deser)?)
        } else {
            None
        };
        let content_width = u8::deserialize(deser)?;
        let params_width = u8::deserialize(deser)?;
        if content_width != 2 || params_width != 2 {
            bail!(DeserializeError::InvalidValue(
                "Corrupt MapBlock: content_width and params_width not both 2".into(),
            ));
        }
        Ok(Self {
            is_underground: (flags & 0x1) != 0,
            day_night_diff: (flags & 0x2) != 0,
            generated: (flags & 0x8) == 0,
            lighting_complete,
        })
    }
}

impl Deserialize for MapBlock {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let ver = deser.context().ser_fmt;
        if ver < 28 {
            bail!("Unsupported ser fmt");
        }
        // TODO(paradust): I can't make the borrow checker happy with sharing
        // code here, so for now the code has two different paths.
        if ver >= 29 {
            let mut tmp: Vec<u8> = Vec::new();
            // Decompress to a temporary buffer
            let bytes_taken = zstd_decompress(deser.peek_all(), |chunk| {
                tmp.extend_from_slice(chunk);
                Ok(())
            })?;
            deser.take(bytes_taken)?;
            let deser = &mut Deserializer::new(deser.context(), &tmp);
            let header = MapBlockHeader::deserialize(deser)?;
            let nodes = MapNodesBulk::deserialize(deser)?;
            let node_metadata = NodeMetadataList::deserialize(deser)?;
            Ok(Self {
                is_underground: header.is_underground,
                day_night_diff: header.day_night_diff,
                generated: header.generated,
                lighting_complete: header.lighting_complete,
                nodes,
                node_metadata,
            })
        } else {
            let header = MapBlockHeader::deserialize(deser)?;
            let (consumed1, nodes_raw) = decompress_zlib(deser.peek_all())?;
            deser.take(consumed1)?;
            let nodes = {
                let mut tmp = Deserializer::new(deser.context(), &nodes_raw);
                MapNodesBulk::deserialize(&mut tmp)?
            };
            let (consumed2, metadata_raw) = decompress_zlib(deser.peek_all())?;
            deser.take(consumed2)?;
            let node_metadata = {
                let mut tmp = Deserializer::new(deser.context(), &metadata_raw);
                NodeMetadataList::deserialize(&mut tmp)?
            };
            Ok(Self {
                is_underground: header.is_underground,
                day_night_diff: header.day_night_diff,
                generated: header.generated,
                lighting_complete: header.lighting_complete,
                nodes,
                node_metadata,
            })
        }
    }
}

/// This has a special serialization, presumably to make it compress better.
/// Each param is stored in a separate array.
#[derive(Debug, Clone, PartialEq)]
pub struct MapNodesBulk {
    pub nodes: [MapNode; NODECOUNT as usize],
}

impl Serialize for MapNodesBulk {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        let nodecount = NODECOUNT as usize;
        // Write all param0 first
        ser.write(2 * nodecount, |buf| {
            assert_eq!(buf.len(), 2 * nodecount, "size mismatch");
            for index in 0..nodecount {
                let bytes = value.nodes[index].param0.to_be_bytes();
                buf[2 * index] = bytes[0];
                buf[2 * index + 1] = bytes[1];
            }
        })?;
        // Write all param1
        ser.write(nodecount, |buf| {
            assert_eq!(buf.len(), nodecount, "size mismatch");
            #[expect(
                clippy::needless_range_loop,
                reason = "// TODO transform into iterator"
            )]
            for index in 0..nodecount {
                buf[index] = value.nodes[index].param1;
            }
        })?;
        // Write all param2
        ser.write(nodecount, |buf| {
            assert_eq!(buf.len(), nodecount, "size mismatch");
            #[expect(
                clippy::needless_range_loop,
                reason = "// TODO transform into iterator"
            )]
            for i in 0..nodecount {
                buf[i] = value.nodes[i].param2;
            }
        })?;
        Ok(())
    }
}

impl Deserialize for MapNodesBulk {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let nodecount = NODECOUNT as usize;
        let data = deser.take(4 * nodecount)?;
        let mut nodes: Vec<MapNode> = Vec::with_capacity(nodecount);
        let param1_offset = 2 * nodecount;
        let param2_offset = 3 * nodecount;
        for i in 0..nodecount {
            nodes.push(MapNode {
                param0: u16::from_be_bytes(data[2 * i..2 * i + 2].try_into().unwrap()),
                param1: data[param1_offset + i],
                param2: data[param2_offset + i],
            });
        }
        Ok(Self {
            nodes: match nodes.try_into() {
                Ok(value) => value,
                Err(_) => bail!("Bug in MapNodesBulk"),
            },
        })
    }
}

/// The default serialization is used for single nodes.
/// But for transferring entire blocks, `MapNodeBulk` is used instead.
#[derive(Debug, Default, Clone, Copy, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct MapNode {
    pub param0: u16,
    pub param1: u8,
    pub param2: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeMetadataList {
    pub metadata: Vec<(BlockPos, NodeMetadata)>,
}

impl Serialize for NodeMetadataList {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        if value.metadata.is_empty() {
            u8::serialize(&0, ser)?; // version 0 indicates no data
            return Ok(());
        }
        u8::serialize(&2, ser)?; // version == 2
        <Array16<Pair<BlockPos, NodeMetadata>> as Serialize>::serialize(&value.metadata, ser)?;
        Ok(())
    }
}

impl Deserialize for NodeMetadataList {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let ver = u8::deserialize(deser)?;
        if ver == 0 {
            Ok(Self {
                metadata: Vec::new(),
            })
        } else if ver == 2 {
            Ok(Self {
                metadata: <Array16<Pair<BlockPos, NodeMetadata>> as Deserialize>::deserialize(
                    deser,
                )?,
            })
        } else {
            bail!(DeserializeError::InvalidValue(
                "Invalid NodeMetadataList version".into(),
            ))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AbsNodeMetadataList {
    pub metadata: Vec<(AbsBlockPos, NodeMetadata)>,
}

impl Serialize for AbsNodeMetadataList {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        if value.metadata.is_empty() {
            u8::serialize(&0, ser)?; // version 0 indicates no data
            return Ok(());
        }
        u8::serialize(&2, ser)?; // version == 2
        <Array16<Pair<AbsBlockPos, NodeMetadata>> as Serialize>::serialize(&value.metadata, ser)?;
        Ok(())
    }
}

impl Deserialize for AbsNodeMetadataList {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let ver = u8::deserialize(deser)?;
        if ver == 0 {
            Ok(Self {
                metadata: Vec::new(),
            })
        } else if ver == 2 {
            Ok(Self {
                metadata: <Array16<Pair<AbsBlockPos, NodeMetadata>> as Deserialize>::deserialize(
                    deser,
                )?,
            })
        } else {
            bail!(DeserializeError::InvalidValue(
                "Invalid AbsNodeMetadataList version".into(),
            ))
        }
    }
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct AbsBlockPos {
    pos: v3s16,
}

/// `BlockPos` addresses a node within a block
/// It is equivalent to (16*z + y)*16 + x, where x,y,z are from 0 to 15.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockPos {
    pub raw: u16,
}

impl BlockPos {
    #[must_use]
    pub fn new(x: s16, y: s16, z: s16) -> Self {
        let valid = 0..(MAP_BLOCKSIZE as s16);
        assert!(
            valid.contains(&x) && valid.contains(&y) && valid.contains(&z),
            "//TODO add proper error message"
        );
        let x = x as u16;
        let y = y as u16;
        let z = z as u16;
        Self {
            raw: (MAP_BLOCKSIZE * z + y) * MAP_BLOCKSIZE + x,
        }
    }

    #[must_use]
    pub fn from_xyz(pos: v3s16) -> Self {
        Self::new(pos.x, pos.y, pos.z)
    }

    #[must_use]
    pub fn to_xyz(&self) -> v3s16 {
        let x = self.raw % 16;
        let y = (self.raw / 16) % 16;
        let z = (self.raw / 256) % 16;
        v3s16::new(x as i16, y as i16, z as i16)
    }
}

impl Serialize for BlockPos {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        u16::serialize(&value.raw, ser)?;
        Ok(())
    }
}

impl Deserialize for BlockPos {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let raw = u16::deserialize(deser)?;
        if raw >= 4096 {
            bail!(DeserializeError::InvalidValue("Invalid BlockPos".into(),))
        }
        Ok(Self { raw })
    }
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct NodeMetadata {
    #[wrap(Array32<StringVar>)]
    pub stringvars: Vec<StringVar>,
    pub inventory: Inventory,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct StringVar {
    pub name: String,
    #[wrap(BinaryData32)]
    pub value: Vec<u8>,
    pub is_private: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Inventory {
    pub entries: Vec<InventoryEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InventoryEntry {
    // Inventory lists to keep
    KeepList(String),
    // Inventory lists to add or update
    Update(InventoryList),
}

/// Inventory is sent as a "almost" line-based text format.
/// Unfortunately there's no way to simplify this code, it has to mirror
/// the way Luanti does it exactly, because it is so arbitrary.
impl Serialize for Inventory {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        for entry in &value.entries {
            match entry {
                InventoryEntry::KeepList(list_name) => {
                    // TODO(paradust): Performance. A format!-like macro that
                    //                 writes directly to ser could be faster.
                    ser.write_bytes(b"KeepList ")?;
                    ser.write_bytes(list_name.as_bytes())?;
                    ser.write_bytes(b"\n")?;
                }
                InventoryEntry::Update(list) => {
                    // Takes care of the List header line
                    InventoryList::serialize(list, ser)?;
                }
            }
        }
        ser.write_bytes(b"EndInventory\n")?;
        Ok(())
    }
}

impl Deserialize for Inventory {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let mut result = Self {
            entries: Vec::new(),
        };
        while deser.has_remaining() {
            // Peek the line, but don't take it yet.
            let line = deser.peek_line()?;
            let words = split_by_whitespace(line);
            if words.is_empty() {
                deser.take_line()?;
                continue;
            }
            let name = words[0];
            if name == b"EndInventory" || name == b"End" {
                // Take the line
                deser.take_line()?;
                return Ok(result);
            } else if name == b"List" {
                // InventoryList will take the line
                result
                    .entries
                    .push(InventoryEntry::Update(InventoryList::deserialize(deser)?));
            } else if name == b"KeepList" {
                if words.len() < 2 {
                    bail!(DeserializeError::InvalidValue(
                        "KeepList missing name".into(),
                    ));
                }
                match std::str::from_utf8(words[1]) {
                    Ok(str) => result.entries.push(InventoryEntry::KeepList(str.into())),
                    Err(_) => {
                        bail!(DeserializeError::InvalidValue(
                            "KeepList name is invalid UTF8".into(),
                        ))
                    }
                }
                // Take the line
                deser.take_line()?;
            } else {
                // Anything else is supposed to be ignored. Gross.
                deser.take_line()?;
            }
        }
        // If we ran out before seeing the end marker, it's an error
        bail!(DeserializeError::Eof("Inventory::deserialize(_)".into()))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InventoryList {
    pub name: String,
    pub width: u32,
    pub items: Vec<ItemStackUpdate>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemStackUpdate {
    Empty,
    Keep, // this seems to not be used yet
    Item(ItemStack),
}

impl Serialize for InventoryList {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        // List <name> <size>
        ser.write_bytes(b"List ")?;
        ser.write_bytes(value.name.as_bytes())?;
        ser.write_bytes(b" ")?;
        ser.write_bytes(value.items.len().to_string().as_bytes())?;
        ser.write_bytes(b"\n")?;

        // Width <width>
        ser.write_bytes(b"Width ")?;
        ser.write_bytes(value.width.to_string().as_bytes())?;
        ser.write_bytes(b"\n")?;

        for item in &value.items {
            match item {
                ItemStackUpdate::Empty => ser.write_bytes(b"Empty\n")?,
                ItemStackUpdate::Keep => ser.write_bytes(b"Keep\n")?,
                ItemStackUpdate::Item(item_stack) => {
                    // Writes Item line
                    ItemStack::serialize(item_stack, ser)?;
                }
            }
        }
        ser.write_bytes(b"EndInventoryList\n")?;
        Ok(())
    }
}

impl Deserialize for InventoryList {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        // First line should be: List <name> <item_count>
        let line = deser.take_line()?;
        let words = split_by_whitespace(line);
        if words.len() != 3 || words[0] != b"List" {
            bail!(DeserializeError::InvalidValue("Broken List tag".into(),));
        }
        let list_name = std::str::from_utf8(words[1])?;
        let _count: u32 = stoi(words[2])?;
        let mut result = Self {
            name: list_name.into(),
            width: 0,
            items: Vec::new(),
        };
        while deser.has_remaining() {
            // Peek the line, but don't take it yet.
            let peeked_line = deser.peek_line()?;
            let peeked_words = split_by_whitespace(peeked_line);
            if peeked_words.is_empty() {
                deser.take_line()?;
                continue;
            }
            let name = peeked_words[0];
            if name == b"EndInventoryList" || name == b"end" {
                deser.take_line()?;
                return Ok(result);
            } else if name == b"Width" {
                if peeked_words.len() < 2 {
                    bail!(DeserializeError::InvalidValue("Width value missing".into(),));
                }
                result.width = stoi(peeked_words[1])?;
                deser.take_line()?;
            } else if name == b"Item" {
                // ItemStack takes the line
                result
                    .items
                    .push(ItemStackUpdate::Item(ItemStack::deserialize(deser)?));
            } else if name == b"Empty" {
                result.items.push(ItemStackUpdate::Empty);
                deser.take_line()?;
            } else if name == b"Keep" {
                result.items.push(ItemStackUpdate::Keep);
                deser.take_line()?;
            } else {
                // Ignore unrecognized lines
                deser.take_line()?;
            }
        }
        bail!(DeserializeError::Eof(
            "InventoryList::deserialize(_)".into()
        ))
    }
}

// Custom deserialization, part of Inventory
#[derive(Debug, Clone, PartialEq)]
pub struct ItemStack {
    pub name: String,
    pub count: u16,
    pub wear: u16,
    pub metadata: ItemStackMetadata,
}

impl Serialize for ItemStack {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        // Item <name_json> [count] [wear] [metadata]
        ser.write_bytes(b"Item ")?;
        serialize_json_string_if_needed(value.name.as_bytes(), |chunk| ser.write_bytes(chunk))?;

        let mut parts = 1;
        if !value.metadata.string_vars.is_empty() {
            parts = 4;
        } else if value.wear != 0 {
            parts = 3;
        } else if value.count != 1 {
            parts = 2;
        }

        if parts >= 2 {
            ser.write_bytes(b" ")?;
            ser.write_bytes(value.count.to_string().as_bytes())?;
        }
        if parts >= 3 {
            ser.write_bytes(b" ")?;
            ser.write_bytes(value.wear.to_string().as_bytes())?;
        }
        if parts >= 4 {
            ser.write_bytes(b" ")?;
            ItemStackMetadata::serialize(&value.metadata, ser)?;
        }
        ser.write_bytes(b"\n")?;
        Ok(())
    }
}

impl Deserialize for ItemStack {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        // Item "name maybe escaped" [count] [wear] ["metadata escaped"]
        let line = deser.take_line()?;
        let err = DeserializeError::InvalidValue("Truncated Item line".into());
        let (first_word, line) = next_word(line).ok_or(err)?;
        if first_word != b"Item" {
            bail!(DeserializeError::InvalidValue("Invalid Item line".into(),));
        }
        let line = skip_whitespace(line);
        let (name, skip) = deserialize_json_string_if_needed(line)?;
        let line = skip_whitespace(&line[skip..]);

        let mut result = Self {
            name: std::str::from_utf8(&name)?.into(),
            count: 1,
            wear: 0,
            metadata: ItemStackMetadata {
                string_vars: Vec::new(),
            },
        };
        if let Some((count_str, line)) = next_word(line) {
            result.count = stoi(count_str)?;
            if let Some((wear_str, line)) = next_word(line) {
                result.wear = stoi(wear_str)?;
                let line = skip_whitespace(line);
                if !line.is_empty() {
                    let mut tmp_deser = Deserializer::new(deser.context(), line);
                    result.metadata = ItemStackMetadata::deserialize(&mut tmp_deser)?;
                }
            }
        }
        Ok(result)
    }
}

// Custom deserialization as json blob
#[derive(Debug, Clone, PartialEq)]
pub struct ItemStackMetadata {
    pub string_vars: Vec<(ByteString, ByteString)>,
}

const DESERIALIZE_START: &[u8; 1] = b"\x01";
const DESERIALIZE_KV_DELIM: &[u8; 1] = b"\x02";
const DESERIALIZE_PAIR_DELIM: &[u8; 1] = b"\x03";

impl Serialize for ItemStackMetadata {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        let mut buf: Vec<u8> = Vec::new();
        buf.extend(DESERIALIZE_START);
        for (key, val) in &value.string_vars {
            if !key.is_empty() || !val.is_empty() {
                buf.extend(key.as_bytes());
                buf.extend(DESERIALIZE_KV_DELIM);
                buf.extend(val.as_bytes());
                buf.extend(DESERIALIZE_PAIR_DELIM);
            }
        }
        serialize_json_string_if_needed(&buf, |chunk| ser.write_bytes(chunk))?;
        Ok(())
    }
}

impl Deserialize for ItemStackMetadata {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let (raw, count) = deserialize_json_string_if_needed(deser.peek_all())?;
        deser.take(count)?;
        let mut result = Self {
            string_vars: Vec::new(),
        };
        if raw.is_empty() {
            return Ok(result);
        }
        if raw[0] != DESERIALIZE_START[0] {
            bail!(DeserializeError::InvalidValue(
                "ItemStackMetadata bad start".into(),
            ));
        }
        let mut raw = &raw[1..];
        // This is odd, but matches the behavior of ItemStackMetadata::deSerialize
        while !raw.is_empty() {
            let kv_delim_pos = raw
                .iter()
                .position(|ch| *ch == DESERIALIZE_KV_DELIM[0])
                .unwrap_or(raw.len());
            let name = &raw[..kv_delim_pos];
            raw = &raw[kv_delim_pos..];
            if !raw.is_empty() {
                raw = &raw[1..];
            }
            let pair_delim_pos = raw
                .iter()
                .position(|ch| *ch == DESERIALIZE_PAIR_DELIM[0])
                .unwrap_or(raw.len());
            let var = &raw[..pair_delim_pos];
            raw = &raw[pair_delim_pos..];
            if !raw.is_empty() {
                raw = &raw[1..];
            }
            result.string_vars.push((name.into(), var.into()));
        }
        Ok(result)
    }
}

/// This is the send format used by `SendSpawnParticle`
/// See `ParticleParameters::serialize`
#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
#[expect(clippy::struct_excessive_bools, reason = "this is mandated by the API")]
pub struct ParticleParameters {
    pub pos: v3f,
    pub vel: v3f,
    pub acc: v3f,
    pub expiration_time: f32,
    pub size: f32,
    pub collision_detection: bool,
    #[wrap(LongString)]
    pub texture: String, // ServerParticleTexture.string
    pub vertical: bool,
    pub collision_removal: bool,
    pub animation: TileAnimationParams,
    pub glow: u8,
    pub object_collision: bool,
    // These are omitted in earlier protocol versions
    pub node_param0: Option<u16>,
    pub node_param2: Option<u8>,
    pub node_tile: Option<u8>,
    pub drag: Option<v3f>,
    pub jitter: Option<RangedParameter<v3f>>,
    pub bounce: Option<RangedParameter<f32>>,
}

#[derive(Debug, Default, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct RangedParameter<T: Serialize + Deserialize>
where
    T: Serialize<Input = T>,
    T: Deserialize<Output = T>,
{
    pub min: T,
    pub max: T,
    pub bias: f32,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct Lighting {
    pub shadow_intensity: f32,
    pub saturation: f32,
    pub exposure: AutoExposure,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct AutoExposure {
    pub luminance_min: f32,
    pub luminance_max: f32,
    pub exposure_correction: f32,
    pub speed_dark_bright: f32,
    pub speed_bright_dark: f32,
    pub center_weight_power: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HudSetParam {
    SetHotBarItemCount(s32),
    SetHotBarImage(String),
    SetHotBarSelectedImage(String),
}

impl Serialize for HudSetParam {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        #![allow(clippy::enum_glob_use, reason = "improves readability")]
        use HudSetParam::*;
        let param: u16 = match value {
            SetHotBarItemCount(_) => 1,
            SetHotBarImage(_) => 2,
            SetHotBarSelectedImage(_) => 3,
        };
        u16::serialize(&param, ser)?;
        match value {
            SetHotBarItemCount(value) => {
                // The value is wrapped in a a String16
                u16::serialize(&4, ser)?;
                s32::serialize(value, ser)?;
            }
            SetHotBarImage(value) => String::serialize(value, ser)?,
            SetHotBarSelectedImage(value) => String::serialize(value, ser)?,
        };
        Ok(())
    }
}

impl Deserialize for HudSetParam {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        #[allow(clippy::enum_glob_use, reason = "improves readability")]
        use HudSetParam::*;
        let param = u16::deserialize(deser)?;
        Ok(match param {
            1 => {
                let size = u16::deserialize(deser)?;
                if size != 4 {
                    bail!("Invalid size in SetHotBarItemCount: {}", size);
                }
                SetHotBarItemCount(s32::deserialize(deser)?)
            }
            2 => SetHotBarImage(String::deserialize(deser)?),
            3 => SetHotBarSelectedImage(String::deserialize(deser)?),
            _ => bail!("Invalid HudSetParam param: {}", param),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "// TODO rewrite using a crate for flags or bit-fields"
)]
pub struct HudFlags {
    pub hotbar_visible: bool,
    pub healthbar_visible: bool,
    pub crosshair_visible: bool,
    pub wielditem_visible: bool,
    pub breathbar_visible: bool,
    pub minimap_visible: bool,
    pub minimap_radar_visible: bool,
    pub basic_debug: bool,
    pub chat_visible: bool,
}

impl HudFlags {
    #[must_use]
    pub fn to_u32(&self) -> u32 {
        #![expect(clippy::identity_op, reason = "for symmetry")]
        let mut flags: u32 = 0;
        flags |= u32::from(self.hotbar_visible) << 0;
        flags |= u32::from(self.healthbar_visible) << 1;
        flags |= u32::from(self.crosshair_visible) << 2;
        flags |= u32::from(self.wielditem_visible) << 3;
        flags |= u32::from(self.breathbar_visible) << 4;
        flags |= u32::from(self.minimap_visible) << 5;
        flags |= u32::from(self.minimap_radar_visible) << 6;
        flags |= u32::from(self.basic_debug) << 7;
        flags |= u32::from(self.chat_visible) << 8;
        flags
    }

    #[must_use]
    pub fn from_u32(flags: u32) -> Self {
        Self {
            hotbar_visible: (flags & (1 << 0)) != 0,
            healthbar_visible: (flags & (1 << 1)) != 0,
            crosshair_visible: (flags & (1 << 2)) != 0,
            wielditem_visible: (flags & (1 << 3)) != 0,
            breathbar_visible: (flags & (1 << 4)) != 0,
            minimap_visible: (flags & (1 << 5)) != 0,
            minimap_radar_visible: (flags & (1 << 6)) != 0,
            basic_debug: (flags & (1 << 7)) != 0,
            chat_visible: (flags & (1 << 8)) != 0,
        }
    }
}

impl Serialize for HudFlags {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        let value = value.to_u32();
        u32::serialize(&value, ser)
    }
}

impl Deserialize for HudFlags {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let value = u32::deserialize(deser)?;
        if (value & !0b1_1111_1111) != 0 {
            bail!("Invalid HudFlags: {}", value);
        }
        Ok(HudFlags::from_u32(value))
    }
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub enum InteractAction {
    StartDigging,
    StopDigging,
    DiggingCompleted,
    Place,
    Use,
    Activate,
}

#[derive(Debug, Clone, PartialEq)]
#[expect(
    variant_size_differences,
    reason = "// TODO review whether this is actually a problem"
)]
pub enum PointedThing {
    Nothing,
    Node {
        under_surface: v3s16,
        above_surface: v3s16,
    },
    Object {
        object_id: u16,
    },
}

impl Serialize for PointedThing {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        // version, always 0
        u8::serialize(&0, ser)?;

        let typ: u8 = match value {
            PointedThing::Nothing => 0,
            PointedThing::Node { .. } => 1,
            PointedThing::Object { .. } => 2,
        };
        u8::serialize(&typ, ser)?;

        match value {
            PointedThing::Nothing => (),
            PointedThing::Node {
                under_surface,
                above_surface,
            } => {
                v3s16::serialize(under_surface, ser)?;
                v3s16::serialize(above_surface, ser)?;
            }
            PointedThing::Object { object_id } => {
                u16::serialize(object_id, ser)?;
            }
        }
        Ok(())
    }
}

impl Deserialize for PointedThing {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let ver = u8::deserialize(deser)?;
        if ver != 0 {
            bail!("Invalid PointedThing version: {}", ver);
        }
        let typ = u8::deserialize(deser)?;
        Ok(match typ {
            0 => PointedThing::Nothing,
            1 => PointedThing::Node {
                under_surface: v3s16::deserialize(deser)?,
                above_surface: v3s16::deserialize(deser)?,
            },
            2 => PointedThing::Object {
                object_id: u16::deserialize(deser)?,
            },
            _ => bail!("Invalid PointedThing type: {}", typ),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InventoryAction {
    Move {
        count: u16,
        from_inv: InventoryLocation,
        from_list: String,
        from_i: s16,
        to_inv: InventoryLocation,
        to_list: String,
        to_i: Option<s16>,
    },
    Craft {
        count: u16,
        craft_inv: InventoryLocation,
    },
    Drop {
        count: u16,
        from_inv: InventoryLocation,
        from_list: String,
        from_i: s16,
    },
}

impl Serialize for InventoryAction {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        match value {
            InventoryAction::Move {
                count,
                from_inv,
                from_list,
                from_i,
                to_inv,
                to_list,
                to_i,
            } => {
                if to_i.is_some() {
                    ser.write_bytes(b"Move ")?;
                } else {
                    ser.write_bytes(b"MoveSomewhere ")?;
                }
                ser.write_bytes(itos!(count))?;
                ser.write_bytes(b" ")?;
                InventoryLocation::serialize(from_inv, ser)?;
                ser.write_bytes(b" ")?;
                ser.write_bytes(from_list.as_bytes())?;
                ser.write_bytes(b" ")?;
                ser.write_bytes(itos!(from_i))?;
                ser.write_bytes(b" ")?;
                InventoryLocation::serialize(to_inv, ser)?;
                ser.write_bytes(b" ")?;
                ser.write_bytes(to_list.as_bytes())?;
                if let Some(to_i) = to_i {
                    ser.write_bytes(b" ")?;
                    ser.write_bytes(itos!(to_i))?;
                }
            }
            InventoryAction::Craft { count, craft_inv } => {
                ser.write_bytes(b"Craft ")?;
                ser.write_bytes(itos!(count))?;
                ser.write_bytes(b" ")?;
                InventoryLocation::serialize(craft_inv, ser)?;
                // This extra space is present in Luanti
                ser.write_bytes(b" ")?;
            }
            InventoryAction::Drop {
                count,
                from_inv,
                from_list,
                from_i,
            } => {
                ser.write_bytes(b"Drop ")?;
                ser.write_bytes(itos!(count))?;
                ser.write_bytes(b" ")?;
                InventoryLocation::serialize(from_inv, ser)?;
                ser.write_bytes(b" ")?;
                ser.write_bytes(from_list.as_bytes())?;
                ser.write_bytes(b" ")?;
                ser.write_bytes(itos!(from_i))?;
            }
        }
        Ok(())
    }
}

impl Deserialize for InventoryAction {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let word = deser.take_word(true);
        if word == b"Move" || word == b"MoveSomewhere" {
            Ok(InventoryAction::Move {
                count: stoi(deser.take_word(true))?,
                from_inv: InventoryLocation::deserialize(deser)?,
                from_list: std::str::from_utf8(deser.take_word(true))?.to_owned(),
                from_i: stoi(deser.take_word(true))?,
                to_inv: InventoryLocation::deserialize(deser)?,
                to_list: std::str::from_utf8(deser.take_word(true))?.to_owned(),
                #[expect(clippy::if_then_some_else_none, reason = "`?`-operator prohibits this")]
                to_i: if word == b"Move" {
                    Some(stoi(deser.take_word(true))?)
                } else {
                    None
                },
            })
        } else if word == b"Drop" {
            Ok(InventoryAction::Drop {
                count: stoi(deser.take_word(true))?,
                from_inv: InventoryLocation::deserialize(deser)?,
                from_list: std::str::from_utf8(deser.take_word(true))?.to_owned(),
                from_i: stoi(deser.take_word(true))?,
            })
        } else if word == b"Craft" {
            Ok(InventoryAction::Craft {
                count: stoi(deser.take_word(true))?,
                craft_inv: InventoryLocation::deserialize(deser)?,
            })
        } else {
            bail!("Invalid InventoryAction kind");
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InventoryLocation {
    Undefined,
    CurrentPlayer,
    Player { name: String },
    NodeMeta { pos: v3s16 },
    Detached { name: String },
}

impl Serialize for InventoryLocation {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        match value {
            InventoryLocation::Undefined => ser.write_bytes(b"undefined")?,
            InventoryLocation::CurrentPlayer => ser.write_bytes(b"current_player")?,
            InventoryLocation::Player { name } => {
                ser.write_bytes(b"player:")?;
                ser.write_bytes(name.as_bytes())?;
            }
            InventoryLocation::NodeMeta { pos } => {
                ser.write_bytes(format!("nodemeta:{},{},{}", pos.x, pos.y, pos.z).as_bytes())?;
            }
            InventoryLocation::Detached { name } => {
                ser.write_bytes(b"detached:")?;
                ser.write_bytes(name.as_bytes())?;
            }
        }
        Ok(())
    }
}

impl Deserialize for InventoryLocation {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let word = deser.take_word(true);
        if word == b"undefined" {
            Ok(InventoryLocation::Undefined)
        } else if word == b"current_player" {
            Ok(InventoryLocation::CurrentPlayer)
        } else if word.starts_with(b"player:") {
            Ok(InventoryLocation::Player {
                name: std::str::from_utf8(&word[7..])?.into(),
            })
        } else if word.starts_with(b"nodemeta:") {
            // TODO replace with strip_prefix
            let coords: Vec<&[u8]> = word[9..].split(|&ch| ch == b',').collect();
            if coords.len() != 3 {
                bail!("Corrupted nodemeta InventoryLocation");
            }
            let mut xyz = [0_i16; 3];
            for (i, &n) in coords.iter().enumerate() {
                xyz[i] = stoi(n)?;
            }
            let pos = v3s16::new(xyz[0], xyz[1], xyz[2]);
            Ok(InventoryLocation::NodeMeta { pos })
        } else if word.starts_with(b"detached:") {
            Ok(InventoryLocation::Detached {
                name: std::str::from_utf8(&word[9..])?.into(),
            })
        } else {
            Err(anyhow!("Unknown InventoryLocation: {:?}", word))
        }
    }
}
