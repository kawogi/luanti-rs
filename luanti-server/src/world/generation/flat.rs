//! contains `MapgenFlat`

use super::WorldGenerator;
use crate::world::WorldBlock;
use luanti_core::{ContentId, MapBlockNodes, MapBlockPos, MapNode, MapNodeIndex, MapNodePos};

/// Generates a world where all nodes below z=0 are of a given type, while everything above is air.
pub struct MapgenFlat {
    node: ContentId,
}

impl MapgenFlat {
    /// Create a new flat world generator.
    #[must_use]
    pub fn new(node: ContentId) -> Self {
        Self { node }
    }
}

impl WorldGenerator for MapgenFlat {
    fn generate_block(&self, map_block_pos: MapBlockPos) -> WorldBlock {
        let nodes = std::array::from_fn(|index| {
            let node_pos = map_block_pos.node_pos(MapNodeIndex::from(index));
            let content_id = match node_pos.0.y {
                i16::MIN..0 => self.node,
                _ => ContentId::AIR,
            };
            MapNode {
                content_id,
                param1: 255,
                param2: 255,
            }
        });

        WorldBlock {
            version: 0,
            pos: map_block_pos,
            is_underground: MapNodePos::from(map_block_pos).0.y < 0,
            day_night_differs: false,
            lighting_complete: 0xffff,
            nodes: MapBlockNodes(nodes),
            metadata: vec![],
        }
    }
}
