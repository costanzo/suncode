//! Rust-owned Chromium DevTools Protocol client for the CEF browser instance.
//!
//! Avalonia owns CEF and exposes a localhost remote debugging endpoint. This crate
//! connects to that already-running endpoint through `chromiumoxide`; it never
//! launches Chromium or owns browser UI state.
//!
//! Layering: [`CdpSession`] owns the connection and parses each wire request into
//! a typed command; `actions` executes commands against the page; `locator`
//! turns semantic element targets into DOM lookups.

mod actions;
mod command;
mod discovery;
pub mod error;
mod launch;
mod locator;
mod session;

pub use launch::{CdpLaunch, DEFAULT_CDP_PORT, PROTOCOL_VERSION};
pub use session::CdpSession;
