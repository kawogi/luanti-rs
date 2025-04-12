pub(crate) mod generation;
pub(crate) mod storage;

use luanti_core::map::MapBlockPos;
use luanti_protocol::types::MapBlock;
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
struct WorldBlock {
    /// number of updates this `MapBlock` has received
    /// This can be used
    version: u64,
    /// Location within the world
    pos: MapBlockPos,
    /// The actual map block
    map_block: MapBlock,
}
