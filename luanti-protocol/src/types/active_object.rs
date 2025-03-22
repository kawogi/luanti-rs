use super::{Array8, Array16, Pair, SColor, Wrapped32, aabb3f, s8, s16, v2f, v2s16, v3f};
use crate::wire::{
    deser::{Deserialize, DeserializeResult, Deserializer},
    ser::{Serialize, SerializeResult, Serializer},
};
use anyhow::bail;
use luanti_protocol_derive::{LuantiDeserialize, LuantiSerialize};

/// This corresponds to `GenericCAO::Initialize` in Luanti
#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct GenericInitData {
    pub version: u8,
    pub name: String,
    pub is_player: bool,
    pub id: u16,
    pub position: v3f,
    pub rotation: v3f,
    pub hp: u16,
    #[wrap(Array8<Wrapped32<ActiveObjectCommand>>)]
    pub messages: Vec<ActiveObjectCommand>,
}

// TODO(paradust): Handle this in derive macros
#[derive(Debug, Clone, PartialEq)]
#[expect(clippy::large_enum_variant, reason = "consider `Box`ing variants")]
pub enum ActiveObjectCommand {
    SetProperties(AOCSetProperties),
    UpdatePosition(AOCUpdatePosition),
    SetTextureMod(AOCSetTextureMod),
    SetSprite(AOCSetSprite),
    SetPhysicsOverride(AOCSetPhysicsOverride),
    SetAnimation(AOCSetAnimation),
    SetAnimationSpeed(AOCSetAnimationSpeed),
    SetBonePosition(AOCSetBonePosition),
    AttachTo(AOCAttachTo),
    Punched(AOCPunched),
    UpdateArmorGroups(AOCUpdateArmorGroups),
    SpawnInfant(AOCSpawnInfant),
    Obsolete1(AOCObsolete1),
}

const AO_CMD_SET_PROPERTIES: u8 = 0;
const AO_CMD_UPDATE_POSITION: u8 = 1;
const AO_CMD_SET_TEXTURE_MOD: u8 = 2;
const AO_CMD_SET_SPRITE: u8 = 3;
const AO_CMD_PUNCHED: u8 = 4;
const AO_CMD_UPDATE_ARMOR_GROUPS: u8 = 5;
const AO_CMD_SET_ANIMATION: u8 = 6;
const AO_CMD_SET_BONE_POSITION: u8 = 7;
const AO_CMD_ATTACH_TO: u8 = 8;
const AO_CMD_SET_PHYSICS_OVERRIDE: u8 = 9;
const AO_CMD_OBSOLETE1: u8 = 10;
const AO_CMD_SPAWN_INFANT: u8 = 11;
const AO_CMD_SET_ANIMATION_SPEED: u8 = 12;

impl ActiveObjectCommand {
    fn get_command_prefix(&self) -> u8 {
        match self {
            ActiveObjectCommand::SetProperties(_) => AO_CMD_SET_PROPERTIES,
            ActiveObjectCommand::UpdatePosition(_) => AO_CMD_UPDATE_POSITION,
            ActiveObjectCommand::SetTextureMod(_) => AO_CMD_SET_TEXTURE_MOD,
            ActiveObjectCommand::SetSprite(_) => AO_CMD_SET_SPRITE,
            ActiveObjectCommand::SetPhysicsOverride(_) => AO_CMD_SET_PHYSICS_OVERRIDE,
            ActiveObjectCommand::SetAnimation(_) => AO_CMD_SET_ANIMATION,
            ActiveObjectCommand::SetAnimationSpeed(_) => AO_CMD_SET_ANIMATION_SPEED,
            ActiveObjectCommand::SetBonePosition(_) => AO_CMD_SET_BONE_POSITION,
            ActiveObjectCommand::AttachTo(_) => AO_CMD_ATTACH_TO,
            ActiveObjectCommand::Punched(_) => AO_CMD_PUNCHED,
            ActiveObjectCommand::UpdateArmorGroups(_) => AO_CMD_UPDATE_ARMOR_GROUPS,
            ActiveObjectCommand::SpawnInfant(_) => AO_CMD_SPAWN_INFANT,
            ActiveObjectCommand::Obsolete1(_) => AO_CMD_OBSOLETE1,
        }
    }
}

impl Serialize for ActiveObjectCommand {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        u8::serialize(&value.get_command_prefix(), ser)?;
        match value {
            ActiveObjectCommand::SetProperties(command) => {
                AOCSetProperties::serialize(command, ser)?;
            }
            ActiveObjectCommand::UpdatePosition(command) => {
                AOCUpdatePosition::serialize(command, ser)?;
            }
            ActiveObjectCommand::SetTextureMod(command) => {
                AOCSetTextureMod::serialize(command, ser)?;
            }
            ActiveObjectCommand::SetSprite(command) => AOCSetSprite::serialize(command, ser)?,
            ActiveObjectCommand::SetPhysicsOverride(command) => {
                AOCSetPhysicsOverride::serialize(command, ser)?;
            }
            ActiveObjectCommand::SetAnimation(command) => AOCSetAnimation::serialize(command, ser)?,
            ActiveObjectCommand::SetAnimationSpeed(command) => {
                AOCSetAnimationSpeed::serialize(command, ser)?;
            }
            ActiveObjectCommand::SetBonePosition(command) => {
                AOCSetBonePosition::serialize(command, ser)?;
            }
            ActiveObjectCommand::AttachTo(command) => AOCAttachTo::serialize(command, ser)?,
            ActiveObjectCommand::Punched(command) => AOCPunched::serialize(command, ser)?,
            ActiveObjectCommand::UpdateArmorGroups(command) => {
                AOCUpdateArmorGroups::serialize(command, ser)?;
            }
            ActiveObjectCommand::SpawnInfant(command) => AOCSpawnInfant::serialize(command, ser)?,
            ActiveObjectCommand::Obsolete1(command) => AOCObsolete1::serialize(command, ser)?,
        }
        Ok(())
    }
}

impl Deserialize for ActiveObjectCommand {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        #[allow(
            clippy::enum_glob_use,
            reason = "this improves readability and is very local"
        )]
        use ActiveObjectCommand::*;
        let cmd = u8::deserialize(deser)?;
        Ok(match cmd {
            AO_CMD_SET_PROPERTIES => SetProperties(AOCSetProperties::deserialize(deser)?),
            AO_CMD_UPDATE_POSITION => UpdatePosition(AOCUpdatePosition::deserialize(deser)?),
            AO_CMD_SET_TEXTURE_MOD => SetTextureMod(AOCSetTextureMod::deserialize(deser)?),
            AO_CMD_SET_SPRITE => SetSprite(AOCSetSprite::deserialize(deser)?),
            AO_CMD_PUNCHED => Punched(AOCPunched::deserialize(deser)?),
            AO_CMD_UPDATE_ARMOR_GROUPS => {
                UpdateArmorGroups(AOCUpdateArmorGroups::deserialize(deser)?)
            }
            AO_CMD_SET_ANIMATION => SetAnimation(AOCSetAnimation::deserialize(deser)?),
            AO_CMD_SET_BONE_POSITION => SetBonePosition(AOCSetBonePosition::deserialize(deser)?),
            AO_CMD_ATTACH_TO => AttachTo(AOCAttachTo::deserialize(deser)?),
            AO_CMD_SET_PHYSICS_OVERRIDE => {
                SetPhysicsOverride(AOCSetPhysicsOverride::deserialize(deser)?)
            }
            AO_CMD_OBSOLETE1 => Obsolete1(AOCObsolete1::deserialize(deser)?),
            AO_CMD_SPAWN_INFANT => SpawnInfant(AOCSpawnInfant::deserialize(deser)?),
            AO_CMD_SET_ANIMATION_SPEED => {
                SetAnimationSpeed(AOCSetAnimationSpeed::deserialize(deser)?)
            }
            _ => bail!("ActiveObjectCommand: Invalid cmd={}", cmd),
        })
    }
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct AOCSetProperties {
    pub newprops: ObjectProperties,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
#[expect(clippy::struct_excessive_bools, reason = "this is mandated by the API")]
pub struct ObjectProperties {
    pub version: u8, // must be 4
    pub hp_max: u16,
    pub physical: bool,
    pub _unused: u32,
    pub collision_box: aabb3f,
    pub selection_box: aabb3f,
    pub pointable: bool,
    pub visual: String,
    pub visual_size: v3f,
    #[wrap(Array16<String>)]
    pub textures: Vec<String>,
    pub spritediv: v2s16,
    pub initial_sprite_basepos: v2s16,
    pub is_visible: bool,
    pub makes_footstep_sound: bool,
    pub automatic_rotate: f32,
    pub mesh: String,
    #[wrap(Array16<SColor>)]
    pub colors: Vec<SColor>,
    pub collide_with_objects: bool,
    pub stepheight: f32,
    pub automatic_face_movement_dir: bool,
    pub automatic_face_movement_dir_offset: f32,
    pub backface_culling: bool,
    pub nametag: String,
    pub nametag_color: SColor,
    pub automatic_face_movement_max_rotation_per_sec: f32,
    pub infotext: String,
    pub wield_item: String,
    pub glow: s8,
    pub breath_max: u16,
    pub eye_height: f32,
    pub zoom_fov: f32,
    pub use_texture_alpha: bool,
    pub damage_texture_modifier: Option<String>,
    pub shaded: Option<bool>,
    pub show_on_minimap: Option<bool>,
    pub nametag_bgcolor: Option<SColor>,
    pub rotate_selectionbox: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct AOCUpdatePosition {
    pub position: v3f,
    pub velocity: v3f,
    pub acceleration: v3f,
    pub rotation: v3f,
    pub do_interpolate: bool,
    pub is_end_position: bool,
    pub update_interval: f32,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct AOCSetTextureMod {
    pub modifier: String,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct AOCSetSprite {
    pub base_pos: v2s16,
    pub anum_num_frames: u16,
    pub anim_frame_length: f32,
    pub select_horiz_by_yawpitch: bool,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct AOCSetPhysicsOverride {
    pub override_speed: f32,
    pub override_jump: f32,
    pub override_gravity: f32,
    pub not_sneak: bool,
    pub not_sneak_glitch: bool,
    pub not_new_move: bool,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct AOCSetAnimation {
    pub range: v2f, // this is always casted to v2s32 by Luanti for some reason
    pub speed: f32,
    pub blend: f32,
    pub no_loop: bool,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct AOCSetAnimationSpeed {
    pub speed: f32,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct AOCSetBonePosition {
    pub bone: String,
    pub position: v3f,
    pub rotation: v3f,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct AOCAttachTo {
    pub parent_id: s16,
    pub bone: String,
    pub position: v3f,
    pub rotation: v3f,
    pub force_visible: bool,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct AOCPunched {
    pub hp: u16,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct AOCUpdateArmorGroups {
    // name -> rating
    #[wrap(Array16<Pair<String, s16>>)]
    pub ratings: Vec<(String, s16)>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct AOCSpawnInfant {
    pub child_id: u16,
    pub typ: u8,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct AOCObsolete1;
