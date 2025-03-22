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

define_protocol!(41, 0x4f457403, ToServer, ToServerCommand => {
    /////////////////////////////////////////////////////////////////////////
    // ToServer

    Init, 0x02, Init, false => InitSpec,
    Init2, 0x11, Init, true => Init2Spec,
    ModchannelJoin, 0x17, Default, true => ModchannelJoinSpec,
    ModchannelLeave, 0x18, Default, true => ModchannelLeaveSpec,
    TSModchannelMsg, 0x19, Default, true => TSModchannelMsgSpec,
    Playerpos, 0x23, Default, false => PlayerposSpec,
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

// $($crate::proto_struct!($spec_ty { $($fname: $ftype $([$attr])?),* });)*

proto_struct! {
    UpdateClientInfoSpec {
        render_target_size: v2u32,
        real_gui_scaling: f32,
        real_hud_scaling: f32,
        max_fs_size: v2f
    }
}

proto_struct! {
    InitSpec {
        serialization_ver_max: u8,
        supp_compr_modes: u16,
        min_net_proto_version: u16,
        max_net_proto_version: u16,
        player_name: String
    }
}

proto_struct! {
    Init2Spec {
        lang: Option<String>
    }
}

proto_struct! {
    ModchannelJoinSpec {
        channel_name: String
    }
}

proto_struct! {
    ModchannelLeaveSpec {
        channel_name: String
    }
}

proto_struct! {
    TSModchannelMsgSpec {
        channel_name: String,
        channel_msg: String
    }
}

proto_struct! {
    PlayerposSpec {
        player_pos: PlayerPos
    }
}

proto_struct! {
    GotblocksSpec {
        blocks: Vec<v3s16> [wrap(Array8<v3s16>)]
    }
}

proto_struct! {
    DeletedblocksSpec {
        blocks: Vec<v3s16> [wrap(Array8<v3s16>)]
    }
}

proto_struct! {
    InventoryActionSpec {
        action: InventoryAction
    }
}

proto_struct! {
    TSChatMessageSpec {
        message: String [wrap(WString)]
    }
}

proto_struct! {
    DamageSpec {
        damage: u16
    }
}

proto_struct! {
    PlayeritemSpec {
        item: u16
    }
}

proto_struct! {
    RespawnSpec {
        // empty
    }
}

proto_struct! {
    InteractSpec {
        action: InteractAction,
        item_index: u16,
        pointed_thing: PointedThing [wrap(Wrapped32<PointedThing>)],
        player_pos: PlayerPos
    }
}

proto_struct! {
    RemovedSoundsSpec {
        ids: Vec<s32> [wrap(Array16<s32>)]
    }
}

proto_struct! {
    NodemetaFieldsSpec {
        p: v3s16,
        form_name: String,
        // (name, value)
        fields: Vec<(String, String)> [wrap(Array16<Pair<String, LongString>>)]
    }
}

proto_struct! {
    InventoryFieldsSpec {
        client_formspec_name: String,
        fields: Vec<(String, String)> [wrap(Array16<Pair<String, LongString>>)]
    }
}

proto_struct! {
    RequestMediaSpec {
        files: Vec<String> [wrap(Array16<String>)]
    }
}

proto_struct! {
    HaveMediaSpec {
        tokens: Vec<u32> [wrap(Array8<u32>)]
    }
}

proto_struct! {
    ClientReadySpec {
        major_ver: u8,
        minor_ver: u8,
        patch_ver: u8,
        reserved: u8,
        full_ver: String,
        formspec_ver: Option<u16>
    }
}

proto_struct! {
    FirstSrpSpec {
        salt: Vec<u8> [wrap(BinaryData16)],
        verification_key: Vec<u8> [wrap(BinaryData16)],
        is_empty: bool
    }
}

proto_struct! {
    SrpBytesASpec {
        bytes_a: Vec<u8> [wrap(BinaryData16)],
        based_on: u8
    }
}

proto_struct! {
    SrpBytesMSpec {
        bytes_m: Vec<u8> [wrap(BinaryData16)]
    }
}
