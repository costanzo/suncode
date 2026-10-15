//! Audited in-process operations for the SunCode runtime.
//!
//! - `operations`: the public `Operations` API.
//! - `dispatch`: the explicit method-to-tool table behind `execute_in_project`.
//! - `context`: per-call execution context and network settings.
//! - `scope`: project roots and project-relative path containment.
//! - `journal`: idempotency records for mutating operations.
//! - `read`, `write`, `edit`, `search`, `checkpoint`, `process`, `webfetch`: tools.
//! - `project_view`: bounded directory listing and file viewing for clients.

mod arguments;
mod artifacts;
mod checkpoint;
mod context;
mod dispatch;
mod edit;
mod error;
mod journal;
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

pub use context::ProcessOutputCallback;
pub use operations::{CertificatePath, Operations, ProxyConfiguration};
pub use process::BackgroundProcess;

#[cfg(test)]
mod tests;
