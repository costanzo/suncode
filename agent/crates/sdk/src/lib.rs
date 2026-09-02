//! Stable Rust SDK facade. Hosts should depend on this crate instead of the core internals.

pub use suncode_agent::*;
pub use suncode_common::BusinessError;
pub use suncode_config::Config;
pub use suncode_data::{ApprovalInput, Store};
pub use suncode_llm::{ModelCapabilities, ModelDescriptor, ModelLimits};
