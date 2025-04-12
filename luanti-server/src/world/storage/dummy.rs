use super::WorldStorage;
use crate::world::WorldBlock;
use anyhow::Result;
use luanti_core::map::MapBlockPos;

/// A world storage provider which actually never stores or loads anything.
/// This is useful for temporary throwaway worlds and for mapgen tests.
pub(crate) struct DummyStorage;

impl WorldStorage for DummyStorage {
    fn store_block(&mut self, _map_block: &WorldBlock) -> Result<()> {
        Ok(())
    }

    fn load_block(&self, _pos: MapBlockPos) -> Result<Option<WorldBlock>> {
        Ok(None)
    }
}
