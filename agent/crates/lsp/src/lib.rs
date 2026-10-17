//! Bounded local-stdio Language Server Protocol client.
//!
//! The crate owns protocol framing and process lifecycle only. Project authority,
//! persistence, model tool semantics, and client DTOs remain outside this boundary.

mod capabilities;
mod client;
mod config;
mod environment;
mod error;
mod framing;
mod provider;
mod root;
mod runtime;
mod spec;

pub use capabilities::{PositionEncoding, ServerCapabilities};
pub use client::{connect, Client};
pub use config::ConnectionConfig;
pub use root::{match_root_marker, validate_root_marker};
pub use runtime::{
    is_lsp_tool, LanguageServerRuntimeState, LanguageServerRuntimeStatus, LspRuntime,
};
pub use spec::{ServerSource, ServerSpec};
