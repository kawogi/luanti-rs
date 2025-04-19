use super::WorldGenerator;
use crate::world::WorldBlock;
use luanti_core::{ContentId, MapBlockNodes, MapBlockPos, MapNode, MapNodeIndex, MapNodePos};

pub(crate) struct MapgenFlat;

impl WorldGenerator for MapgenFlat {
    fn generate_block(&self, map_block_pos: MapBlockPos) -> WorldBlock {
        let nodes = std::array::from_fn(|index| {
            let node_pos = map_block_pos.node_pos(MapNodeIndex::from(index));
            let content = match node_pos.0.y {
                i16::MIN..0 => {
                    if (node_pos.0.x & 0x1) == (node_pos.0.z & 0x1) {
                        ContentId(10)
                    } else {
                        ContentId::UNKNOWN
                    }
                } // demo node
                _ => ContentId::AIR,
            };
            MapNode {
                content_id: content,
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
