use anyhow::bail;
use luanti_protocol_derive::{LuantiDeserialize, LuantiSerialize};

use crate::wire::{
    deser::{Deserialize, DeserializeError, DeserializeResult, Deserializer},
    ser::{Serialize, SerializeResult, Serializer},
};

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
