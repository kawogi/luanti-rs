use crate::wire::command::Command;
use crate::wire::packet::InnerBody;
use crate::wire::packet::MAX_ORIGINAL_BODY_SIZE;
use crate::wire::packet::MAX_SPLIT_BODY_SIZE;
use crate::wire::packet::OriginalBody;
use crate::wire::packet::SplitBody;
use crate::wire::ser::MockSerializer;
use crate::wire::ser::Serialize;
use crate::wire::ser::VecSerializer;
use crate::wire::types::ProtocolContext;

use super::sequence_number::SequenceNumber;

pub(super) struct SplitSender {
    next_seqnum: SequenceNumber,
}

impl SplitSender {
    pub(super) fn new() -> Self {
        Self {
            next_seqnum: SequenceNumber::init(),
        }
    }

    /// Push a Command for transmission
    /// This will possibly split it into 1 or more packets.
    pub(super) fn push(
        &mut self,
        context: ProtocolContext,
        command: Command,
    ) -> anyhow::Result<Vec<InnerBody>> {
        let total_size = {
            let mut ser = MockSerializer::new(context);
            Command::serialize(&command, &mut ser)?;
            ser.len()
        };
        let mut result = Vec::new();
        // Packets should serialize to at most 512 bytes
        if total_size <= MAX_ORIGINAL_BODY_SIZE {
            // Doesn't need to be split
            result.push(InnerBody::Original(OriginalBody {
                command: Some(command),
            }));
        } else {
            // TODO(paradust): Can this extra allocation be avoided?
            let mut ser = VecSerializer::new(context, total_size);
            Command::serialize(&command, &mut ser)?;
            let data = ser.take();
            assert_eq!(data.len(), total_size, "length mismatch");
            let mut index: usize = 0;
            let mut offset: usize = 0;
            let total_chunks = total_size.div_ceil(MAX_SPLIT_BODY_SIZE);
            while offset < total_size {
                let end = std::cmp::min(offset + MAX_SPLIT_BODY_SIZE, total_size);
                result.push(InnerBody::Split(SplitBody {
                    seqnum: self.next_seqnum.partial(),
                    chunk_count: total_chunks as u16,
                    chunk_num: index as u16,
                    chunk_data: data[offset..end].to_vec(),
                }));
                offset += MAX_SPLIT_BODY_SIZE;
                index += 1;
            }
            assert_eq!(index, total_chunks, "size mismatch");
            self.next_seqnum.inc();
        }
        Ok(result)
    }
}
