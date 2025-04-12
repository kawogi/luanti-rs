pub(crate) mod flat;

use luanti_core::map::MapBlockPos;
use luanti_protocol::types::MapBlock;

pub(crate) trait WorldGenerator {
    fn generate_map_block(&self, pos: MapBlockPos) -> MapBlock;
}
