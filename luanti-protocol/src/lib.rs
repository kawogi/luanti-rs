//! Luanti protocol implemented in Rust

#![expect(
    missing_docs,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    reason = "//TODO add documentation"
)]

pub mod peer;
pub mod services;
pub mod wire;

pub use services::client::LuantiClient;
pub use services::conn::LuantiConnection;
pub use services::server::LuantiServer;
pub use wire::audit::audit_on;
pub use wire::command::CommandRef;
pub use wire::types::CommandDirection;
