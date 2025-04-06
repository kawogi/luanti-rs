//! Contains all kinds of map position primitives and conversions between them.

use glam::{I16Vec3, U8Vec3, UVec3};

/// The coordinates of a single node within the world
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct MapNodePos(pub I16Vec3);

impl MapNodePos {
    /// Position of the map node at the world's center
    pub const ZERO: Self = Self(I16Vec3::ZERO);
    /// Position of the map node with the lowest possible coordinates.
    pub const MIN: Self = Self(I16Vec3::MIN);
    /// Position of the map node with the highest possible coordinates.
    pub const MAX: Self = Self(I16Vec3::MAX);

    /// Splits a map node position into its map block position and its index wherein.
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

/// The index of a map node within its map block.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct MapNodeIndex(u32);

impl MapNodeIndex {
    /// Bit indices of the individual coordinates within the index.
    const SHIFT: UVec3 = UVec3::new(0, MapBlockPos::SIZE_BITS, 2 * MapBlockPos::SIZE_BITS);
    /// Bit masks of the individual coordinates when they've been aligned towards the least significant bit.
    const MASK: UVec3 = UVec3::splat(MapBlockPos::SIZE_MASK);

    /// Converts a given node position into the index within its containing map block.
    #[must_use]
    pub fn for_node(node_pos: MapNodePos) -> Self {
        // only retain the lower-most bits of the coordinates and align them next to each other
        let vec = (node_pos.0.as_uvec3() & Self::MASK) << Self::SHIFT;
        Self(vec.x | vec.y | vec.z)
    }
}

impl From<MapNodeIndex> for UVec3 {
    fn from(value: MapNodeIndex) -> Self {
        // right-align the bits of all three coordinates and mask off excessive high-bits
        (UVec3::splat(value.0) >> MapNodeIndex::SHIFT) & MapNodeIndex::MASK
    }
}

impl From<MapNodeIndex> for U8Vec3 {
    fn from(value: MapNodeIndex) -> Self {
        UVec3::from(value).as_u8vec3()
    }
}

impl From<MapNodeIndex> for usize {
    fn from(value: MapNodeIndex) -> Self {
        value
            .0
            .try_into()
            .unwrap_or_else(|_| unreachable!("16-bit platforms are unsupported"))
    }
}
