//! Rust-owned Browser Use for the CEF browser instance.
//!
//! Avalonia owns CEF and exposes a localhost remote debugging endpoint. This crate
//! connects to that already-running endpoint through `chromiumoxide`; it never
//! launches Chromium or owns browser UI state.
//!
//! - `tools`: the model-facing `browser_*` tool contract.
//! - `runtime`: per-project lifecycle, status, and managed browser data.
//! - `cdp`: the CDP transport used by the runtime.
//!
//! The agent embeds [`BrowserRuntime`] and routes `browser_*` tool calls to it.

mod action;
mod cdp;
mod runtime;
pub mod tools;

pub use cdp::error;
pub use runtime::{
    BrowserHostCallback, BrowserInstallationState, BrowserRuntime, BrowserRuntimeInfo,
    BrowserRuntimeState, BrowserVisibilityCapability,
};
