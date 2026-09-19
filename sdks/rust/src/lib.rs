//! Stable, typed Rust SDK facade for the SunCode agent harness.

mod facade;
mod types;

pub use facade::{AgentSdk, SessionEventStream, SessionEventStreamControl, SubscriptionError};
pub use suncode_agent::logging as logging_module;
pub mod events {
    pub use suncode_agent::agent::events::*;
}
pub use events::{AgentEvent, EventPayload as AgentEventPayload, EventType};
pub use types::*;
