pub(crate) mod flat;

use luanti_core::MapBlockPos;

use super::WorldBlock;

pub(crate) trait WorldGenerator: Send + Sync {
    fn generate_block(&self, pos: MapBlockPos) -> WorldBlock;
}
