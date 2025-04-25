//! Contains the `WorldGenerator` trait and some implementations thereof.

pub mod flat;

use luanti_core::MapBlockPos;

use super::WorldBlock;

/// This trait is implemented by map generators.
pub trait WorldGenerator: Send + Sync {
    /// generate and return a new `WorldBlock` for the given position.
    fn generate_block(&self, pos: MapBlockPos) -> WorldBlock;
}
