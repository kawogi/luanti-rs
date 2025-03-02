use super::util::rel_to_abs;
use crate::wire::packet::InnerBody;
use crate::wire::packet::ReliableBody;
use crate::wire::packet::SEQNUM_INITIAL;
use std::collections::BTreeMap;

pub(super) struct ReliableReceiver {
    // Next sequence number in the reliable stream
    next_seqnum: u64,

    // Stores packets that have been received, but not yet processed,
    // because we're waiting for earlier packets.
    // It must always be true that: smallest key in buffer > next_seqnum
    buffer: BTreeMap<u64, InnerBody>,
}

impl ReliableReceiver {
    pub(super) fn new() -> Self {
        ReliableReceiver {
            next_seqnum: u64::from(SEQNUM_INITIAL),
            buffer: BTreeMap::new(),
        }
    }

    /// Push a reliable packet (from remote) into the receiver
    pub(super) fn push(&mut self, body: ReliableBody) {
        let seqnum = rel_to_abs(self.next_seqnum, body.seqnum);
        if seqnum < self.next_seqnum {
            // Packet was already received and processed. Ignore
        } else if seqnum >= self.next_seqnum {
            // Future packet. Put it in the buffer.
            // Don't override it if it's already there.
            self.buffer.entry(seqnum).or_insert(body.inner);
        }
    }

    // Pull a single body to be processed, from the reliable stream.
    // These are guaranteed to be in the same order as they were sent.
    // This should be called until exhaustion, after a push.
    pub(super) fn pop(&mut self) -> Option<InnerBody> {
        match self.buffer.first_key_value().map(|(seqnum, _)| *seqnum) {
            Some(seqnum) => (seqnum == self.next_seqnum).then(|| {
                self.next_seqnum += 1;
                self.buffer.pop_first().unwrap().1
            }),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::Wrapping;

    use crate::wire::command::*;
    use crate::wire::packet::OriginalBody;
    use crate::wire::packet::PacketBody;
    use rand::prelude::*;
    use rand::rng;

    use super::*;

    fn make_inner(index: u32) -> InnerBody {
        // The Hudrm command is only used here because it stores a u32
        // which can be used to verify the packet contents.
        let command = Command::ToClient(ToClientCommand::Hudrm(Box::new(HudrmSpec {
            server_id: index,
        })));
        InnerBody::Original(OriginalBody { command })
    }

    fn recover_index(body: &InnerBody) -> u32 {
        match body {
            InnerBody::Original(body) => match &body.command {
                Command::ToClient(ToClientCommand::Hudrm(spec)) => spec.server_id,
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
                .map(|i| {
                    #[expect(clippy::cast_possible_truncation, reason = "truncation is on purpose")]
                    let seqnum: u16 = (Wrapping(SEQNUM_INITIAL) + Wrapping(i as u16)).0;
                    match make_inner(i).into_reliable(seqnum) {
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
