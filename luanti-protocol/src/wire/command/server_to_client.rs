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
    Hello, 0x02, Default, true => HelloSpec {
        serialization_ver: u8,
        compression_mode: u16,
        proto_ver: u16,
        auth_mechs: AuthMechsBitset,
        username_legacy: String
    },

    AuthAccept, 0x03, Default, true => AuthAcceptSpec {
        player_pos: v3f,
        map_seed: u64,
        recommended_send_interval: f32,
        sudo_auth_methods: u32
    },

    AcceptSudoMode, 0x04, Default, true => AcceptSudoModeSpec {
        // No fields
    },

    DenySudoMode, 0x05, Default, true => DenySudoModeSpec {
        // No fields
    },

    AccessDenied, 0x0A, Default, true => AccessDeniedSpec {
        code: AccessDeniedCode,
        reason: String,
        reconnect: bool
    },

    Blockdata, 0x20, Response, true => BlockdataSpec {
        pos: v3s16,
        block: MapBlock,
        network_specific_version: u8
    },
    Addnode, 0x21, Default, true => AddnodeSpec {
        pos: v3s16,
        node: MapNode,
        keep_metadata: bool
    },

    Removenode, 0x22, Default, true => RemovenodeSpec {
        pos: v3s16
    },

    Inventory, 0x27, Default, true => InventorySpec {
        inventory: Inventory
    },

    TimeOfDay, 0x29, Default, true => TimeOfDaySpec {
        time_of_day: u16,
        time_speed: Option<f32>
    },

    CsmRestrictionFlags, 0x2A, Default, true => CsmRestrictionFlagsSpec {
        csm_restriction_flags: u64,
        csm_restriction_noderange: u32
    },

    PlayerSpeed, 0x2B, Default, true => PlayerSpeedSpec {
        added_vel: v3f
    },

    MediaPush, 0x2C, Default, true => MediaPushSpec {
        raw_hash: String,
        filename: String,
        cached: bool,
        token: u32
    },

    TCChatMessage, 0x2F, Default, true => TCChatMessageSpec {
        version: u8,
        message_type: u8,
        sender: String [wrap(WString)],
        message: String [wrap(WString)],
        timestamp: u64
    },

    ActiveObjectRemoveAdd, 0x31, Default, true => ActiveObjectRemoveAddSpec {
        removed_object_ids: Vec<u16> [wrap(Array16<u16>)],
        added_objects: Vec<AddedObject> [wrap(Array16<AddedObject>)]
    },

    ActiveObjectMessages, 0x32, Default, true => ActiveObjectMessagesSpec {
        objects: Vec<ActiveObjectMessage> [wrap(Array0<ActiveObjectMessage>)]
    },

    Hp, 0x33, Default, true => HpSpec {
        hp: u16,
        damage_effect: Option<bool>
    },

    MovePlayer, 0x34, Default, true => MovePlayerSpec {
        pos: v3f,
        pitch: f32,
        yaw: f32
    },

    AccessDeniedLegacy, 0x35, Default, true => AccessDeniedLegacySpec {
        reason: String [wrap(WString)]
    },

    Fov, 0x36, Default, true => FovSpec {
        fov: f32,
        is_multiplier: bool,
        transition_time: Option<f32>
    },

    Deathscreen, 0x37, Default, true => DeathscreenSpec {
        set_camera_point_target: bool,
        camera_point_target: v3f
    },

    Media, 0x38, Response, true => MediaSpec {
        num_bunches: u16,
        bunch_index: u16,
        files: Vec<MediaFileData> [wrap(Array32<MediaFileData>)]
    },

    Nodedef, 0x3a, Default, true => NodedefSpec {
        node_def: NodeDefManager [wrap(ZLibCompressed<NodeDefManager>)]
    },

    AnnounceMedia, 0x3c, Default, true => AnnounceMediaSpec {
        files: Vec<MediaAnnouncement> [wrap(Array16<MediaAnnouncement>)],
        remote_servers: String
    },

    Itemdef, 0x3d, Default, true => ItemdefSpec {
        item_def: ItemdefList [wrap(ZLibCompressed<ItemdefList>)]
    },

    PlaySound, 0x3f, Default, true => PlaySoundSpec {
        server_id: s32,
        spec_name: String,
        spec_gain: f32,
        typ: u8, // 0=local, 1=positional, 2=object
        pos: v3f,
        object_id: u16,
        spec_loop: bool,
        spec_fade: Option<f32>,
        spec_pitch: Option<f32>,
        ephemeral: Option<bool>
    },

    StopSound, 0x40, Default, true => StopSoundSpec {
        server_id: s32
    },

    Privileges, 0x41, Default, true => PrivilegesSpec {
        privileges: Vec<String> [wrap(Array16<String>)]
    },

    InventoryFormspec, 0x42, Default, true => InventoryFormspecSpec {
        formspec: String [wrap(LongString)]
    },

    DetachedInventory, 0x43, Default, true => DetachedInventorySpec {
        name: String,
        keep_inv: bool,
        // These are present if keep_inv is true.
        ignore: Option<u16>,
        contents: Option<Inventory>
    },

    ShowFormspec, 0x44, Default, true => ShowFormspecSpec {
        form_spec: String [wrap(LongString)],
        form_name: String
    },

    Movement, 0x45, Default, true => MovementSpec {
        acceleration_default: f32,
        acceleration_air: f32,
        acceleration_fast: f32,
        speed_walk: f32,
        speed_crouch: f32,
        speed_fast: f32,
        speed_climb: f32,
        speed_jump: f32,
        liquid_fluidity: f32,
        liquid_fluidity_smooth: f32,
        liquid_sink: f32,
        gravity: f32
    },

    SpawnParticle, 0x46, Default, true => SpawnParticleSpec {
        data: ParticleParameters
    },

    AddParticlespawner, 0x47, Default, true => AddParticlespawnerSpec {
        legacy: AddParticleSpawnerLegacy
    },

    Hudadd, 0x49, Init, true => HudaddSpec {
        server_id: u32,
        typ: u8,
        pos: v2f,
        name: String,
        scale: v2f,
        text: String,
        number: u32,
        item: u32,
        dir: u32,
        align: v2f,
        offset: v2f,
        world_pos: Option<v3f>,
        size: Option<v2s32>,
        z_index: Option<s16>,
        text2: Option<String>,
        style: Option<u32>
    },

    Hudrm, 0x4a, Init, true => HudrmSpec {
        server_id: u32
    },

    Hudchange, 0x4b, Init, true => HudchangeSpec {
        server_id: u32,
        stat: HudStat
    },

    HudSetFlags, 0x4c, Init, true => HudSetFlagsSpec {
        flags: HudFlags, // flags added
        mask: HudFlags   // flags possibly removed
    },

    HudSetParam, 0x4d, Init, true => HudSetParamSpec {
        value: HudSetParam
    },

    Breath, 0x4e, Default, true => BreathSpec {
        breath: u16
    },

    SetSky, 0x4f, Default, true => SetSkySpec {
        params: SkyboxParams
    },

    OverrideDayNightRatio, 0x50, Default, true => OverrideDayNightRatioSpec {
        do_override: bool,
        day_night_ratio: u16
    },

    LocalPlayerAnimations, 0x51, Default, true => LocalPlayerAnimationsSpec {
        idle: v2s32,
        walk: v2s32,
        dig: v2s32,
        walk_dig: v2s32,
        frame_speed: f32
    },

    EyeOffset, 0x52, Default, true => EyeOffsetSpec {
        eye_offset_first: v3f,
        eye_offset_third: v3f
    },

    DeleteParticlespawner, 0x53, Default, true => DeleteParticlespawnerSpec {
        server_id: u32
    },

    CloudParams, 0x54, Default, true => CloudParamsSpec {
        density: f32,
        color_bright: SColor,
        color_ambient: SColor,
        height: f32,
        thickness: f32,
        speed: v2f
    },

    FadeSound, 0x55, Default, true => FadeSoundSpec {
        sound_id: s32,
        step: f32,
        gain: f32
    },

    UpdatePlayerList, 0x56, Default, true => UpdatePlayerListSpec {
        typ: u8,
        players: Vec<String> [wrap(Array16<String>)]
    },

    TCModchannelMsg, 0x57, Default, true => TCModchannelMsgSpec {
        channel_name: String,
        sender: String,
        channel_msg: String
    },

    ModchannelSignal, 0x58, Default, true => ModchannelSignalSpec {
        signal_tmp: u8,
        channel: String,
        // signal == MODCHANNEL_SIGNAL_SET_STATE
        state: Option<u8>
    },

    NodemetaChanged, 0x59, Default, true => NodemetaChangedSpec {
        list: AbsNodeMetadataList [wrap(ZLibCompressed<AbsNodeMetadataList>)]
    },

    SetSun, 0x5a, Default, true => SetSunSpec {
        sun: SunParams
    },

    SetMoon, 0x5b, Default, true => SetMoonSpec {
        moon: MoonParams
    },

    SetStars, 0x5c, Default, true => SetStarsSpec {
        stars: StarParams
    },

    SrpBytesSB, 0x60, Default, true => SrpBytesSBSpec {
         s: Vec<u8> [wrap(BinaryData16)],
         b: Vec<u8> [wrap(BinaryData16)]
    },

    FormspecPrepend, 0x61, Default, true => FormspecPrependSpec {
        formspec_prepend: String
    },

    MinimapModes, 0x62, Default, true => MinimapModesSpec {
        modes: MinimapModeList
    },

    SetLighting, 0x63, Default, true => SetLightingSpec {
        lighting: Lighting
    }
});
