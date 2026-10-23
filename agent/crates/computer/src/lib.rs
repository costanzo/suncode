//! Provider-independent Computer Use actions and serialized desktop execution.
//!
//! This crate owns no provider protocol, policy, approval, persistence, SDK, or UI
//! state. [`ComputerRuntime`] implements the core's tool provider contract.

mod accessibility;
mod action;
mod backend;
mod enigo_backend;
mod executor;
mod frame;
mod provider;
mod runtime;

pub use accessibility::{
    platform_backend as platform_accessibility_backend,
    platform_permission as platform_accessibility_permission, AccessibilityBackend,
    AccessibilityNode, AccessibilityQuery, AccessibilityReport, AccessibilityRequest,
    AccessibilityRuntimeInfo, AccessibilityScope, AccessibilitySnapshot, AccessibilityTarget,
    InputRect, Role, StateFlags, Truncation, OUTLINE_MARKER,
};
pub use action::{
    ComputerAction, KeyChord, KeyModifier, MouseButton, ScrollDirection, MAX_ACTION_DURATION,
    MAX_KEY_REPEAT,
};
pub use backend::{
    BackendRuntimeInfo, ComputerBackend, KeyState, MouseButtonState, PermissionState,
};
pub use enigo_backend::EnigoBackend;
pub use executor::{
    ActionOutcome, BatchItem, ComputerError, ComputerExecutor, ComputerResult, FrameSnapshot,
    HALT_MESSAGE,
};
pub use frame::{
    ComputerFrame, DisplayGeometry, PixelPoint, PixelRegion, MAX_PROVIDER_IMAGE_EDGE,
    MAX_PROVIDER_IMAGE_PIXELS, MAX_PROVIDER_PNG_BYTES,
};
pub use provider::member;
pub use runtime::{ComputerRuntime, ComputerRuntimeInfo};
