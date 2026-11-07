//! Audited in-process operations for the SunCode runtime.
//!
//! - `operations`: the public `Operations` API.
//! - `builtin`: the built-in tools as a `ToolProvider`, including model-argument
//!   translation and the bash hard-deny filter.
//! - `dispatch`: the explicit method-to-tool table behind `execute_in_project`.
//! - `context`: per-call execution context and network settings.
//! - `scope`: project roots and project-relative path containment.
//! - `read`, `write`, `edit`, `search`, `process`, `webfetch`: tools.
//! - `checkpoint`: content-addressed file version and per-turn line diffs.
//! - `project_view`: bounded directory listing and file viewing for clients.

mod arguments;
mod artifacts;
mod builtin;
mod checkpoint;
mod context;
mod dispatch;
mod edit;
mod error;
mod operations;
mod process;
mod project_view;
mod read;
mod scope;
mod search;
mod support;
mod webfetch;
mod write;

pub mod definitions;

pub use builtin::BuiltinTools;
pub use context::ProcessOutputCallback;
pub use checkpoint::{diff, DiffHunk, DiffLine, FileDiff};
pub use operations::{CertificatePath, Operations, ProxyConfiguration};
pub use process::BackgroundProcess;

mod documents;
#[cfg(test)]
mod tests;
