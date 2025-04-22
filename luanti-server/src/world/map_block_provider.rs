use std::{
    collections::HashMap,
    sync::Arc,
    thread::{self, JoinHandle},
};

use anyhow::Result;
use log::{error, trace};
use luanti_core::ContentId;
use tokio::sync::mpsc;

use super::{
    WorldUpdate, generation::WorldGenerator, storage::WorldStorage, view_tracker::BlockInterest,
};

pub(crate) struct MapBlockProvider {
    _runner: JoinHandle<Result<()>>,
}

impl MapBlockProvider {
    pub(crate) fn new(
        request_receiver: mpsc::UnboundedReceiver<BlockInterest>,
        block_sender: mpsc::UnboundedSender<WorldUpdate>,
        storage: Option<Box<dyn WorldStorage>>,
        generator: Option<Box<dyn WorldGenerator>>,
        content_map: Arc<HashMap<Box<[u8]>, ContentId>>,
    ) -> Self {
        let runner = thread::spawn(move || {
            Self::run(
                request_receiver,
                &block_sender,
                storage,
                generator,
                content_map,
            )
            .inspect_err(|error| {
                error!("router exited with error: {error}");
            })
        });

        Self { _runner: runner }
    }

    fn run(
        mut request_receiver: mpsc::UnboundedReceiver<BlockInterest>,
        block_sender: &mpsc::UnboundedSender<WorldUpdate>,
        mut storage: Option<Box<dyn WorldStorage>>,
        mut generator: Option<Box<dyn WorldGenerator>>,
        content_map: Arc<HashMap<Box<[u8]>, ContentId>>,
    ) -> Result<()> {
        'next_request: while let Some(message) = request_receiver.blocking_recv() {
            let BlockInterest {
                player_key: _,
                pos,
                priority: _,
            } = message;

            if let Some(storage) = &mut storage {
                if let Some(block) = storage.load_block(pos, Arc::clone(&content_map))? {
                    block_sender.send(WorldUpdate::NewMapBlock(block))?;
                    continue 'next_request;
                }
            }

            if let Some(generator) = &mut generator {
                let block = generator.generate_block(pos);
                block_sender.send(WorldUpdate::NewMapBlock(block))?;
                continue 'next_request;
            }

            trace!("map block {pos} couldn't be obtained from any source");
        }

        Ok(())
    }
}
