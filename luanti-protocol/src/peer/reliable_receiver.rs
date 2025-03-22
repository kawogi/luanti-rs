use crate::wire::packet::InnerBody;
use crate::wire::packet::ReliableBody;
use std::collections::BTreeMap;

use super::sequence_number::SequenceNumber;

pub(super) struct ReliableReceiver {
    // Next sequence number in the reliable stream
    next_seqnum: SequenceNumber,

    // Stores packets that have been received, but not yet processed,
    // because we're waiting for earlier packets.
    // It must always be true that: smallest key in buffer > next_seqnum
    // TODO documentation doesn't match the implementation. After a `push`, `buffer` may equal `next_seqnum`
    buffer: BTreeMap<SequenceNumber, InnerBody>,
}

impl ReliableReceiver {
    pub(super) fn new() -> Self {
        ReliableReceiver {
            next_seqnum: SequenceNumber::init(),
            buffer: BTreeMap::new(),
        }
    }

    /// Push a reliable packet (from remote) into the receiver
    pub(super) fn push(&mut self, body: ReliableBody) {
        let seqnum = self.next_seqnum.goto(body.seqnum);
        if seqnum >= self.next_seqnum {
            // Future packet. Put it in the buffer.
            // Don't override it if it's already there.
            self.buffer.entry(seqnum).or_insert(body.inner);
        } else {
            // Packet was already received and processed. Ignore
        }
    }

    // Pull a single body to be processed, from the reliable stream.
    // These are guaranteed to be in the same order as they were sent.
    // This should be called until exhaustion, after a push.
    pub(super) fn pop(&mut self) -> Option<InnerBody> {
        match self.buffer.first_key_value().map(|(seqnum, _)| *seqnum) {
            Some(seqnum) => (seqnum == self.next_seqnum).then(|| {
                self.next_seqnum.inc();
                self.buffer.pop_first().unwrap().1
            }),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::server_to_client::HudrmSpec;
    use crate::commands::server_to_client::ToClientCommand;
    use crate::commands::*;
    use crate::wire::packet::OriginalBody;
    use crate::wire::packet::PacketBody;
    use crate::wire::sequence_number::WrappingSequenceNumber;
    use rand::prelude::*;
    use rand::rng;

    use super::*;

    fn make_inner(index: u32) -> InnerBody {
        // The Hudrm command is only used here because it stores a u32
        // which can be used to verify the packet contents.
        let command = Command::ToClient(ToClientCommand::Hudrm(Box::new(HudrmSpec {
            server_id: index,
        })));
        InnerBody::Original(OriginalBody {
            command: Some(command),
        })
    }

    fn recover_index(body: &InnerBody) -> u32 {
        match body {
            InnerBody::Original(body) => match body.command.as_ref() {
                Some(Command::ToClient(ToClientCommand::Hudrm(spec))) => spec.server_id,
                _ => panic!("Unexpected body"),
            },
            _ => panic!("Unexpected body"),
        }
    }

    #[test]
    fn reliable_receiver_test() {
        // Generate random reliable packets

        // The plan:
        // 1) Feed in 30000 reliable packets in a random order
        // 2) Pull them out as they become available.
        // 3) Do this 5 times to test wrapping seqnum. (doing this in chunks guarantees the window never exceeds 30000)
        const CHUNK_LEN: u32 = 30000_u32;

        let mut receiver = ReliableReceiver::new();
        let mut offset: u32 = 0;
        for _ in 0..5 {
            let mut packets: Vec<ReliableBody> = (offset..offset + CHUNK_LEN)
                .map(|packet_index| {
                    #[expect(clippy::cast_possible_truncation, reason = "truncation is on purpose")]
                    let seqnum = WrappingSequenceNumber::INITIAL + (packet_index as u16);
                    match make_inner(packet_index).into_reliable(seqnum) {
                        PacketBody::Reliable(rb) => rb,
                        PacketBody::Inner(_) => panic!(),
                    }
                })
                .collect();
            packets.shuffle(&mut rng());

            let mut out: Vec<u32> = Vec::new();
            for pkt in packets {
                receiver.push(pkt);
                while let Some(body) = receiver.pop() {
                    let recovered_index = recover_index(&body);
                    out.push(recovered_index);
                }
            }
            assert_eq!(out.len(), CHUNK_LEN as usize, "Not all packets processed");
            let expected: Vec<u32> = (offset..offset + CHUNK_LEN).collect();
            for i in 0..out.len() {
                assert_eq!(out[i], expected[i]);
            }
            offset += CHUNK_LEN;
        }
    }
}
