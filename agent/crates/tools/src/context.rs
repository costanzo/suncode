//! Per-call execution context threaded from `Operations` into each tool.

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use suncode_common::HttpProxyConfiguration;
use suncode_documents::Documents;

/// Callback for bounded, best-effort streaming process output chunks.
pub type ProcessOutputCallback = Arc<dyn Fn(&str, &[u8]) + Send + Sync>;

/// TLS and proxy settings for outbound HTTP, snapshotted per call.
#[derive(Clone)]
pub(crate) struct NetworkSettings {
    pub(crate) verify_https_certificates: bool,
    pub(crate) use_system_certificates: bool,
    pub(crate) certificate_path: Option<PathBuf>,
    pub(crate) proxy: HttpProxyConfiguration,
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            verify_https_certificates: true,
            use_system_certificates: true,
            certificate_path: None,
            proxy: HttpProxyConfiguration::default(),
        }
    }
}

pub(crate) struct ExecutionContext<'a> {
    pub(crate) project_root: Option<&'a Path>,
    pub(crate) checkpoint_root: Option<&'a Path>,
    pub(crate) cancellation: Option<&'a AtomicBool>,
    pub(crate) output_callback: Option<ProcessOutputCallback>,
    pub(crate) network: NetworkSettings,
    pub(crate) documents: &'a Documents,
}
