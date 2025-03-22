//! Luanti protocol implemented in Rust

#![expect(
    missing_docs,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    reason = "//TODO add documentation and improve error handling"
)]
#![expect(
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    reason = "//TODO there's some unidiomatic code left"
)]

pub mod commands;
pub mod peer;
pub mod services;
pub mod types;
pub mod wire;

pub use commands::CommandRef;
pub use services::client::LuantiClient;
pub use services::conn::LuantiConnection;
pub use services::server::LuantiServer;
pub use types::CommandDirection;
pub use wire::audit::audit_on;
