use anyhow::Result;
use luanti_core::MapBlockPos;

use super::WorldBlock;

pub(crate) mod dummy;
pub(crate) mod minetestworld;

pub(crate) trait WorldStorage: Send + Sync {
    /// Stores a given world block containing a map block.
    fn store_block(&mut self, map_block: &WorldBlock) -> Result<()>;
    /// Tries to load a world block containing a map block from the storage.
    /// Returns `None`, if the requested block doesn't exist.
    fn load_block(&self, pos: MapBlockPos) -> Result<Option<WorldBlock>>;
}
