//! Connection parameters for the CEF remote debugging endpoint.

use std::time::Duration;

pub const PROTOCOL_VERSION: u32 = 1;
pub const DEFAULT_CDP_PORT: u16 = 9222;

/// Which CEF page a session attaches to. With neither field set, the first
/// page is used.
#[derive(Debug, Clone, Default)]
pub struct PageTarget {
    /// URL the host loads into a newly created project page.
    pub initial_url: Option<String>,
    /// Target id of a page attached earlier; it survives navigation.
    pub target_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CdpLaunch {
    pub port: u16,
    pub page: PageTarget,
    pub startup_timeout: Duration,
    pub request_timeout: Duration,
}

impl CdpLaunch {
    pub(crate) fn endpoint(&self) -> String {
        format!("http://127.0.0.1:{}/", self.port)
    }
}
