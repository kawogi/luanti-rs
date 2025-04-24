use std::{path::Path, sync::Arc};

use super::WorldStorage;
use crate::{ContentIdMap, world::WorldBlock};
use anyhow::Result;
use log::{debug, info};
use luanti_core::{ContentId, MapBlockNodes, MapBlockPos, MapNode, MapNodePos};
use minetestworld::Position;

/// A world storage provider which uses the `minetestworld` crate.
pub(crate) struct MinetestworldStorage {
    world: minetestworld::World,
    content_id_map: Arc<ContentIdMap>,
}

impl MinetestworldStorage {
    pub(crate) async fn new(
        path: impl AsRef<Path>,
        content_id_map: Arc<ContentIdMap>,
    ) -> Result<Self> {
        info!("loading world from {path}", path = path.as_ref().display());
        let world = minetestworld::World::open(path);
        for (key, value) in world.get_world_metadata().await? {
            debug!("world metadata: {key}: {value}");
        }

        Ok(MinetestworldStorage {
            world,
            content_id_map,
        })
    }
}

impl WorldStorage for MinetestworldStorage {
    fn store_block(&mut self, _map_block: &WorldBlock) -> Result<()> {
        Ok(())
    }

    fn load_block(&self, map_block_pos: MapBlockPos) -> Result<Option<WorldBlock>> {
        let (x, y, z) = map_block_pos.vec().into();
        let map_data: Result<_> = pollster::block_on(async {
            let map_data = self.world.get_map_data().await?;
            Ok(map_data.get_mapblock(Position::new(x, y, z)).await?)
        });
        let map_block = map_data?;

        let mut id_map = Vec::with_capacity(map_block.name_id_mappings.len());
        for (id, name) in map_block.name_id_mappings {
            let global_id = self.content_id_map[name.as_slice()];
            let index = usize::from(id);
            if let Some(slot) = id_map.get_mut(index) {
                *slot = global_id;
            } else {
                id_map.resize(index, ContentId::UNKNOWN);
                id_map.push(global_id);
            }
        }

        #[expect(
            clippy::indexing_slicing,
            reason = "block size is known at compile-time"
        )]
        let nodes = std::array::from_fn(|index| MapNode {
            content_id: id_map[usize::from(map_block.param0[index])],
            param1: map_block.param1[index],
            param2: map_block.param2[index],
        });

        Ok(Some(WorldBlock {
            version: 0,
            pos: map_block_pos,
            is_underground: MapNodePos::from(map_block_pos).0.y < 0,
            day_night_differs: false,
            lighting_complete: 0xffff,
            nodes: MapBlockNodes(nodes),
            metadata: vec![],
        }))
    }
}
