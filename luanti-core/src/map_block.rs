//! Contains all kinds of map position primitives and conversions between them.

use std::{
    fmt::{self, Display},
    ops::{Index, IndexMut},
};

use glam::{I16Vec3, UVec3};

use crate::map_node::{MapNode, MapNodeIndex, MapNodePos};

/// Contains all `MapNodes` of a single map block.
#[derive(Clone)]
pub struct MapBlockNodes(pub [MapNode; MapBlockPos::NODE_COUNT as usize]);

impl Index<MapNodeIndex> for MapBlockNodes {
    type Output = MapNode;

    fn index(&self, index: MapNodeIndex) -> &Self::Output {
        #[expect(
            clippy::indexing_slicing,
            reason = "MapNodeIndex by construction is guaranteed to be within bounds"
        )]
        &self.0[usize::from(index)]
    }
}

impl IndexMut<MapNodeIndex> for MapBlockNodes {
    fn index_mut(&mut self, index: MapNodeIndex) -> &mut Self::Output {
        #[expect(
            clippy::indexing_slicing,
            reason = "MapNodeIndex by construction is guaranteed to be within bounds"
        )]
        &mut self.0[usize::from(index)]
    }
}

/// The position of a map block.
/// The position is _not_ measured in world coordinates. It can be viewed as a signed 3D-index,
/// where `(0, 0, 0)` is located at the world's center
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct MapBlockPos(I16Vec3);

impl MapBlockPos {
    /// number of bit shifts to perform in order to convert between map node and map block
    /// coordinates.
    pub const SIZE_BITS: u32 = 4;
    /// Number of map nodes per map blocks in each dimension.
    pub const SIZE: u32 = 1 << Self::SIZE_BITS;
    /// Mask to be used to address the bits of a node coordinate that make up the the position
    /// within their block.
    pub const SIZE_MASK: u32 = Self::SIZE - 1;
    /// number of map nodes within a single block
    pub const NODE_COUNT: u32 = Self::SIZE * Self::SIZE * Self::SIZE;
    /// mask to be used to make a number a valid node index by wrapping around
    pub const NODE_COUNT_MASK: u32 = Self::NODE_COUNT - 1;

    /// Position of the map block at the world's center
    pub const ZERO: Self = Self(I16Vec3::ZERO);
    /// Position of the map block with the lowest possible coordinates.
    pub const MIN: Self = Self::for_node(MapNodePos::MIN);
    /// Position of the map block with the highest possible coordinates.
    pub const MAX: Self = Self::for_node(MapNodePos::MAX);

    /// Creates a new `MapBlockPos` as long as the resulting position would fit into the world.
    /// Returns `None` otherwise.
    #[must_use]
    pub fn new(position: I16Vec3) -> Option<Self> {
        (position.cmpge(Self::MIN.0).all() && position.cmple(Self::MAX.0).all())
            .then_some(Self(position))
    }

    /// Converts a given node position into that of the containing map block.
    #[must_use]
    pub const fn for_node(node_pos: MapNodePos) -> Self {
        //TODO(kawogi) check whether is is accurate; maybe the origin is located in the center of a block
        Self(I16Vec3 {
            x: node_pos.0.x >> MapBlockPos::SIZE_BITS,
            y: node_pos.0.y >> MapBlockPos::SIZE_BITS,
            z: node_pos.0.z >> MapBlockPos::SIZE_BITS,
        })
    }

    /// Check whether the given map node is located within this map block
    #[must_use]
    pub fn contains(self, node_pos: MapNodePos) -> bool {
        Self::for_node(node_pos) == self
    }

    /// returns the map node position for a certain map node in this map block
    #[must_use]
    pub fn node_pos(self, index: MapNodeIndex) -> MapNodePos {
        MapNodePos(MapNodePos::from(self).0 + UVec3::from(index).as_i16vec3())
    }
}

impl Display for MapBlockPos {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // use double square brackets to indicate that a map block is bigger than a single map node
        write!(formatter, "[[{}, {}, {}]]", self.0.x, self.0.y, self.0.z)
    }
}

impl From<MapBlockPos> for MapNodePos {
    fn from(value: MapBlockPos) -> Self {
        Self(value.0 << MapBlockPos::SIZE_BITS)
    }
}

impl From<MapBlockPos> for I16Vec3 {
    fn from(value: MapBlockPos) -> Self {
        value.0
    }
}
