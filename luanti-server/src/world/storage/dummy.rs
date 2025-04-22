use std::{collections::HashMap, sync::Arc};

use super::WorldStorage;
use crate::world::WorldBlock;
use anyhow::Result;
use luanti_core::{ContentId, MapBlockPos};

/// A world storage provider which actually never stores or loads anything.
/// This is useful for temporary throwaway worlds and for mapgen tests.
pub(crate) struct DummyStorage;

impl WorldStorage for DummyStorage {
    fn store_block(&mut self, _map_block: &WorldBlock) -> Result<()> {
        Ok(())
    }

    fn load_block(
        &self,
        _pos: MapBlockPos,
        _content_map: Arc<HashMap<Box<[u8]>, ContentId>>,
    ) -> Result<Option<WorldBlock>> {
        Ok(None)
    }
}
