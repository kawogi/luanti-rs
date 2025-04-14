pub(crate) mod flat;

use luanti_core::map::MapBlockPos;
use luanti_protocol::types::TransferrableMapBlock;

pub(crate) trait WorldGenerator {
    fn generate_map_block(&self, pos: MapBlockPos) -> TransferrableMapBlock;
}
