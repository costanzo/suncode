//! Connection parameters for the CEF remote debugging endpoint.

use std::time::Duration;

pub const PROTOCOL_VERSION: u32 = 1;
pub const DEFAULT_CDP_PORT: u16 = 9222;

#[derive(Debug, Clone)]
pub struct CdpLaunch {
    pub port: u16,
    /// Page URL to attach to; `None` attaches to the first available page.
    pub target_url: Option<String>,
    pub startup_timeout: Duration,
    pub request_timeout: Duration,
}

impl CdpLaunch {
    pub(crate) fn endpoint(&self) -> String {
        format!("http://127.0.0.1:{}/", self.port)
    }
}
