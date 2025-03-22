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

proto_struct! {
    HelloSpec {
        serialization_ver: u8,
        compression_mode: u16,
        proto_ver: u16,
        auth_mechs: AuthMechsBitset,
        username_legacy: String
    }
}

proto_struct! {
    AuthAcceptSpec {
        player_pos: v3f,
        map_seed: u64,
        recommended_send_interval: f32,
        sudo_auth_methods: u32
    }
}

proto_struct! {
    AcceptSudoModeSpec {
        // No fields
    }
}

proto_struct! {
    DenySudoModeSpec {
        // No fields
    }
}

proto_struct! {
    AccessDeniedSpec {
        code: AccessDeniedCode,
        reason: String,
        reconnect: bool
    }
}

proto_struct! {
    BlockdataSpec {
        pos: v3s16,
        block: MapBlock,
        network_specific_version: u8
    }
}

proto_struct! {
    AddnodeSpec {
        pos: v3s16,
        node: MapNode,
        keep_metadata: bool
    }
}

proto_struct! {
    RemovenodeSpec {
        pos: v3s16
    }
}

proto_struct! {
    InventorySpec {
        inventory: Inventory
    }
}

proto_struct! {
    TimeOfDaySpec {
        time_of_day: u16,
        time_speed: Option<f32>
    }
}

proto_struct! {
    CsmRestrictionFlagsSpec {
        csm_restriction_flags: u64,
        csm_restriction_noderange: u32
    }
}

proto_struct! {
    PlayerSpeedSpec {
        added_vel: v3f
    }
}

proto_struct! {
    MediaPushSpec {
        raw_hash: String,
        filename: String,
        cached: bool,
        token: u32
    }
}

proto_struct! {
    TCChatMessageSpec {
        version: u8,
        message_type: u8,
        sender: String [wrap(WString)],
        message: String [wrap(WString)],
        timestamp: u64
    }
}

proto_struct! {
    ActiveObjectRemoveAddSpec {
        removed_object_ids: Vec<u16> [wrap(Array16<u16>)],
        added_objects: Vec<AddedObject> [wrap(Array16<AddedObject>)]
    }
}

proto_struct! {
    ActiveObjectMessagesSpec {
        objects: Vec<ActiveObjectMessage> [wrap(Array0<ActiveObjectMessage>)]
    }
}

proto_struct! {
    HpSpec {
        hp: u16,
        damage_effect: Option<bool>
    }
}

proto_struct! {
    MovePlayerSpec {
        pos: v3f,
        pitch: f32,
        yaw: f32
    }
}

proto_struct! {
    AccessDeniedLegacySpec {
        reason: String [wrap(WString)]
    }
}

proto_struct! {
    FovSpec {
        fov: f32,
        is_multiplier: bool,
        transition_time: Option<f32>
    }
}

proto_struct! {
    DeathscreenSpec {
        set_camera_point_target: bool,
        camera_point_target: v3f
    }
}

proto_struct! {
    MediaSpec {
        num_bunches: u16,
        bunch_index: u16,
        files: Vec<MediaFileData> [wrap(Array32<MediaFileData>)]
    }
}

proto_struct! {
    NodedefSpec {
        node_def: NodeDefManager [wrap(ZLibCompressed<NodeDefManager>)]
    }
}

proto_struct! {
    AnnounceMediaSpec {
        files: Vec<MediaAnnouncement> [wrap(Array16<MediaAnnouncement>)],
        remote_servers: String
    }
}

proto_struct! {
    ItemdefSpec {
        item_def: ItemdefList [wrap(ZLibCompressed<ItemdefList>)]
    }
}

proto_struct! {
    PlaySoundSpec {
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
    }
}

proto_struct! {
    StopSoundSpec {
        server_id: s32
    }
}

proto_struct! {
    PrivilegesSpec {
        privileges: Vec<String> [wrap(Array16<String>)]
    }
}

proto_struct! {
    InventoryFormspecSpec {
        formspec: String [wrap(LongString)]
    }
}

proto_struct! {
    DetachedInventorySpec {
        name: String,
        keep_inv: bool,
        // These are present if keep_inv is true.
        ignore: Option<u16>,
        contents: Option<Inventory>
    }
}

proto_struct! {
    ShowFormspecSpec {
        form_spec: String [wrap(LongString)],
        form_name: String
    }
}

proto_struct! {
    MovementSpec {
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
    }
}

proto_struct! {
    SpawnParticleSpec {
        data: ParticleParameters
    }
}

proto_struct! {
    AddParticlespawnerSpec {
        legacy: AddParticleSpawnerLegacy
    }
}

proto_struct! {
    HudaddSpec {
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
    }
}

proto_struct! {
    HudrmSpec {
        server_id: u32
    }
}

proto_struct! {
    HudchangeSpec {
        server_id: u32,
        stat: HudStat
    }
}

proto_struct! {
    HudSetFlagsSpec {
        flags: HudFlags, // flags added
        mask: HudFlags   // flags possibly removed
    }
}

proto_struct! {
    HudSetParamSpec {
        value: HudSetParam
    }
}

proto_struct! {
    BreathSpec {
        breath: u16
    }
}

proto_struct! {
    SetSkySpec {
        params: SkyboxParams
    }
}

proto_struct! {
    OverrideDayNightRatioSpec {
        do_override: bool,
        day_night_ratio: u16
    }
}

proto_struct! {
    LocalPlayerAnimationsSpec {
        idle: v2s32,
        walk: v2s32,
        dig: v2s32,
        walk_dig: v2s32,
        frame_speed: f32
    }
}

proto_struct! {
    EyeOffsetSpec {
        eye_offset_first: v3f,
        eye_offset_third: v3f
    }
}

proto_struct! {
    DeleteParticlespawnerSpec {
        server_id: u32
    }
}

proto_struct! {
    CloudParamsSpec {
        density: f32,
        color_bright: SColor,
        color_ambient: SColor,
        height: f32,
        thickness: f32,
        speed: v2f
    }
}

proto_struct! {
    FadeSoundSpec {
        sound_id: s32,
        step: f32,
        gain: f32
    }
}

proto_struct! {
    UpdatePlayerListSpec {
        typ: u8,
        players: Vec<String> [wrap(Array16<String>)]
    }
}

proto_struct! {
    TCModchannelMsgSpec {
        channel_name: String,
        sender: String,
        channel_msg: String
    }
}

proto_struct! {
    ModchannelSignalSpec {
        signal_tmp: u8,
        channel: String,
        // signal == MODCHANNEL_SIGNAL_SET_STATE
        state: Option<u8>
    }
}

proto_struct! {
    NodemetaChangedSpec {
        list: AbsNodeMetadataList [wrap(ZLibCompressed<AbsNodeMetadataList>)]
    }
}

proto_struct! {
    SetSunSpec {
        sun: SunParams
    }
}

proto_struct! {
    SetMoonSpec {
        moon: MoonParams
    }
}

proto_struct! {
    SetStarsSpec {
        stars: StarParams
    }
}

proto_struct! {
    SrpBytesSBSpec {
         s: Vec<u8> [wrap(BinaryData16)],
         b: Vec<u8> [wrap(BinaryData16)]
    }
}

proto_struct! {
    FormspecPrependSpec {
        formspec_prepend: String
    }
}

proto_struct! {
    MinimapModesSpec {
        modes: MinimapModeList
    }
}

proto_struct! {
    SetLightingSpec {
        lighting: Lighting
    }
}
