use super::CommandProperties;
use crate::wire::audit::audit_command;
use crate::wire::channel_id::ChannelId;
use crate::wire::deser::Deserialize;
use crate::wire::deser::DeserializeError;
use crate::wire::deser::DeserializeResult;
use crate::wire::deser::Deserializer;
use crate::wire::ser::Serialize;
use crate::wire::ser::SerializeResult;
use crate::wire::ser::Serializer;
#[allow(clippy::wildcard_imports, reason = "greatly simplifies macros")]
use crate::wire::types::*;
use anyhow::bail;
use luanti_protocol_derive::LuantiDeserialize;
use luanti_protocol_derive::LuantiSerialize;
use std::ops::Deref;

define_protocol!(41, 0x4f457403, ToClient, ToClientCommand => {
    // CommandName, CommandType, Direction, Channel, Reliable
    Hello, 0x02, Default, true => HelloSpec,
    AuthAccept, 0x03, Default, true => AuthAcceptSpec,
    AcceptSudoMode, 0x04, Default, true => AcceptSudoModeSpec,
    DenySudoMode, 0x05, Default, true => DenySudoModeSpec,
    AccessDenied, 0x0A, Default, true => AccessDeniedSpec,
    Blockdata, 0x20, Response, true => BlockdataSpec,
    Addnode, 0x21, Default, true => AddnodeSpec,
    Removenode, 0x22, Default, true => RemovenodeSpec,
    Inventory, 0x27, Default, true => InventorySpec,
    TimeOfDay, 0x29, Default, true => TimeOfDaySpec,
    CsmRestrictionFlags, 0x2A, Default, true => CsmRestrictionFlagsSpec,
    PlayerSpeed, 0x2B, Default, true => PlayerSpeedSpec,
    MediaPush, 0x2C, Default, true => MediaPushSpec,
    TCChatMessage, 0x2F, Default, true => TCChatMessageSpec,
    ActiveObjectRemoveAdd, 0x31, Default, true => ActiveObjectRemoveAddSpec,
    ActiveObjectMessages, 0x32, Default, true => ActiveObjectMessagesSpec,
    Hp, 0x33, Default, true => HpSpec,
    MovePlayer, 0x34, Default, true => MovePlayerSpec,
    AccessDeniedLegacy, 0x35, Default, true => AccessDeniedLegacySpec,
    Fov, 0x36, Default, true => FovSpec,
    Deathscreen, 0x37, Default, true => DeathscreenSpec,
    Media, 0x38, Response, true => MediaSpec,
    Nodedef, 0x3a, Default, true => NodedefSpec,
    AnnounceMedia, 0x3c, Default, true => AnnounceMediaSpec,
    Itemdef, 0x3d, Default, true => ItemdefSpec,
    PlaySound, 0x3f, Default, true => PlaySoundSpec,
    StopSound, 0x40, Default, true => StopSoundSpec,
    Privileges, 0x41, Default, true => PrivilegesSpec,
    InventoryFormspec, 0x42, Default, true => InventoryFormspecSpec,
    DetachedInventory, 0x43, Default, true => DetachedInventorySpec,
    ShowFormspec, 0x44, Default, true => ShowFormspecSpec,
    Movement, 0x45, Default, true => MovementSpec,
    SpawnParticle, 0x46, Default, true => SpawnParticleSpec,
    AddParticlespawner, 0x47, Default, true => AddParticlespawnerSpec,
    Hudadd, 0x49, Init, true => HudaddSpec,
    Hudrm, 0x4a, Init, true => HudrmSpec,
    Hudchange, 0x4b, Init, true => HudchangeSpec,
    HudSetFlags, 0x4c, Init, true => HudSetFlagsSpec,
    HudSetParam, 0x4d, Init, true => HudSetParamSpec,
    Breath, 0x4e, Default, true => BreathSpec,
    SetSky, 0x4f, Default, true => SetSkySpec,
    OverrideDayNightRatio, 0x50, Default, true => OverrideDayNightRatioSpec,
    LocalPlayerAnimations, 0x51, Default, true => LocalPlayerAnimationsSpec,
    EyeOffset, 0x52, Default, true => EyeOffsetSpec,
    DeleteParticlespawner, 0x53, Default, true => DeleteParticlespawnerSpec,
    CloudParams, 0x54, Default, true => CloudParamsSpec,
    FadeSound, 0x55, Default, true => FadeSoundSpec,
    UpdatePlayerList, 0x56, Default, true => UpdatePlayerListSpec,
    TCModchannelMsg, 0x57, Default, true => TCModchannelMsgSpec,
    ModchannelSignal, 0x58, Default, true => ModchannelSignalSpec,
    NodemetaChanged, 0x59, Default, true => NodemetaChangedSpec,
    SetSun, 0x5a, Default, true => SetSunSpec,
    SetMoon, 0x5b, Default, true => SetMoonSpec,
    SetStars, 0x5c, Default, true => SetStarsSpec,
    SrpBytesSB, 0x60, Default, true => SrpBytesSBSpec,
    FormspecPrepend, 0x61, Default, true => FormspecPrependSpec,
    MinimapModes, 0x62, Default, true => MinimapModesSpec,
    SetLighting, 0x63, Default, true => SetLightingSpec
});

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct HelloSpec {
    pub serialization_ver: u8,
    pub compression_mode: u16,
    pub proto_ver: u16,
    pub auth_mechs: AuthMechsBitset,
    pub username_legacy: String,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct AuthAcceptSpec {
    pub player_pos: v3f,
    pub map_seed: u64,
    pub recommended_send_interval: f32,
    pub sudo_auth_methods: u32,
}

#[derive(Debug, Clone, PartialEq, Default, LuantiSerialize, LuantiDeserialize)]
pub struct AcceptSudoModeSpec;

#[derive(Debug, Clone, PartialEq, Default, LuantiSerialize, LuantiDeserialize)]
pub struct DenySudoModeSpec;

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct AccessDeniedSpec {
    pub code: AccessDeniedCode,
    pub reason: String,
    pub reconnect: bool,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct BlockdataSpec {
    pub pos: v3s16,
    pub block: MapBlock,
    pub network_specific_version: u8,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct AddnodeSpec {
    pub pos: v3s16,
    pub node: MapNode,
    pub keep_metadata: bool,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct RemovenodeSpec {
    pub pos: v3s16,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct InventorySpec {
    pub inventory: Inventory,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct TimeOfDaySpec {
    pub time_of_day: u16,
    pub time_speed: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct CsmRestrictionFlagsSpec {
    pub csm_restriction_flags: u64,
    pub csm_restriction_noderange: u32,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct PlayerSpeedSpec {
    pub added_vel: v3f,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct MediaPushSpec {
    pub raw_hash: String,
    pub filename: String,
    pub cached: bool,
    pub token: u32,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct TCChatMessageSpec {
    pub version: u8,
    pub message_type: u8,
    #[wrap(WString)]
    pub sender: String,
    #[wrap(WString)]
    pub message: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct ActiveObjectRemoveAddSpec {
    #[wrap(Array16<u16>)]
    pub removed_object_ids: Vec<u16>,
    #[wrap(Array16<AddedObject>)]
    pub added_objects: Vec<AddedObject>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct ActiveObjectMessagesSpec {
    #[wrap(Array0<ActiveObjectMessage>)]
    pub objects: Vec<ActiveObjectMessage>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct HpSpec {
    pub hp: u16,
    pub damage_effect: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct MovePlayerSpec {
    pub pos: v3f,
    pub pitch: f32,
    pub yaw: f32,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct AccessDeniedLegacySpec {
    #[wrap(WString)]
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct FovSpec {
    pub fov: f32,
    pub is_multiplier: bool,
    pub transition_time: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct DeathscreenSpec {
    pub set_camera_point_target: bool,
    pub camera_point_target: v3f,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct MediaSpec {
    pub num_bunches: u16,
    pub bunch_index: u16,
    #[wrap(Array32<MediaFileData>)]
    pub files: Vec<MediaFileData>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct NodedefSpec {
    #[wrap(ZLibCompressed<NodeDefManager>)]
    pub node_def: NodeDefManager,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct AnnounceMediaSpec {
    #[wrap(Array16<MediaAnnouncement>)]
    pub files: Vec<MediaAnnouncement>,
    pub remote_servers: String,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct ItemdefSpec {
    #[wrap(ZLibCompressed<ItemdefList>)]
    pub item_def: ItemdefList,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct PlaySoundSpec {
    pub server_id: s32,
    pub spec_name: String,
    pub spec_gain: f32,
    pub typ: u8,
    pub pos: v3f,
    pub object_id: u16,
    pub spec_loop: bool,
    pub spec_fade: Option<f32>,
    pub spec_pitch: Option<f32>,
    pub ephemeral: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct StopSoundSpec {
    pub server_id: s32,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct PrivilegesSpec {
    #[wrap(Array16<String>)]
    pub privileges: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct InventoryFormspecSpec {
    #[wrap(LongString)]
    pub formspec: String,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct DetachedInventorySpec {
    pub name: String,
    pub keep_inv: bool,
    pub ignore: Option<u16>,
    pub contents: Option<Inventory>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct ShowFormspecSpec {
    #[wrap(LongString)]
    pub form_spec: String,
    pub form_name: String,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct MovementSpec {
    pub acceleration_default: f32,
    pub acceleration_air: f32,
    pub acceleration_fast: f32,
    pub speed_walk: f32,
    pub speed_crouch: f32,
    pub speed_fast: f32,
    pub speed_climb: f32,
    pub speed_jump: f32,
    pub liquid_fluidity: f32,
    pub liquid_fluidity_smooth: f32,
    pub liquid_sink: f32,
    pub gravity: f32,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct SpawnParticleSpec {
    pub data: ParticleParameters,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct AddParticlespawnerSpec {
    pub legacy: AddParticleSpawnerLegacy,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct HudaddSpec {
    pub server_id: u32,
    pub typ: u8,
    pub pos: v2f,
    pub name: String,
    pub scale: v2f,
    pub text: String,
    pub number: u32,
    pub item: u32,
    pub dir: u32,
    pub align: v2f,
    pub offset: v2f,
    pub world_pos: Option<v3f>,
    pub size: Option<v2s32>,
    pub z_index: Option<s16>,
    pub text2: Option<String>,
    pub style: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct HudrmSpec {
    pub server_id: u32,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct HudchangeSpec {
    pub server_id: u32,
    pub stat: HudStat,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct HudSetFlagsSpec {
    pub flags: HudFlags,
    pub mask: HudFlags,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct HudSetParamSpec {
    pub value: HudSetParam,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct BreathSpec {
    pub breath: u16,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct SetSkySpec {
    pub params: SkyboxParams,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct OverrideDayNightRatioSpec {
    pub do_override: bool,
    pub day_night_ratio: u16,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct LocalPlayerAnimationsSpec {
    pub idle: v2s32,
    pub walk: v2s32,
    pub dig: v2s32,
    pub walk_dig: v2s32,
    pub frame_speed: f32,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct EyeOffsetSpec {
    pub eye_offset_first: v3f,
    pub eye_offset_third: v3f,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct DeleteParticlespawnerSpec {
    pub server_id: u32,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct CloudParamsSpec {
    pub density: f32,
    pub color_bright: SColor,
    pub color_ambient: SColor,
    pub height: f32,
    pub thickness: f32,
    pub speed: v2f,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct FadeSoundSpec {
    pub sound_id: s32,
    pub step: f32,
    pub gain: f32,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct UpdatePlayerListSpec {
    pub typ: u8,
    #[wrap(Array16<String>)]
    pub players: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct TCModchannelMsgSpec {
    pub channel_name: String,
    pub sender: String,
    pub channel_msg: String,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct ModchannelSignalSpec {
    pub signal_tmp: u8,
    pub channel: String,
    pub state: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct NodemetaChangedSpec {
    #[wrap(ZLibCompressed<AbsNodeMetadataList>)]
    pub list: AbsNodeMetadataList,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct SetSunSpec {
    pub sun: SunParams,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct SetMoonSpec {
    pub moon: MoonParams,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct SetStarsSpec {
    pub stars: StarParams,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct SrpBytesSBSpec {
    #[wrap(BinaryData16)]
    pub s: Vec<u8>,
    #[wrap(BinaryData16)]
    pub b: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct FormspecPrependSpec {
    pub formspec_prepend: String,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct MinimapModesSpec {
    pub modes: MinimapModeList,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct SetLightingSpec {
    pub lighting: Lighting,
}
