//! Contains a single `MapNode` which is the fundamental building block of a Luanti world.

use crate::content_id::ContentId;

/// A single map block with it's parameters
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct MapNode {
    /// describes the _material_ this node is made of.
    pub content_id: ContentId,
    /// content-dependent auxiliary parameter 1 describing the properties of this node
    pub param1: u8,
    /// content-dependent auxiliary parameter 2 describing the properties of this node
    pub param2: u8,
}
