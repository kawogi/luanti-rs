//! Contains the `WorldStorage` trait and some implementations thereof.

use super::WorldBlock;
use anyhow::Result;
use luanti_core::MapBlockPos;

pub mod dummy;
pub mod minetestworld;

/// This trait needs to be implemented by a storage provider for map data
pub trait WorldStorage: Send + Sync {
    /// Stores a given world block containing a map block.
    ///
    /// # Errors
    ///
    /// Returns an error if the block could be stored
    fn store_block(&mut self, map_block: &WorldBlock) -> Result<()>;
    /// Tries to load a world block containing a map block from the storage.
    /// Returns `None`, if the requested block doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the block could be retrieved for other reasons.
    fn load_block(&self, pos: MapBlockPos) -> Result<Option<WorldBlock>>;
}
