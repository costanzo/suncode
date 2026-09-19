//! Provider-independent Computer Use actions and serialized desktop execution.
//!
//! This crate owns no provider protocol, policy, approval, persistence, SDK, or UI state.

mod action;
mod backend;
mod enigo_backend;
mod executor;
mod frame;

pub use action::{
    ComputerAction, KeyChord, KeyModifier, MouseButton, ScrollDirection, MAX_ACTION_DURATION,
    MAX_KEY_REPEAT,
};
pub use backend::{
    BackendRuntimeInfo, ComputerBackend, KeyState, MouseButtonState, PermissionState,
};
pub use enigo_backend::EnigoBackend;
pub use executor::{
    ActionOutcome, BatchItem, ComputerError, ComputerExecutor, ComputerResult, HALT_MESSAGE,
};
pub use frame::{ComputerFrame, DisplayGeometry, PixelPoint, PixelRegion};
