//! Chromium DevTools Protocol transport to the host-owned CEF page.
//!
//! [`CdpSession`] owns the connection; `actions` executes typed
//! [`BrowserAction`](crate::action::BrowserAction)s against the page; `locator` turns
//! semantic element targets into DOM lookups.

mod actions;
mod discovery;
pub mod error;
mod launch;
mod locator;
mod session;

pub use launch::{CdpLaunch, PageTarget, DEFAULT_CDP_PORT, PROTOCOL_VERSION};
pub use session::CdpSession;
