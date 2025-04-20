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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MapBlockPos(I16Vec3);

impl MapBlockPos {
    /// number of bit shifts to perform in order to convert between map node and map block
    /// coordinates.
    pub const SIZE_BITS: u32 = 4;
    /// Number of map nodes per map blocks in each dimension.
    pub const SIZE: u16 = 1 << Self::SIZE_BITS;
    /// Mask to be used to address the bits of a node coordinate that make up the the position
    /// within their block.
    pub const SIZE_MASK: u16 = Self::SIZE - 1;
    /// number of map nodes within a single block
    pub const NODE_COUNT: u16 = Self::SIZE * Self::SIZE * Self::SIZE;
    /// mask to be used to make a number a valid node index by wrapping around
    pub const NODE_COUNT_MASK: u16 = Self::NODE_COUNT - 1;

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
        Self::for_vec(node_pos.0)
    }

    /// Converts a given world position into that of the containing map block.
    ///
    /// `for_node` is preferred in most cases but sometimes we only have a raw vector and it would
    /// be unnecessary to wrap that in a `MapNodePos`.
    #[must_use]
    pub const fn for_vec(pos: I16Vec3) -> Self {
        Self(I16Vec3 {
            x: pos.x >> MapBlockPos::SIZE_BITS,
            y: pos.y >> MapBlockPos::SIZE_BITS,
            z: pos.z >> MapBlockPos::SIZE_BITS,
        })
    }

    /// returns the inner position vector of this block which is measured in block steps from the
    /// origin
    #[must_use]
    pub fn vec(self) -> I16Vec3 {
        self.0
    }

    /// Returns the map block position with a given displacement.
    ///
    /// e.g. `pos.checked_add(IVec3::new(0, 1, 0))` returns the block above (`Y + 1`) the current
    /// one.
    ///
    /// Returns `None` if the resulting block would be located out of this map.
    #[must_use]
    pub fn checked_add(self, delta: I16Vec3) -> Option<Self> {
        self.0.checked_add(delta).and_then(Self::new)
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
        value.vec()
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, reason = "ok for tests")]

    use super::*;

    #[test]
    fn test_map_block_pos_new() {
        let pos0 = MapBlockPos::new(I16Vec3::new(0, 0, 0)).unwrap();
        assert_eq!(pos0.vec(), I16Vec3::new(0, 0, 0));
        assert_eq!(pos0, MapBlockPos::ZERO);

        let pos_max_x = MapBlockPos::new(I16Vec3::new(2047, 0, 0)).unwrap();
        assert_eq!(pos_max_x.vec(), I16Vec3::new(2047, 0, 0));

        let pos_max_y = MapBlockPos::new(I16Vec3::new(0, 2047, 0)).unwrap();
        assert_eq!(pos_max_y.vec(), I16Vec3::new(0, 2047, 0));

        let pos_max_z = MapBlockPos::new(I16Vec3::new(0, 0, 2047)).unwrap();
        assert_eq!(pos_max_z.vec(), I16Vec3::new(0, 0, 2047));

        let pos_max = MapBlockPos::new(I16Vec3::new(2047, 2047, 2047)).unwrap();
        assert_eq!(pos_max.vec(), I16Vec3::new(2047, 2047, 2047));
        assert_eq!(pos_max, MapBlockPos::MAX);

        let pos_min_x = MapBlockPos::new(I16Vec3::new(-2048, 0, 0)).unwrap();
        assert_eq!(pos_min_x.vec(), I16Vec3::new(-2048, 0, 0));

        let pos_min_y = MapBlockPos::new(I16Vec3::new(0, -2048, 0)).unwrap();
        assert_eq!(pos_min_y.vec(), I16Vec3::new(0, -2048, 0));

        let pos_min_z = MapBlockPos::new(I16Vec3::new(0, 0, -2048)).unwrap();
        assert_eq!(pos_min_z.vec(), I16Vec3::new(0, 0, -2048));

        let pos_min = MapBlockPos::new(I16Vec3::new(-2048, -2048, -2048)).unwrap();
        assert_eq!(pos_min.vec(), I16Vec3::new(-2048, -2048, -2048));
        assert_eq!(pos_min, MapBlockPos::MIN);

        assert!(MapBlockPos::new(I16Vec3::new(2048, 2047, 2047)).is_none());
        assert!(MapBlockPos::new(I16Vec3::new(2047, 2048, 2047)).is_none());
        assert!(MapBlockPos::new(I16Vec3::new(2047, 2047, 2048)).is_none());
        assert!(MapBlockPos::new(I16Vec3::new(2048, 2048, 2048)).is_none());

        assert!(MapBlockPos::new(I16Vec3::new(-2049, -2048, -2048)).is_none());
        assert!(MapBlockPos::new(I16Vec3::new(-2048, -2049, -2048)).is_none());
        assert!(MapBlockPos::new(I16Vec3::new(-2048, -2048, -2049)).is_none());
        assert!(MapBlockPos::new(I16Vec3::new(-2049, -2049, -2049)).is_none());

        assert!(MapBlockPos::new(I16Vec3::new(i16::MAX, 0, 0)).is_none());
        assert!(MapBlockPos::new(I16Vec3::new(0, i16::MAX, 0)).is_none());
        assert!(MapBlockPos::new(I16Vec3::new(0, 0, i16::MAX)).is_none());
        assert!(MapBlockPos::new(I16Vec3::MAX).is_none());

        assert!(MapBlockPos::new(I16Vec3::new(i16::MIN, 0, 0)).is_none());
        assert!(MapBlockPos::new(I16Vec3::new(0, i16::MIN, 0)).is_none());
        assert!(MapBlockPos::new(I16Vec3::new(0, 0, i16::MIN)).is_none());
        assert!(MapBlockPos::new(I16Vec3::MIN).is_none());
    }

    #[test]
    fn test_for_pos() {
        assert_eq!(MapBlockPos::for_vec(I16Vec3::ZERO), MapBlockPos::ZERO);
        assert_eq!(MapBlockPos::for_vec(I16Vec3::MAX), MapBlockPos::MAX);
        assert_eq!(MapBlockPos::for_vec(I16Vec3::MIN), MapBlockPos::MIN);
        assert_eq!(
            MapBlockPos::for_vec(I16Vec3::new(15, 15, 15)).vec(),
            I16Vec3::new(0, 0, 0)
        );
        assert_eq!(
            MapBlockPos::for_vec(I16Vec3::new(16, 15, 15)).vec(),
            I16Vec3::new(1, 0, 0)
        );
        assert_eq!(
            MapBlockPos::for_vec(I16Vec3::new(15, 16, 15)).vec(),
            I16Vec3::new(0, 1, 0)
        );
        assert_eq!(
            MapBlockPos::for_vec(I16Vec3::new(15, 15, 16)).vec(),
            I16Vec3::new(0, 0, 1)
        );
        assert_eq!(
            MapBlockPos::for_vec(I16Vec3::new(16, 16, 16)).vec(),
            I16Vec3::new(1, 1, 1)
        );
        assert_eq!(
            MapBlockPos::for_vec(I16Vec3::new(-1, 0, 0)).vec(),
            I16Vec3::new(-1, 0, 0)
        );
        assert_eq!(
            MapBlockPos::for_vec(I16Vec3::new(0, -1, 0)).vec(),
            I16Vec3::new(0, -1, 0)
        );
        assert_eq!(
            MapBlockPos::for_vec(I16Vec3::new(0, 0, -1)).vec(),
            I16Vec3::new(0, 0, -1)
        );
        assert_eq!(
            MapBlockPos::for_vec(I16Vec3::new(-1, -1, -1)).vec(),
            I16Vec3::new(-1, -1, -1)
        );
    }

    #[test]
    fn node_pos() {
        assert_eq!(
            MapBlockPos::ZERO.node_pos(MapNodeIndex::MIN),
            MapNodePos::ZERO
        );
        assert_eq!(
            MapBlockPos::MAX.node_pos(MapNodeIndex::MAX),
            MapNodePos::MAX
        );
        assert_eq!(
            MapBlockPos::MIN.node_pos(MapNodeIndex::MIN),
            MapNodePos::MIN
        );
        {
            let node_pos = MapNodePos(I16Vec3::new(i16::MAX, 0, 0));
            let (block_pos, node_index) = node_pos.split_index();
            assert!(block_pos.contains(node_pos));
            assert_eq!(node_pos, block_pos.node_pos(node_index));
        }
        {
            let node_pos = MapNodePos(I16Vec3::new(0, i16::MAX, 0));
            let (block_pos, node_index) = node_pos.split_index();
            assert!(block_pos.contains(node_pos));
            assert_eq!(node_pos, block_pos.node_pos(node_index));
        }
        {
            let node_pos = MapNodePos(I16Vec3::new(0, 0, i16::MAX));
            let (block_pos, node_index) = node_pos.split_index();
            assert!(block_pos.contains(node_pos));
            assert_eq!(node_pos, block_pos.node_pos(node_index));
        }
        {
            let node_pos = MapNodePos(I16Vec3::new(i16::MIN, 0, 0));
            let (block_pos, node_index) = node_pos.split_index();
            assert!(block_pos.contains(node_pos));
            assert_eq!(node_pos, block_pos.node_pos(node_index));
        }
        {
            let node_pos = MapNodePos(I16Vec3::new(0, i16::MIN, 0));
            let (block_pos, node_index) = node_pos.split_index();
            assert!(block_pos.contains(node_pos));
            assert_eq!(node_pos, block_pos.node_pos(node_index));
        }
        {
            let node_pos = MapNodePos(I16Vec3::new(0, 0, i16::MIN));
            let (block_pos, node_index) = node_pos.split_index();
            assert!(block_pos.contains(node_pos));
            assert_eq!(node_pos, block_pos.node_pos(node_index));
        }
    }

    #[test]
    fn test_checked_add() {
        assert_eq!(
            MapBlockPos::ZERO.checked_add(MapBlockPos::MAX.vec()),
            Some(MapBlockPos::MAX)
        );
        assert_eq!(
            MapBlockPos::ZERO.checked_add(MapBlockPos::MIN.vec()),
            Some(MapBlockPos::MIN)
        );
        assert!(
            MapBlockPos::MAX
                .checked_add(I16Vec3::new(1, 0, 0))
                .is_none()
        );
        assert!(
            MapBlockPos::MAX
                .checked_add(I16Vec3::new(0, 1, 0))
                .is_none()
        );
        assert!(
            MapBlockPos::MAX
                .checked_add(I16Vec3::new(0, 0, 1))
                .is_none()
        );
        assert!(
            MapBlockPos::MAX
                .checked_add(I16Vec3::new(1, 1, 1))
                .is_none()
        );
        assert!(
            MapBlockPos::MIN
                .checked_add(I16Vec3::new(-1, 0, 0))
                .is_none()
        );
        assert!(
            MapBlockPos::MIN
                .checked_add(I16Vec3::new(0, -1, 0))
                .is_none()
        );
        assert!(
            MapBlockPos::MIN
                .checked_add(I16Vec3::new(0, 0, -1))
                .is_none()
        );
        assert!(
            MapBlockPos::MIN
                .checked_add(I16Vec3::new(-1, -1, -1))
                .is_none()
        );
    }
}
