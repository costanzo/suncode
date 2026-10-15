//! Chromium DevTools Protocol transport to the host-owned CEF page.
//!
//! [`CdpSession`] owns the connection and parses each wire request into a typed
//! command; `actions` executes commands against the page; `locator` turns
//! semantic element targets into DOM lookups.

mod actions;
mod command;
mod discovery;
pub mod error;
mod launch;
mod locator;
mod session;

pub use launch::{CdpLaunch, DEFAULT_CDP_PORT, PROTOCOL_VERSION};
pub use session::CdpSession;
