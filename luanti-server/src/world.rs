pub(crate) mod content_id_map;
pub(crate) mod generation;
pub(crate) mod map_block_provider;
pub(crate) mod map_block_router;
pub(crate) mod media_registry;
pub(crate) mod priority;
pub(crate) mod storage;
pub(crate) mod view_tracker;

use luanti_core::{MapBlockNodes, MapBlockPos, MapNodeIndex};
use luanti_protocol::types::NodeMetadata;
use storage::WorldStorage;

/// A single Luanti world with all items, nodes, media, etc.
struct World {
    /// The user-facing name of the world
    name: String,
    /// This is where the world is being stored.
    storage: Box<dyn WorldStorage>,
}

/// This is a wrapper for a raw `MapBlock` which contains extra information that simplifies handling
/// in the API.
#[derive(Clone)]
pub(crate) struct WorldBlock {
    /// number of updates this `MapBlock` has received
    /// This can be used
    pub(crate) version: u64,
    /// Location within the world
    pub(crate) pos: MapBlockPos,

    /// Should be set to `false` if there will be no light obstructions above the block.
    /// If/when sunlight of a block is updated and there is no block above it, this value is checked
    /// for determining whether sunlight comes from the top.
    pub(crate) is_underground: bool,

    /// Whether the lighting of the block is different on day and night.
    /// Only blocks that have this bit set are updated when day transforms to night.
    pub(crate) day_night_differs: bool,

    /// This contains 12 flags, each of them corresponds to a direction.
    ///
    /// Indicates if the light is correct at the sides of a map block.
    /// Lighting may not be correct if the light changed, but a neighbor
    /// block was not loaded at that time.
    /// If these flags are false, Luanti will automatically recompute light
    /// when both this block and its required neighbor are loaded.
    ///
    /// The bit order is:
    ///
    /// - bits 15-12: nothing,  nothing,  nothing,  nothing,
    /// - bits 11-6: night X-, night Y-, night Z-, night Z+, night Y+, night X+,
    /// - bits 5-0: day X-,   day Y-,   day Z-,   day Z+,   day Y+,   day X+.
    ///
    /// Where 'day' is for the day light bank, 'night' is for the night light bank.
    /// The 'nothing' bits should be always set, as they will be used
    /// to indicate if direct sunlight spreading is finished.
    ///
    /// Example: if the block at `(0, 0, 0)` has `lighting_complete = 0b1111111111111110`,
    ///  Luanti will correct lighting in the day light bank when the block at
    ///  `(1, 0, 0)` is also loaded.
    pub(crate) lighting_complete: u16,

    pub(crate) nodes: MapBlockNodes,

    pub(crate) metadata: Vec<(MapNodeIndex, NodeMetadata)>,
}

/// A value of this type describes a change to the world.
#[derive(Clone)]
pub(crate) enum WorldUpdate {
    /// A new map block was made available. This usually means that this block has just been
    /// generated or loaded from storage.
    ///
    /// This may also be created for an existing map block that is _new_ to a certain player.
    NewMapBlock(WorldBlock),
}

impl std::fmt::Debug for WorldUpdate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NewMapBlock(world_block) => write!(formatter, "NewMapBlock: {}", world_block.pos),
        }
    }
}
