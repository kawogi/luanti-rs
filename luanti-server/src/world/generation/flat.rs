use luanti_core::{
    content_id::ContentId,
    map::{MapBlockPos, MapNodeIndex, MapNodePos},
    node::MapNode,
};
use luanti_protocol::types::{MapNodesBulk, NodeMetadataList, TransferrableMapBlock};

use super::WorldGenerator;

pub(crate) struct MapgenFlat;

impl WorldGenerator for MapgenFlat {
    fn generate_map_block(&self, map_block_pos: MapBlockPos) -> TransferrableMapBlock {
        let nodes = std::array::from_fn(|index| {
            let node_pos = map_block_pos.node_pos(MapNodeIndex::from(index));
            let content = match node_pos.0.y {
                i16::MIN..0 => ContentId(10), // demo node
                _ => ContentId::AIR,
            };
            MapNode {
                content_id: content,
                param1: 255,
                param2: 255,
            }
        });

        TransferrableMapBlock {
            is_underground: MapNodePos::from(map_block_pos).0.y < 0,
            day_night_differs: false,
            generated: true,
            lighting_complete: Some(0xffff),
            nodes: MapNodesBulk { nodes },
            node_metadata: NodeMetadataList { metadata: vec![] },
        }
    }
}
