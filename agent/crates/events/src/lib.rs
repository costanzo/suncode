//! Agent event contracts and in-process fan-out.
//!
//! `payload` defines the session event names and payloads published to
//! clients. `SessionEventHub` fans those events out per session, and
//! `AttentionEventHub` carries cross-session attention notifications. The
//! crate holds no agent, persistence, or provider logic.

mod attention_hub;
pub mod payload;
mod session_hub;

pub use attention_hub::*;
pub use payload::*;
pub use session_hub::*;
