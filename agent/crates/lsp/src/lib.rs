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

pub use capabilities::{PositionEncoding, ServerCapabilities};
pub use client::{connect, Client};
pub use config::ConnectionConfig;
