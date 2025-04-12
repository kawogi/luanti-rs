use luanti_core::map::{MapBlockPos, MapNodeIndex, MapNodePos};
use luanti_protocol::types::{MapBlock, MapNode, MapNodesBulk, NodeMetadataList};

use super::WorldGenerator;

pub(crate) struct MapgenFlat;

impl WorldGenerator for MapgenFlat {
    fn generate_map_block(&self, map_block_pos: MapBlockPos) -> MapBlock {
        let nodes = std::array::from_fn(|index| {
            let node_pos = map_block_pos.node_pos(MapNodeIndex::from(index));
            let content = match node_pos.0.y {
                i16::MIN..0 => 10, // demo node
                _ => 126,          // CONTENT_AIR
            };
            MapNode {
                param0: content,
                param1: 255,
                param2: 255,
            }
        });

        MapBlock {
            is_underground: MapNodePos::from(map_block_pos).0.y < 0,
            day_night_differs: false,
            generated: true,
            lighting_complete: Some(0xffff),
            nodes: MapNodesBulk { nodes },
            node_metadata: NodeMetadataList { metadata: vec![] },
        }
    }
}
