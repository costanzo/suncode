//! Per-project Browser Use runtime: CDP sessions, status, and managed data.
//!
//! One CEF page per project, owned by the desktop host. The runtime connects to
//! it on demand, tracks its state, and stores profiles and artifacts under the
//! agent data directory.

mod connection;
mod lifecycle;
mod status;
mod storage;
mod tool_call;

use crate::cdp::{CdpSession, DEFAULT_CDP_PORT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use suncode_common::{logging, BusinessError};
use tokio::sync::Mutex as AsyncMutex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BrowserInstallationState {
    Disabled,
    Ready,
    Missing,
    Invalid,
    Unsupported,
    Verifying,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BrowserRuntimeState {
    NotStarted,
    Starting,
    Background,
    UserControlled,
    Stopping,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BrowserVisibilityCapability {
    Full,
    Limited,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BrowserRuntimeInfo {
    pub enabled: bool,
    pub installation_state: BrowserInstallationState,
    pub runtime_state: BrowserRuntimeState,
    pub target: String,
    pub node_path: String,
    pub chromium_path: String,
    pub chromium_version: String,
    pub chromium_revision: String,
    pub worker_protocol_version: u32,
    pub integrity_state: String,
    pub control_owner: Option<String>,
    pub profile_path: Option<String>,
    pub profile_size_bytes: Option<u64>,
    pub active_page_count: usize,
    pub visibility_capability: BrowserVisibilityCapability,
    pub error: Option<String>,
}

/// Cheap to clone; clones share one runtime.
#[derive(Clone)]
pub struct BrowserRuntime {
    inner: Arc<Inner>,
}

/// Asks the host to initialize CEF and open the page for a project scope.
pub type BrowserHostCallback = Arc<dyn Fn(&str) -> Result<(), BusinessError> + Send + Sync>;

struct Inner {
    data_dir: PathBuf,
    cdp_port: u16,
    enabled: AtomicBool,
    verified: AtomicBool,
    verification_error: RwLock<Option<String>>,
    projects: AsyncMutex<HashMap<String, BrowserSlot>>,
    /// Per-project start locks so concurrent starts share one CDP session instead of
    /// spawning a second one and orphaning the first.
    start_locks: Mutex<HashMap<String, Arc<AsyncMutex<()>>>>,
    verification_session: AsyncMutex<Option<Arc<CdpSession>>>,
    closed: AtomicBool,
    host_available: bool,
    host_callback: RwLock<Option<BrowserHostCallback>>,
}

struct BrowserSlot {
    state: BrowserRuntimeState,
    error: Option<String>,
    session: Option<Arc<CdpSession>>,
    active_page_count: usize,
    control_owner: String,
}

impl BrowserSlot {
    fn starting() -> Self {
        Self::idle(BrowserRuntimeState::Starting, None)
    }

    fn failed(error: &BusinessError) -> Self {
        Self::idle(BrowserRuntimeState::Failed, Some(error.message.clone()))
    }

    fn idle(state: BrowserRuntimeState, error: Option<String>) -> Self {
        Self {
            state,
            error,
            session: None,
            active_page_count: 0,
            control_owner: "agent".into(),
        }
    }
}

impl BrowserRuntime {
    /// `enabled` is the persisted user setting; `host_available` reports whether
    /// the host can provide a CEF page at all.
    pub fn new(data_dir: PathBuf, enabled: bool, host_available: bool) -> Self {
        logging::debug(
            "browser.lifecycle",
            format!(
                "manager_created host_available={host_available} enabled={enabled} cdp_port={DEFAULT_CDP_PORT}"
            ),
        );
        Self::with_cdp_port(data_dir, DEFAULT_CDP_PORT, enabled, host_available)
    }

    fn with_cdp_port(
        data_dir: PathBuf,
        cdp_port: u16,
        enabled: bool,
        host_available: bool,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                data_dir,
                cdp_port,
                enabled: AtomicBool::new(enabled),
                verified: AtomicBool::new(false),
                verification_error: RwLock::new(None),
                projects: AsyncMutex::new(HashMap::new()),
                start_locks: Mutex::new(HashMap::new()),
                verification_session: AsyncMutex::new(None),
                closed: AtomicBool::new(false),
                host_available,
                host_callback: RwLock::new(None),
            }),
        }
    }

    pub fn set_host_callback(&self, callback: Option<BrowserHostCallback>) {
        let registered = callback.is_some();
        if let Ok(mut current) = self.inner.host_callback.write() {
            *current = callback;
        }
        logging::debug("browser.host", format!("callback_registered={registered}"));
    }

    pub async fn set_enabled(&self, enabled: bool) {
        if enabled && !self.inner.host_available {
            return;
        }
        if enabled && self.inner.closed.load(Ordering::Acquire) {
            return;
        }
        self.inner.enabled.store(enabled, Ordering::SeqCst);
        if enabled {
            self.inner.verified.store(false, Ordering::SeqCst);
            if let Ok(mut error) = self.inner.verification_error.write() {
                *error = None;
            }
            return;
        }
        let sessions = {
            let mut projects = self.inner.projects.lock().await;
            projects
                .drain()
                .filter_map(|(_, slot)| slot.session)
                .collect::<Vec<_>>()
        };
        for session in sessions {
            session.close().await;
        }
    }

    fn require_enabled(&self) -> Result<(), BusinessError> {
        if !self.inner.host_available {
            return Err(BusinessError::new(
                "browser_host_unavailable",
                "Browser Use is unavailable in this host",
            ));
        }
        if self.inner.closed.load(Ordering::Acquire) {
            return Err(lifecycle::shutting_down());
        }
        if self.inner.enabled.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err(BusinessError::new(
                "browser_use_disabled",
                "Browser Use is disabled",
            ))
        }
    }
}
