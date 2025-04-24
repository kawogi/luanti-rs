//! Holds the content id type

use std::num::TryFromIntError;

/// The content id describes the _material_ a `MapNode` is made of.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ContentId(pub u16);

impl Default for ContentId {
    fn default() -> Self {
        Self::IGNORE
    }
}

impl ContentId {
    /// A solid walkable node with the texture `unknown_node.png`.
    ///
    /// For example, used on the client to display unregistered node IDs
    /// (instead of expanding the vector of node definitions each time
    /// such a node is received).
    pub const UNKNOWN: Self = Self(125);

    /// The common material through which the player can walk and which
    /// is transparent to light
    pub const AIR: Self = Self(126);

    /// Ignored node.
    ///
    /// Unloaded chunks are considered to consist of this. Several other
    /// methods return this when an error occurs. Also, during
    /// map generation this means the node has not been set yet.
    ///
    /// Doesn't create faces with anything and is considered being
    /// out-of-map in the game map.
    pub const IGNORE: Self = Self(127);
}

impl From<ContentId> for usize {
    fn from(value: ContentId) -> Self {
        usize::from(value.0)
    }
}

impl TryFrom<usize> for ContentId {
    type Error = TryFromIntError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        value.try_into().map(Self)
    }
}
