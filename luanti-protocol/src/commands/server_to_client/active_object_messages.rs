use crate::wire::deser::{Deserialize, DeserializeResult, Deserializer};
use crate::wire::ser::{Serialize, SerializeResult, Serializer};
use luanti_protocol_derive::{LuantiDeserialize, LuantiSerialize};

use crate::types::{ActiveObjectCommand, Array0, Wrapped16};

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct ActiveObjectMessagesCommand {
    #[wrap(Array0<ActiveObjectMessage>)]
    pub objects: Vec<ActiveObjectMessage>,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct ActiveObjectMessage {
    pub id: u16,
    #[wrap(Wrapped16<ActiveObjectCommand>)]
    pub data: ActiveObjectCommand,
}
