use anyhow::bail;
use log::trace;

use super::channel_id::ChannelId;
use super::deser::Deserialize;
use super::deser::DeserializeError;
use super::deser::DeserializeResult;
use super::deser::Deserializer;
use super::peer_id::PeerId;
use super::sequence_number::WrappingSequenceNumber;
use super::ser::Serialize;
use super::ser::SerializeResult;
use super::ser::Serializer;
use crate::commands::Command;

pub const PROTOCOL_ID: u32 = 0x4f45_7403;

pub const LATEST_PROTOCOL_VERSION: u16 = 47;
pub const SER_FMT_VER_HIGHEST_WRITE: u8 = 29;

// Serialization format of map data
pub const SER_FMT_HIGHEST_READ: u8 = 29;
pub const SER_FMT_HIGHEST_WRITE: u8 = 29;
pub const SER_FMT_LOWEST_READ: u8 = 28;
pub const SER_FMT_LOWEST_WRITE: u8 = 29;

pub const MAX_PACKET_SIZE: usize = 512;
pub const PACKET_HEADER_SIZE: usize = 7;
pub const RELIABLE_HEADER_SIZE: usize = 3;
pub const SPLIT_HEADER_SIZE: usize = 7;
pub const MAX_ORIGINAL_BODY_SIZE: usize =
    MAX_PACKET_SIZE - PACKET_HEADER_SIZE - RELIABLE_HEADER_SIZE;
pub const MAX_SPLIT_BODY_SIZE: usize = MAX_ORIGINAL_BODY_SIZE - SPLIT_HEADER_SIZE;

#[derive(Debug, Clone, PartialEq)]
pub struct AckBody {
    pub seqnum: WrappingSequenceNumber,
}

impl AckBody {
    #[must_use]
    pub fn new(seqnum: WrappingSequenceNumber) -> Self {
        AckBody { seqnum }
    }
    #[must_use]
    pub fn into_inner(self) -> InnerBody {
        InnerBody::Control(ControlBody::Ack(self))
    }
}

impl Serialize for AckBody {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        WrappingSequenceNumber::serialize(&value.seqnum, ser)
    }
}

impl Deserialize for AckBody {
    type Output = Self;
    fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let seqnum = WrappingSequenceNumber::deserialize(deserializer)?;
        Ok(Self { seqnum })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetPeerIdBody {
    pub peer_id: PeerId,
}

impl SetPeerIdBody {
    #[must_use]
    pub fn new(peer_id: PeerId) -> Self {
        Self { peer_id }
    }

    #[must_use]
    pub fn into_inner(self) -> InnerBody {
        InnerBody::Control(ControlBody::SetPeerId(self))
    }
}

impl Serialize for SetPeerIdBody {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        PeerId::serialize(&value.peer_id, ser)
    }
}

impl Deserialize for SetPeerIdBody {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        Ok(Self {
            peer_id: PeerId::deserialize(deser)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ControlBody {
    Ack(AckBody),
    SetPeerId(SetPeerIdBody),
    Ping,
    Disconnect,
}

impl ControlBody {
    #[must_use]
    pub fn into_inner(self) -> InnerBody {
        InnerBody::Control(self)
    }
}

impl Serialize for ControlBody {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        let control_type = match value {
            ControlBody::Ack(_) => 0,
            ControlBody::SetPeerId(_) => 1,
            ControlBody::Ping => 2,
            ControlBody::Disconnect => 3,
        };
        u8::serialize(&control_type, ser)?;
        match value {
            ControlBody::Ack(body) => AckBody::serialize(body, ser)?,
            ControlBody::SetPeerId(body) => SetPeerIdBody::serialize(body, ser)?,
            ControlBody::Ping | ControlBody::Disconnect => (),
        }
        Ok(())
    }
}

impl Deserialize for ControlBody {
    type Output = Self;

    fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let control_type = u8::deserialize(deserializer)?;
        trace!("ControlBody::control_type: {control_type}");
        match control_type {
            0 => Ok(ControlBody::Ack(AckBody::deserialize(deserializer)?)),
            1 => Ok(ControlBody::SetPeerId(SetPeerIdBody::deserialize(
                deserializer,
            )?)),
            2 => Ok(ControlBody::Ping),
            3 => Ok(ControlBody::Disconnect),
            _ => bail!(DeserializeError::InvalidValue(String::from(
                "Invalid control_type in ControlBody",
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OriginalBody {
    pub command: Option<Command>,
}

impl Serialize for OriginalBody {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        if let Some(command) = value.command.as_ref() {
            Command::serialize(command, ser)
        } else {
            // the deserializer of a command will handle an empty payload as `None`
            Ok(())
        }
    }
}

impl Deserialize for OriginalBody {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        Ok(OriginalBody {
            command: Command::deserialize(deser)?,
        })
    }
}

#[derive(Clone, PartialEq)]
pub struct SplitBody {
    pub seqnum: WrappingSequenceNumber,
    pub chunk_count: u16,
    pub chunk_num: u16,
    pub chunk_data: Vec<u8>,
}

impl std::fmt::Debug for SplitBody {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SplitBody")
            .field("seqnum", &self.seqnum)
            .field("chunk_count", &self.chunk_count)
            .field("chunk_num", &self.chunk_num)
            .field("chunk_data", &format!("{} bytes", self.chunk_data.len()))
            .finish()
    }
}

impl Serialize for SplitBody {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        WrappingSequenceNumber::serialize(&value.seqnum, ser)?;
        u16::serialize(&value.chunk_count, ser)?;
        u16::serialize(&value.chunk_num, ser)?;
        ser.write_bytes(&value.chunk_data)?;
        Ok(())
    }
}

impl Deserialize for SplitBody {
    type Output = Self;

    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let seqnum = WrappingSequenceNumber::deserialize(deser)?;
        let chunk_count = u16::deserialize(deser)?;
        let chunk_num = u16::deserialize(deser)?;
        let chunk_data = Vec::from(deser.take_all());
        Ok(SplitBody {
            seqnum,
            chunk_count,
            chunk_num,
            chunk_data,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReliableBody {
    pub seqnum: WrappingSequenceNumber,
    pub inner: InnerBody,
}

impl Serialize for ReliableBody {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        let packet_type: u8 = 3;
        u8::serialize(&packet_type, ser)?;
        WrappingSequenceNumber::serialize(&value.seqnum, ser)?;
        InnerBody::serialize(&value.inner, ser)?;
        Ok(())
    }
}

impl Deserialize for ReliableBody {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let packet_type = u8::deserialize(deser)?;
        // very noisy; re-enable if there are protocol errors to be debugged
        // trace!("ReliableBody::packet_type: {packet_type}");
        if packet_type != 3 {
            bail!(DeserializeError::InvalidValue(
                "Invalid packet_type for ReliableBody".into(),
            ))
        }
        let seqnum = WrappingSequenceNumber::deserialize(deser)?;
        // very noisy; re-enable if there are protocol errors to be debugged
        // trace!("ReliableBody::seqnum: {seqnum}");
        Ok(ReliableBody {
            seqnum,
            inner: InnerBody::deserialize(deser)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InnerBody {
    Control(ControlBody),
    Original(OriginalBody),
    Split(SplitBody),
}

impl InnerBody {
    #[must_use]
    pub fn into_reliable(self, seqnum: WrappingSequenceNumber) -> PacketBody {
        PacketBody::Reliable(ReliableBody {
            seqnum,
            inner: self,
        })
    }

    #[must_use]
    pub fn into_unreliable(self) -> PacketBody {
        PacketBody::Inner(self)
    }

    /// Get a reference to the Command this body contains, if any.
    /// If this is part of a split packet, None will be returned
    /// even though there is a fragment of a Command inside.
    ///
    /// This doesn't differentiate between a body which _cannot_ have a command and a body which
    /// _doesn't_ have a command.
    #[must_use]
    pub fn command(&self) -> Option<&Command> {
        match self {
            InnerBody::Original(body) => body.command.as_ref(),
            InnerBody::Control(_) | InnerBody::Split(_) => None,
        }
    }
}

impl Serialize for InnerBody {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        let packet_type: u8 = match value {
            InnerBody::Control(..) => 0,
            InnerBody::Original(..) => 1,
            InnerBody::Split(..) => 2,
        };
        u8::serialize(&packet_type, ser)?;
        match value {
            InnerBody::Control(body) => ControlBody::serialize(body, ser),
            InnerBody::Original(body) => OriginalBody::serialize(body, ser),
            InnerBody::Split(body) => SplitBody::serialize(body, ser),
        }
    }
}

impl Deserialize for InnerBody {
    type Output = Self;

    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let packet_type = u8::deserialize(deser)?;
        // very noisy; re-enable if there are protocol errors to be debugged
        // trace!("InnerBody::type: {packet_type}");
        match packet_type {
            0 => Ok(InnerBody::Control(ControlBody::deserialize(deser)?)),
            1 => Ok(InnerBody::Original(OriginalBody::deserialize(deser)?)),
            2 => Ok(InnerBody::Split(SplitBody::deserialize(deser)?)),
            _ => bail!(DeserializeError::InvalidPacketKind(packet_type)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PacketBody {
    Reliable(ReliableBody),
    Inner(InnerBody),
}

impl PacketBody {
    #[must_use]
    pub fn inner(&self) -> &InnerBody {
        match self {
            PacketBody::Reliable(body) => &body.inner,
            PacketBody::Inner(inner) => inner,
        }
    }

    #[must_use]
    pub fn command(&self) -> Option<&Command> {
        self.inner().command()
    }
}

impl Serialize for PacketBody {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        #![allow(clippy::enum_glob_use, reason = "improves readability")]
        use PacketBody::*;
        // Both ReliableBody and InnerBody will emit their own packet type.
        match value {
            Reliable(body) => ReliableBody::serialize(body, ser),
            Inner(inner) => InnerBody::serialize(inner, ser),
        }
    }
}

impl Deserialize for PacketBody {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        #![allow(clippy::enum_glob_use, reason = "improves readability")]
        use PacketBody::*;
        // Both ReliableBody and InnerBody expect to consume the packet type tag.
        // So only peek it.
        let packet_type = deser.peek(1)?[0];
        match packet_type {
            3 => Ok(Reliable(ReliableBody::deserialize(deser)?)),
            _ => Ok(Inner(InnerBody::deserialize(deser)?)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Packet {
    pub protocol_id: u32,
    pub sender_peer_id: PeerId,
    pub channel: ChannelId,
    pub body: PacketBody,
}

impl Packet {
    #[must_use]
    pub fn new(sender_peer_id: PeerId, channel: ChannelId, body: PacketBody) -> Self {
        Self {
            protocol_id: PROTOCOL_ID,
            sender_peer_id,
            channel,
            body,
        }
    }

    #[must_use]
    pub fn inner(&self) -> &InnerBody {
        self.body.inner()
    }

    #[must_use]
    pub fn as_reliable(&self) -> Option<&ReliableBody> {
        match &self.body {
            PacketBody::Reliable(rb) => Some(rb),
            PacketBody::Inner(_) => None,
        }
    }

    #[must_use]
    pub fn as_control(&self) -> Option<&ControlBody> {
        match self.inner() {
            InnerBody::Control(control) => Some(control),
            InnerBody::Original(_) | InnerBody::Split(_) => None,
        }
    }
}

impl Serialize for Packet {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        u32::serialize(&value.protocol_id, ser)?;
        PeerId::serialize(&value.sender_peer_id, ser)?;
        ChannelId::serialize(&value.channel, ser)?;
        PacketBody::serialize(&value.body, ser)?;
        Ok(())
    }
}

impl Deserialize for Packet {
    type Output = Self;
    fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        // very noisy; re-enable if there are protocol errors to be debugged
        // trace!("deserializing packet");

        let protocol_id = u32::deserialize(deserializer)?;
        if protocol_id != PROTOCOL_ID {
            bail!(DeserializeError::InvalidProtocolId(protocol_id))
        }

        let sender_peer_id = PeerId::deserialize(deserializer)?;
        let channel = ChannelId::deserialize(deserializer)?;

        // very noisy; re-enable if there are protocol errors to be debugged
        // trace!("deserializing packet: sender_peer_id={sender_peer_id}, channel: {channel}");
        let body = PacketBody::deserialize(deserializer)?;

        let pkt = Packet {
            protocol_id,
            sender_peer_id,
            channel,
            body,
        };

        // very noisy; re-enable if there are protocol errors to be debugged
        // trace!("deserialized packet: {pkt:?}");

        Ok(pkt)
    }
}
