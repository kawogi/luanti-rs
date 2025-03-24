use super::CommandProperties;
#[allow(clippy::wildcard_imports, reason = "greatly simplifies macros")]
use crate::types::*;
use crate::wire::audit::audit_command;
use crate::wire::channel_id::ChannelId;
use crate::wire::deser::Deserialize;
use crate::wire::deser::DeserializeError;
use crate::wire::deser::DeserializeResult;
use crate::wire::deser::Deserializer;
use crate::wire::ser::Serialize;
use crate::wire::ser::SerializeResult;
use crate::wire::ser::Serializer;
use anyhow::bail;
use luanti_protocol_derive::LuantiDeserialize;
use luanti_protocol_derive::LuantiSerialize;
use std::ops::Deref;

define_protocol!(41, 0x4f457403, ToServer, ToServerCommand => {
    /////////////////////////////////////////////////////////////////////////
    // ToServer

    Init, 0x02, Init, false => InitSpec,
    Init2, 0x11, Init, true => Init2Spec,
    ModchannelJoin, 0x17, Default, true => ModchannelJoinSpec,
    ModchannelLeave, 0x18, Default, true => ModchannelLeaveSpec,
    TSModchannelMsg, 0x19, Default, true => TSModchannelMsgSpec,
    Playerpos, 0x23, Default, false => PlayerPosCommand,
    Gotblocks, 0x24, Response, true => GotblocksSpec,
    Deletedblocks, 0x25, Response, true => DeletedblocksSpec,
    InventoryAction, 0x31, Default, true => InventoryActionSpec,
    TSChatMessage, 0x32, Default, true => TSChatMessageSpec,
    Damage, 0x35, Default, true => DamageSpec,
    Playeritem, 0x37, Default, true => PlayeritemSpec,
    Respawn, 0x38, Default, true => RespawnSpec,
    Interact, 0x39, Default, true => InteractSpec,
    RemovedSounds, 0x3a, Response, true => RemovedSoundsSpec,
    NodemetaFields, 0x3b, Default, true => NodemetaFieldsSpec,
    InventoryFields, 0x3c, Default, true => InventoryFieldsSpec,
    RequestMedia, 0x40, Init, true => RequestMediaSpec,
    HaveMedia, 0x41, Response, true => HaveMediaSpec,
    ClientReady, 0x43, Init, true => ClientReadySpec,
    FirstSrp, 0x50, Init, true => FirstSrpSpec,
    SrpBytesA, 0x51, Init, true => SrpBytesASpec,
    SrpBytesM, 0x52, Init, true => SrpBytesMSpec,
    UpdateClientInfo, 0x53, Init, true => UpdateClientInfoSpec
});

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct UpdateClientInfoSpec {
    pub render_target_size: v2u32,
    pub real_gui_scaling: f32,
    pub real_hud_scaling: f32,
    pub max_fs_size: v2f,
    pub touch_controls: bool,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct InitSpec {
    pub serialization_ver_max: u8,
    pub supp_compr_modes: u16,
    pub min_net_proto_version: u16,
    pub max_net_proto_version: u16,
    pub player_name: String,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct Init2Spec {
    pub lang: Option<String>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct ModchannelJoinSpec {
    pub channel_name: String,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct ModchannelLeaveSpec {
    pub channel_name: String,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct TSModchannelMsgSpec {
    pub channel_name: String,
    pub channel_msg: String,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct PlayerPosCommand {
    pub player_pos: PlayerPos,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct GotblocksSpec {
    #[wrap(Array8<v3s16>)]
    pub blocks: Vec<v3s16>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct DeletedblocksSpec {
    #[wrap(Array8<v3s16>)]
    pub blocks: Vec<v3s16>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct InventoryActionSpec {
    pub action: InventoryAction,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct TSChatMessageSpec {
    #[wrap(WString)]
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct DamageSpec {
    pub damage: u16,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct PlayeritemSpec {
    pub item: u16,
}

#[derive(Debug, Clone, PartialEq, Default, LuantiSerialize, LuantiDeserialize)]
pub struct RespawnSpec;

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct InteractSpec {
    pub action: InteractAction,
    pub item_index: u16,
    #[wrap(Wrapped32<PointedThing>)]
    pub pointed_thing: PointedThing,
    pub player_pos: PlayerPos,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct RemovedSoundsSpec {
    #[wrap(Array16<s32>)]
    pub ids: Vec<s32>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct NodemetaFieldsSpec {
    pub p: v3s16,
    pub form_name: String,
    #[wrap(Array16<Pair<String,LongString>>)]
    pub fields: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct InventoryFieldsSpec {
    pub client_formspec_name: String,
    #[wrap(Array16<Pair<String,LongString>>)]
    pub fields: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct RequestMediaSpec {
    #[wrap(Array16<String>)]
    pub files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct HaveMediaSpec {
    #[wrap(Array8<u32>)]
    pub tokens: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct ClientReadySpec {
    pub major_ver: u8,
    pub minor_ver: u8,
    pub patch_ver: u8,
    pub reserved: u8,
    pub full_ver: String,
    pub formspec_ver: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct FirstSrpSpec {
    #[wrap(BinaryData16)]
    pub salt: Vec<u8>,
    #[wrap(BinaryData16)]
    pub verification_key: Vec<u8>,
    pub is_empty: bool,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct SrpBytesASpec {
    #[wrap(BinaryData16)]
    pub bytes_a: Vec<u8>,
    pub based_on: u8,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct SrpBytesMSpec {
    #[wrap(BinaryData16)]
    pub bytes_m: Vec<u8>,
}
