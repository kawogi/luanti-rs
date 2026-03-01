//! Contains a single `MapNode` which is the fundamental building block (voxel, cube) of a Luanti
//! world.

use crate::{content_id::ContentId, map_block::MapBlockPos};
use glam::{I16Vec3, U8Vec3, U16Vec3, UVec3};

/// A single map node with its parameters.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct MapNode {
    /// describes the _material_ this node is made of.
    pub content_id: ContentId,
    /// content-dependent auxiliary parameter 1 describing the properties of this node
    pub param1: u8,
    /// content-dependent auxiliary parameter 2 describing the properties of this node
    pub param2: u8,
}

/// The coordinates of a single node within the world
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MapNodePos(pub I16Vec3);

impl MapNodePos {
    /// Position of the map node at the world's center
    pub const ZERO: Self = Self(I16Vec3::ZERO);
    /// Position of the map node with the lowest possible coordinates.
    pub const MIN: Self = Self(I16Vec3::MIN);
    /// Position of the map node with the highest possible coordinates.
    pub const MAX: Self = Self(I16Vec3::MAX);

    /// Splits a map node position into its map block position and its index therein.
    #[must_use]
    pub fn split_index(self) -> (MapBlockPos, MapNodeIndex) {
        (self.block_pos(), self.index())
    }

    /// Returns the position of the map block which contains this node.
    #[must_use]
    pub const fn block_pos(self) -> MapBlockPos {
        MapBlockPos::for_node(self)
    }

    /// Returns the position of the map block which contains this node.
    #[must_use]
    pub fn index(self) -> MapNodeIndex {
        MapNodeIndex::for_node(self)
    }
}

impl From<MapNodePos> for I16Vec3 {
    fn from(value: MapNodePos) -> Self {
        value.0
    }
}

/// The index of a map node within its map block.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MapNodeIndex(u16);

impl MapNodeIndex {
    /// Bit indices of the individual coordinates within the index.
    const SHIFT: UVec3 = UVec3::new(0, MapBlockPos::SIZE_BITS, 2 * MapBlockPos::SIZE_BITS);
    /// Bit masks of the individual coordinates when they've been aligned towards the least significant bit.
    const MASK: U16Vec3 = U16Vec3::splat(MapBlockPos::SIZE_MASK);
    /// index of the first node within a block (0, 0, 0)
    pub const MIN: Self = Self(0);
    /// index of the last node within a block (15, 15, 15)
    pub const MAX: Self = Self(MapBlockPos::NODE_COUNT - 1);

    /// Converts a given node position into the index within its containing map block.
    #[must_use]
    pub fn for_node(node_pos: MapNodePos) -> Self {
        // only retain the lower-most bits of the coordinates and align them next to each other
        let vec = (node_pos.0.as_u16vec3() & Self::MASK) << Self::SHIFT;
        Self(vec.x | vec.y | vec.z)
    }
}

impl From<MapNodeIndex> for U16Vec3 {
    fn from(value: MapNodeIndex) -> Self {
        // right-align the bits of all three coordinates and mask off excessive high-bits
        (U16Vec3::splat(value.0) >> MapNodeIndex::SHIFT) & MapNodeIndex::MASK
    }
}

impl From<MapNodeIndex> for U8Vec3 {
    fn from(value: MapNodeIndex) -> Self {
        U16Vec3::from(value).as_u8vec3()
    }
}

impl From<MapNodeIndex> for UVec3 {
    fn from(value: MapNodeIndex) -> Self {
        U16Vec3::from(value).as_uvec3()
    }
}

impl From<MapNodeIndex> for u16 {
    fn from(value: MapNodeIndex) -> Self {
        value.0
    }
}

impl From<MapNodeIndex> for usize {
    fn from(value: MapNodeIndex) -> Self {
        value.0.into()
    }
}

impl From<usize> for MapNodeIndex {
    fn from(value: usize) -> Self {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "truncation is the expected behavior"
        )]
        Self((value as u16) & MapBlockPos::NODE_COUNT_MASK)
    }
}

impl From<u16> for MapNodeIndex {
    fn from(value: u16) -> Self {
        Self(value & MapBlockPos::NODE_COUNT_MASK)
    }
}
