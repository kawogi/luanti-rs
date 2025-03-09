#![expect(
    clippy::min_ident_chars,
    reason = "//TODO rename the remaining fields within the macro"
)]

use super::audit::audit_command;
use super::channel_id::ChannelId;
use super::deser::Deserialize;
use super::deser::DeserializeError;
use super::deser::DeserializeResult;
use super::deser::Deserializer;
use super::ser::Serialize;
use super::ser::SerializeResult;
use super::ser::Serializer;
#[allow(clippy::wildcard_imports, reason = "greatly simplifies macros")]
use super::types::*;
use anyhow::bail;
use luanti_protocol_derive::LuantiDeserialize;
use luanti_protocol_derive::LuantiSerialize;
use std::ops::Deref;

#[macro_export]
macro_rules! as_item {
    ($i:item) => {
        $i
    };
}

#[macro_export]
macro_rules! default_serializer {
    ($spec_ty: ident { }) => {
        impl Serialize for $spec_ty {
            type Input = Self;
            fn serialize<S: Serializer>(value: &Self::Input, _: &mut S) -> SerializeResult {
                Ok(())
            }
        }
    };
    ($spec_ty: ident { $($fname: ident: $ftyp: ty ),+ }) => {
        impl Serialize for $spec_ty {
            type Input = Self;
            fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
                $(
                    <$ftyp as Serialize>::serialize(&value.$fname, ser)?;
                )+
                Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! default_deserializer {
    ($spec_ty: ident { }) => {
        impl Deserialize for $spec_ty {
            type Output = Self;
            fn deserialize(_deserializer: &mut Deserializer) -> DeserializeResult<Self> {
                Ok($spec_ty)
            }
        }
    };
    ($spec_ty: ident { $($fname: ident: $ftyp: ty ),+ }) => {
        impl Deserialize for $spec_ty {
            type Output = Self;
            fn deserialize(deserializer: &mut Deserializer) -> DeserializeResult<Self> {
                Ok($spec_ty {
                    $(
                        $fname: <$ftyp>::deserialize(deser)?,
                    )+
                })
            }
        }
    };
}

#[macro_export]
macro_rules! implicit_from {
    ($command_ty: ident, $name: ident, $spec_ty: ident) => {
        impl From<$spec_ty> for $command_ty {
            fn from(value: $spec_ty) -> Self {
                $command_ty::$name(Box::new(value))
            }
        }
    };
}

#[macro_export]
macro_rules! proto_struct {
    ($spec_ty: ident { }) => {
        #[derive(Debug, Clone, PartialEq, Default, LuantiSerialize, LuantiDeserialize)]
        pub struct $spec_ty;
    };
    ($spec_ty: ident {
        $($fname: ident: $ftype: ty $([$attr:meta])? ),+
    }) => {
        $crate::as_item! {
            #[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
            pub struct $spec_ty {
               $( $(#[$attr])? pub $fname: $ftype),+
            }
        }
    };
}

macro_rules! define_protocol {
    ($version: literal,
     $protocol_id: literal,
     $dir: ident,
     $command_ty: ident => {
         $($name: ident, $id: literal, $channel: ident, $reliable: literal => $spec_ty: ident
             { $($fname: ident : $ftype: ty $([$attr:meta])? ),* } ),*
    }) => {
        $crate::as_item! {
            #[derive(Debug, PartialEq, Clone)]
            pub enum $command_ty {
                $($name(Box<$spec_ty>)),*,
            }
        }

        $crate::as_item! {
            impl CommandProperties for $command_ty {
                fn direction(&self) -> CommandDirection {
                    CommandDirection::$dir
                }

                fn default_channel(&self) -> ChannelId {
                    match self {
                        $($command_ty::$name(_) => ChannelId::$channel),*,
                    }
                }

                fn default_reliability(&self) -> bool {
                    match self {
                        $($command_ty::$name(_) => $reliable),*,
                    }
                }

                fn command_name(&self) -> &'static str {
                    match self {
                        $($command_ty::$name(_) => stringify!($name)),*,
                    }
                }
            }
        }

        $crate::as_item! {
            impl Serialize for $command_ty {
                type Input = Self;
                fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
                    match value {
                        $($command_ty::$name(spec) => { u16::serialize(&$id, ser)?; <$spec_ty as Serialize>::serialize(Deref::deref(spec), ser) }),*,
                    }
                }
            }
        }

        $crate::as_item! {
            impl Deserialize for $command_ty {
                type Output = Option<Self>;
                fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
                    // The first packet a client sends doesn't contain a command but has an empty payload.
                    // It only serves the purpose of triggering the creation of a peer entry within the server.
                    // Rather than requesting every caller to perform a pre-check for a non-empty payload,
                    // we just return an `Option` to force the caller to handle this case.
                    if !deserializer.has_remaining() {
                        return Ok(None);
                    }
                    let orig_buffer = deserializer.peek_all();
                    log::trace!("orig_buffer: {:?}", &orig_buffer[0..(orig_buffer.len().min(64))]);
                    let command_id = u16::deserialize(deserializer)?;
                    let dir = deserializer.direction();
                    let result = match (dir, command_id) {
                        $( (CommandDirection::$dir, $id) => $command_ty::$name(Box::new(<$spec_ty as Deserialize>::deserialize(deserializer)?)) ),*,
                        _ => bail!(DeserializeError::BadPacketId(dir, command_id)),
                    };
                    audit_command(deserializer.context(), orig_buffer, &result);
                    Ok(Some(result))
                }
            }
        }

        $($crate::proto_struct!($spec_ty { $($fname: $ftype $([$attr])?),* });)*
        $($crate::implicit_from!($command_ty, $name, $spec_ty);)*

    };
}

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

define_protocol!(41, 0x4f457403, ToServer, ToServerCommand => {
    /////////////////////////////////////////////////////////////////////////
    // ToServer
    Null, 0x00, Default, false => NullSpec {
        // This appears to be sent before init to initialize
        // the reliable seqnum and peer id.
    },

    Init, 0x02, Init, false => InitSpec {
        serialization_ver_max: u8,
        supp_compr_modes: u16,
        min_net_proto_version: u16,
        max_net_proto_version: u16,
        player_name: String
    },

    Init2, 0x11, Init, true => Init2Spec {
        lang: Option<String>
    },

    ModchannelJoin, 0x17, Default, true => ModchannelJoinSpec {
        channel_name: String
    },

    ModchannelLeave, 0x18, Default, true => ModchannelLeaveSpec {
        channel_name: String
    },

    TSModchannelMsg, 0x19, Default, true => TSModchannelMsgSpec {
        channel_name: String,
        channel_msg: String
    },

    Playerpos, 0x23, Default, false => PlayerposSpec {
        player_pos: PlayerPos
    },

    Gotblocks, 0x24, Response, true => GotblocksSpec {
        blocks: Vec<v3s16> [wrap(Array8<v3s16>)]
    },

    Deletedblocks, 0x25, Response, true => DeletedblocksSpec {
        blocks: Vec<v3s16> [wrap(Array8<v3s16>)]
    },

    InventoryAction, 0x31, Default, true => InventoryActionSpec {
        action: InventoryAction
    },

    TSChatMessage, 0x32, Default, true => TSChatMessageSpec {
        message: String [wrap(WString)]
    },

    Damage, 0x35, Default, true => DamageSpec {
        damage: u16
    },

    Playeritem, 0x37, Default, true => PlayeritemSpec {
        item: u16
    },

    Respawn, 0x38, Default, true => RespawnSpec {
        // empty
    },

    Interact, 0x39, Default, true => InteractSpec {
        action: InteractAction,
        item_index: u16,
        pointed_thing: PointedThing [wrap(Wrapped32<PointedThing>)],
        player_pos: PlayerPos
    },

    RemovedSounds, 0x3a, Response, true => RemovedSoundsSpec {
        ids: Vec<s32> [wrap(Array16<s32>)]
    },

    NodemetaFields, 0x3b, Default, true => NodemetaFieldsSpec {
        p: v3s16,
        form_name: String,
        // (name, value)
        fields: Vec<(String, String)> [wrap(Array16<Pair<String, LongString>>)]
    },

    InventoryFields, 0x3c, Default, true => InventoryFieldsSpec {
        client_formspec_name: String,
        fields: Vec<(String, String)> [wrap(Array16<Pair<String, LongString>>)]
    },

    RequestMedia, 0x40, Init, true => RequestMediaSpec {
        files: Vec<String> [wrap(Array16<String>)]
    },

    HaveMedia, 0x41, Response, true => HaveMediaSpec {
        tokens: Vec<u32> [wrap(Array8<u32>)]
    },

    ClientReady, 0x43, Init, true => ClientReadySpec {
        major_ver: u8,
        minor_ver: u8,
        patch_ver: u8,
        reserved: u8,
        full_ver: String,
        formspec_ver: Option<u16>
    },

    FirstSrp, 0x50, Init, true => FirstSrpSpec {
        salt: Vec<u8> [wrap(BinaryData16)],
        verification_key: Vec<u8> [wrap(BinaryData16)],
        is_empty: bool
    },

    SrpBytesA, 0x51, Init, true => SrpBytesASpec {
        bytes_a: Vec<u8> [wrap(BinaryData16)],
        based_on: u8
    },

    SrpBytesM, 0x52, Init, true => SrpBytesMSpec {
        bytes_m: Vec<u8> [wrap(BinaryData16)]
    },

    UpdateClientInfo, 0x53, Init, true => UpdateClientInfoSpec {
        render_target_size: v2u32,
        real_gui_scaling: f32,
        real_hud_scaling: f32,
        max_fs_size: v2f
    }
});

#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    ToServer(ToServerCommand),
    ToClient(ToClientCommand),
}

pub trait CommandProperties {
    fn direction(&self) -> CommandDirection;
    fn default_channel(&self) -> ChannelId;
    fn default_reliability(&self) -> bool;
    fn command_name(&self) -> &'static str;
}

/// This only exists to make `audit_command` generic, but it
/// wasn't as clean as I hoped.
/// TODO(paradust): Factor this out.
pub trait CommandRef: CommandProperties + std::fmt::Debug {
    fn toserver_ref(&self) -> Option<&ToServerCommand>;
    fn toclient_ref(&self) -> Option<&ToClientCommand>;
}

pub fn serialize_commandref<Cmd: CommandRef, S: Serializer>(
    cmd: &Cmd,
    ser: &mut S,
) -> SerializeResult {
    if let Some(command) = cmd.toserver_ref() {
        ToServerCommand::serialize(command, ser)?;
    }
    if let Some(command) = cmd.toclient_ref() {
        ToClientCommand::serialize(command, ser)?;
    }
    Ok(())
}

impl CommandProperties for Command {
    fn direction(&self) -> CommandDirection {
        match self {
            Command::ToServer(_) => CommandDirection::ToServer,
            Command::ToClient(_) => CommandDirection::ToClient,
        }
    }

    fn default_channel(&self) -> ChannelId {
        match self {
            Command::ToServer(command) => command.default_channel(),
            Command::ToClient(command) => command.default_channel(),
        }
    }

    fn default_reliability(&self) -> bool {
        match self {
            Command::ToServer(command) => command.default_reliability(),
            Command::ToClient(command) => command.default_reliability(),
        }
    }

    fn command_name(&self) -> &'static str {
        match self {
            Command::ToServer(command) => command.command_name(),
            Command::ToClient(command) => command.command_name(),
        }
    }
}

impl CommandRef for Command {
    fn toserver_ref(&self) -> Option<&ToServerCommand> {
        match self {
            Command::ToServer(command) => Some(command),
            Command::ToClient(_) => None,
        }
    }

    fn toclient_ref(&self) -> Option<&ToClientCommand> {
        match self {
            Command::ToServer(_) => None,
            Command::ToClient(command) => Some(command),
        }
    }
}

impl CommandRef for ToClientCommand {
    fn toserver_ref(&self) -> Option<&ToServerCommand> {
        None
    }

    fn toclient_ref(&self) -> Option<&ToClientCommand> {
        Some(self)
    }
}

impl CommandRef for ToServerCommand {
    fn toserver_ref(&self) -> Option<&ToServerCommand> {
        Some(self)
    }

    fn toclient_ref(&self) -> Option<&ToClientCommand> {
        None
    }
}

impl Serialize for Command {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        match value {
            Command::ToServer(command) => ToServerCommand::serialize(command, ser),
            Command::ToClient(command) => ToClientCommand::serialize(command, ser),
        }
    }
}

impl Deserialize for Command {
    type Output = Option<Self>;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        Ok(match deser.direction() {
            CommandDirection::ToClient => ToClientCommand::deserialize(deser)?.map(Self::ToClient),
            CommandDirection::ToServer => ToServerCommand::deserialize(deser)?.map(Self::ToServer),
        })
    }
}
