//! Luanti server implemented in Rust
// #![expect(clippy::expect_used, reason = "//TODO improve error handling")]

#![expect(
    clippy::todo,
    clippy::expect_used,
    reason = "//TODO remove before completion of the prototype"
)]

pub mod authentication;
mod client_connection;
pub mod server;
pub mod world;

use world::content_id_map::ContentIdMap;
use world::media_registry::MediaRegistry;
