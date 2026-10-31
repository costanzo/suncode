//! Plain data types shared across SunCode crates.
//!
//! A type belongs here only when it is pure data used by at least two crates.
//! This crate holds no traits, IO, or runtime state and depends on `serde`
//! alone, so any crate can use it without pulling in an async runtime,
//! database, or HTTP stack.

mod message;
mod tool;
mod usage;

pub use message::{ContentPart, Message, ToolCall};
pub use tool::ToolDefinition;
pub use usage::Usage;

pub mod shell;
