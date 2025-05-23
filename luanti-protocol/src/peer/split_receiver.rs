use crate::wire::packet::SplitBody;
use crate::wire::sequence_number::WrappingSequenceNumber;
use anyhow::bail;
use log::warn;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;

const SPLIT_TIMEOUT: Duration = Duration::from_secs(30);

pub(super) struct IncomingBuffer {
    chunk_count: u16,
    chunks: BTreeMap<u16, Vec<u8>>,
    timeout: Instant,
}

impl IncomingBuffer {
    fn new(now: Instant, chunk_count: u16) -> Self {
        Self {
            chunk_count,
            chunks: BTreeMap::new(),
            timeout: now + SPLIT_TIMEOUT,
        }
    }

    /// Push a new split packet into the split receiver
    /// If a command has become ready as a result, true is returned.
    fn push(&mut self, now: Instant, body: SplitBody) -> anyhow::Result<bool> {
        if body.chunk_count != self.chunk_count {
            bail!("Split packet corrupt: chunk_count mismatch");
        } else if body.chunk_num >= self.chunk_count {
            bail!("Split packet corrupt: chunk_num >= chunk_count");
        }
        self.timeout = now + SPLIT_TIMEOUT;
        if self
            .chunks
            .insert(body.chunk_num, body.chunk_data)
            .is_some()
        {
            warn!("received duplicate packet for chunk #{}", body.chunk_num);
        }
        Ok(self.chunks.len() == self.chunk_count as usize)
    }

    fn take(self) -> Vec<u8> {
        assert_eq!(
            self.chunks.len(),
            self.chunk_count as usize,
            "chunk count mismatch"
        );
        // TODO replace with `flatten`
        let total_size: usize = self.chunks.values().map(Vec::len).sum();
        let mut buf = Vec::with_capacity(total_size);
        for chunk in self.chunks.values() {
            buf.extend_from_slice(chunk);
        }
        assert_eq!(buf.len(), total_size, "buffer length mismatch");
        buf
    }
}

pub(super) struct SplitReceiver {
    pending: HashMap<WrappingSequenceNumber, IncomingBuffer>,
}

impl SplitReceiver {
    pub(super) fn new() -> Self {
        Self {
            pending: HashMap::new(),
        }
    }

    /// Push a split packet for reconstruction
    /// Returns the finished command if it is ready
    pub(super) fn push(
        &mut self,
        now: Instant,
        body: SplitBody,
    ) -> anyhow::Result<Option<Vec<u8>>> {
        let seqnum = body.seqnum;
        let should_take = self
            .pending
            .entry(seqnum)
            .or_insert_with(|| IncomingBuffer::new(now, body.chunk_count))
            .push(now, body)?;

        if should_take {
            Ok(Some(self.pending.remove(&seqnum).unwrap().take()))
        } else {
            Ok(None)
        }
    }
}
