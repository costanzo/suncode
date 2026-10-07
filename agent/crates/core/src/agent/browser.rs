use super::*;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use suncode_browser::{
    CdpLaunch as WorkerLaunch, WorkerProcess, DEFAULT_CDP_PORT, PROTOCOL_VERSION,
};
use suncode_common::{HttpProxyConfiguration, HttpProxyMode};

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

#[derive(Clone)]
pub(super) struct BrowserManager {
    inner: Arc<Inner>,
}

pub type BrowserHostCallback = Arc<dyn Fn(&str) -> Result<(), BusinessError> + Send + Sync>;

struct Inner {
    data_dir: PathBuf,
    cdp_port: u16,
    enabled: AtomicBool,
    verified: AtomicBool,
    verification_error: std::sync::RwLock<Option<String>>,
    proxy: std::sync::RwLock<HttpProxyConfiguration>,
    projects: AsyncMutex<HashMap<String, BrowserSlot>>,
    verification_worker: AsyncMutex<Option<Arc<WorkerProcess>>>,
    closed: AtomicBool,
    host_available: bool,
    host_callback: std::sync::RwLock<Option<BrowserHostCallback>>,
}

struct BrowserSlot {
    state: BrowserRuntimeState,
    error: Option<String>,
    worker: Option<Arc<WorkerProcess>>,
    active_page_count: usize,
    control_owner: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkerState {
    started: bool,
    control_owner: String,
    #[serde(default)]
    pages: Vec<Value>,
}

impl BrowserManager {
    pub(super) fn new(store: Store, data_dir: PathBuf, host_available: bool) -> Self {
        let enabled = store
            .settings(None, None)
            .ok()
            .and_then(|settings| {
                settings
                    .into_iter()
                    .find(|setting| setting.key == "browser_use_enabled")
                    .and_then(|setting| setting.value.as_bool())
            })
            .unwrap_or(false);
        logging::debug(
            "browser.lifecycle",
            format!(
                "manager_created host_available={} enabled={} cdp_port={}",
                host_available, enabled, DEFAULT_CDP_PORT
            ),
        );
        Self::new_with_cdp_port(data_dir, DEFAULT_CDP_PORT, enabled, host_available)
    }

    fn new_with_cdp_port(
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
                verification_error: std::sync::RwLock::new(None),
                proxy: std::sync::RwLock::new(HttpProxyConfiguration::default()),
                projects: AsyncMutex::new(HashMap::new()),
                verification_worker: AsyncMutex::new(None),
                closed: AtomicBool::new(false),
                host_available,
                host_callback: std::sync::RwLock::new(None),
            }),
        }
    }

    pub(super) async fn info(
        &self,
        project_id: Option<&str>,
    ) -> Result<BrowserRuntimeInfo, BusinessError> {
        let enabled = self.inner.enabled.load(Ordering::SeqCst);
        let (_layout, _lock, installation_state, error) = if self.inner.host_available {
            self.installation_snapshot(enabled)
        } else {
            let layout = PathBuf::new();
            (
                layout,
                None,
                BrowserInstallationState::Unsupported,
                Some("Browser Use is unavailable in this host".into()),
            )
        };
        let integrity_verified = self.inner.verified.load(Ordering::SeqCst);
        let installation_ready = matches!(installation_state, BrowserInstallationState::Ready);
        let (runtime_state, control_owner, active_page_count, runtime_error) =
            if let Some(project_id) = project_id {
                let projects = self.inner.projects.lock().await;
                projects
                    .get(project_id)
                    .map(|slot| {
                        (
                            slot.state.clone(),
                            Some(slot.control_owner.clone()),
                            slot.active_page_count,
                            slot.error.clone(),
                        )
                    })
                    .unwrap_or((BrowserRuntimeState::NotStarted, None, 0, None))
            } else {
                (BrowserRuntimeState::NotStarted, None, 0, None)
            };
        let profile = project_id.map(|id| self.profile_path(id));
        Ok(BrowserRuntimeInfo {
            enabled,
            installation_state,
            runtime_state,
            target: format!("cef-cdp:{}", self.inner.cdp_port),
            node_path: String::new(),
            chromium_path: "CEF (embedded)".into(),
            chromium_version: "CEF".into(),
            chromium_revision: String::new(),
            worker_protocol_version: PROTOCOL_VERSION,
            integrity_state: if integrity_verified {
                "verified"
            } else if installation_ready {
                "unverified"
            } else {
                "unavailable"
            }
            .into(),
            control_owner,
            profile_path: profile
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned()),
            profile_size_bytes: profile.as_ref().map(|path| directory_size(path)),
            active_page_count,
            visibility_capability: visibility_capability(),
            error: runtime_error.or(error),
        })
    }

    pub(super) async fn catalog(&self) -> Vec<suncode_llm::ToolDefinition> {
        if !self.inner.host_available {
            logging::warn(
                "browser.catalog",
                "tools_not_advertised reason=host_unavailable",
            );
            return Vec::new();
        }
        let enabled = self.inner.enabled.load(Ordering::SeqCst);
        if !enabled {
            logging::info("browser.catalog", "tools_not_advertised reason=disabled");
            return Vec::new();
        }
        let definitions = suncode_tool::definitions::browser()
            .into_iter()
            .map(|definition| suncode_llm::ToolDefinition {
                name: definition.name.into(),
                description: definition.description.into(),
                parameters: definition.parameters,
            })
            .collect::<Vec<_>>();
        logging::debug(
            "browser.catalog",
            format!("tools_advertised count={}", definitions.len()),
        );
        definitions
    }

    pub(super) fn set_host_callback(&self, callback: Option<BrowserHostCallback>) {
        let registered = callback.is_some();
        if let Ok(mut current) = self.inner.host_callback.write() {
            *current = callback;
        }
        logging::debug("browser.host", format!("callback_registered={registered}"));
    }

    pub(super) async fn call(
        &self,
        project_id: &str,
        name: &str,
        arguments: Value,
        cancellation: CancellationToken,
    ) -> Result<Value, BusinessError> {
        logging::debug(
            "browser.call",
            format!("begin project={} tool={}", project_id, name),
        );
        self.require_enabled()?;
        if cancellation.is_cancelled() {
            return Err(BusinessError::new(
                "cancelled",
                "Browser operation was cancelled",
            ));
        }
        self.start_project(project_id).await?;
        let worker = self.worker(project_id).await?;
        let method = match name {
            "browser_open" => "open",
            "browser_snapshot" => "snapshot",
            "browser_navigate" => "navigate",
            "browser_click" => "click",
            "browser_fill" => "fill",
            "browser_press" => "press",
            "browser_tabs" => "state",
            "browser_screenshot" => "screenshot",
            "browser_close_page" => "close_page",
            _ => {
                return Err(BusinessError::new(
                    "browser_tool_unknown",
                    "Browser tool is not supported",
                ))
            }
        };
        let params = browser_worker_params(name, &arguments)?;
        let request = worker.request::<Value>(method, params);
        tokio::pin!(request);
        let mut result = tokio::select! {
            _ = cancellation.cancelled() => {
                worker.force_close().await;
                self.inner.projects.lock().await.remove(project_id);
                return Err(BusinessError::new("cancelled", "Browser operation was cancelled"));
            }
            result = &mut request => match result {
                Ok(result) => result,
                Err(error) => {
                    logging::write_business_error(
                        "browser.call",
                        "request",
                        &error,
                        format!("project={} tool={}", project_id, name),
                    );
                    worker.force_close().await;
                    self.fail_slot(project_id, &error).await;
                    return Err(error);
                }
            }
        };
        if let Some(pages) = result.get("pages").and_then(Value::as_array) {
            let mut projects = self.inner.projects.lock().await;
            if let Some(slot) = projects.get_mut(project_id) {
                slot.active_page_count = pages.len();
            }
        }
        logging::debug(
            "browser.call",
            format!("complete project={} tool={}", project_id, name),
        );
        if name == "browser_screenshot" {
            result = self.retain_screenshot(result)?;
        }
        Ok(result)
    }

    pub(super) async fn set_enabled(&self, enabled: bool) {
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
        let workers = {
            let mut projects = self.inner.projects.lock().await;
            projects
                .drain()
                .filter_map(|(_, slot)| slot.worker)
                .collect::<Vec<_>>()
        };
        for worker in workers {
            worker.close().await;
        }
    }

    pub(super) fn set_proxy_configuration(&self, configuration: HttpProxyConfiguration) {
        if let Ok(mut current) = self.inner.proxy.write() {
            *current = configuration;
        }
    }

    pub(super) async fn verify(&self) -> Result<(), BusinessError> {
        logging::debug(
            "browser.verify",
            format!(
                "begin host_available={} enabled={} cdp_port={}",
                self.inner.host_available,
                self.inner.enabled.load(Ordering::SeqCst),
                self.inner.cdp_port
            ),
        );
        self.require_enabled()?;
        if !self.inner.enabled.load(Ordering::SeqCst) {
            return Err(BusinessError::new(
                "browser_use_disabled",
                "Browser Use is disabled",
            ));
        }
        let verification = async {
            self.verify_integrity()?;
            let worker = self.spawn_worker("verify").await?;
            {
                let mut current = self.inner.verification_worker.lock().await;
                if self.inner.closed.load(Ordering::Acquire) {
                    drop(current);
                    worker.close().await;
                    return Err(BusinessError::new(
                        "agent_shutting_down",
                        "agent shutdown is in progress",
                    ));
                }
                *current = Some(worker.clone());
            }
            worker.close().await;
            self.inner.verification_worker.lock().await.take();
            Ok::<(), BusinessError>(())
        }
        .await;
        if let Err(error) = verification {
            logging::write_business_error("browser.verify", "runtime", &error, "runtime_check");
            self.inner.verified.store(false, Ordering::SeqCst);
            if let Ok(mut current) = self.inner.verification_error.write() {
                *current = Some(error.message.clone());
            }
            return Err(error);
        }
        logging::info("browser.verify", "runtime_verified=true");
        self.inner.verified.store(true, Ordering::SeqCst);
        if let Ok(mut error) = self.inner.verification_error.write() {
            *error = None;
        }
        Ok(())
    }

    pub(super) async fn start_project(&self, project_id: &str) -> Result<(), BusinessError> {
        self.require_enabled()?;
        if !self.inner.verified.load(Ordering::SeqCst) {
            self.verify_integrity()?;
            self.inner.verified.store(true, Ordering::SeqCst);
        }
        {
            let mut projects = self.inner.projects.lock().await;
            if projects
                .get(project_id)
                .is_some_and(|slot| slot.worker.is_some())
            {
                return Ok(());
            }
            projects.insert(
                project_id.into(),
                BrowserSlot {
                    state: BrowserRuntimeState::Starting,
                    error: None,
                    worker: None,
                    active_page_count: 0,
                    control_owner: "agent".into(),
                },
            );
        }
        let profile_path = self.profile_path(project_id);
        let downloads_path = self.downloads_path(project_id);
        let directory_result = prepare_managed_directory(
            &profile_path,
            &self.inner.data_dir.join("browser").join("profiles"),
        )
        .and_then(|_| {
            prepare_managed_directory(
                &downloads_path,
                &self.inner.data_dir.join("browser").join("downloads"),
            )
        });
        if let Err(error) = directory_result {
            self.fail_slot(project_id, &error).await;
            return Err(error);
        }
        let worker = match self.spawn_worker(project_id).await {
            Ok(worker) => worker,
            Err(error) => {
                self.fail_slot(project_id, &error).await;
                return Err(error);
            }
        };
        let installed = {
            let mut projects = self.inner.projects.lock().await;
            if self.inner.closed.load(Ordering::Acquire) {
                false
            } else if let Some(slot) = projects.get_mut(project_id) {
                slot.worker = Some(worker.clone());
                true
            } else {
                false
            }
        };
        if !installed {
            worker.close().await;
            return Err(BusinessError::new(
                "agent_shutting_down",
                "agent shutdown is in progress",
            ));
        }
        let state = worker
            .request::<WorkerState>(
                "start",
                json!({
                    "profilePath": profile_path,
                    "downloadsPath": downloads_path,
                    "headless": false,
                    "proxy": self.worker_proxy()
                }),
            )
            .await;
        let state = match state {
            Ok(state) => state,
            Err(error) => {
                worker.close().await;
                self.fail_slot(project_id, &error).await;
                return Err(error);
            }
        };
        if self.inner.closed.load(Ordering::Acquire) {
            worker.close().await;
            return Err(BusinessError::new(
                "agent_shutting_down",
                "agent shutdown is in progress",
            ));
        }
        let mut projects = self.inner.projects.lock().await;
        if self.inner.closed.load(Ordering::Acquire) || !projects.contains_key(project_id) {
            drop(projects);
            worker.close().await;
            return Err(BusinessError::new(
                "agent_shutting_down",
                "agent shutdown is in progress",
            ));
        }
        projects.insert(
            project_id.into(),
            BrowserSlot {
                state: BrowserRuntimeState::Background,
                error: None,
                worker: Some(worker),
                active_page_count: state.pages.len(),
                control_owner: state.control_owner,
            },
        );
        Ok(())
    }

    pub(super) async fn take_control(&self, project_id: &str) -> Result<(), BusinessError> {
        self.worker_request(project_id, "take_control").await
    }

    pub(super) async fn return_control(&self, project_id: &str) -> Result<(), BusinessError> {
        self.worker_request(project_id, "return_control").await
    }

    pub(super) async fn restart(&self, project_id: &str) -> Result<(), BusinessError> {
        self.stop(project_id).await?;
        self.start_project(project_id).await
    }

    pub(super) async fn stop(&self, project_id: &str) -> Result<(), BusinessError> {
        let worker = {
            let mut projects = self.inner.projects.lock().await;
            projects.remove(project_id).and_then(|slot| slot.worker)
        };
        if let Some(worker) = worker {
            worker.close().await;
        }
        Ok(())
    }

    pub(super) async fn shutdown(&self) {
        if self.inner.closed.swap(true, Ordering::AcqRel) {
            return;
        }
        let workers = {
            let mut projects = self.inner.projects.lock().await;
            projects
                .drain()
                .filter_map(|(_, slot)| slot.worker)
                .collect::<Vec<_>>()
        };
        if let Some(worker) = self.inner.verification_worker.lock().await.take() {
            worker.close().await;
        }
        for worker in workers {
            worker.close().await;
        }
    }

    pub(super) async fn clear_profile(&self, project_id: &str) -> Result<(), BusinessError> {
        if self
            .inner
            .projects
            .lock()
            .await
            .get(project_id)
            .is_some_and(|slot| slot.worker.is_some())
        {
            return Err(BusinessError::new(
                "browser_runtime_active",
                "Stop the project browser before clearing browser data",
            ));
        }
        let profile = self.profile_path(project_id);
        match fs::symlink_metadata(&profile) {
            Ok(metadata) => {
                let removal = if metadata.file_type().is_symlink() || metadata.is_file() {
                    fs::remove_file(&profile)
                } else {
                    fs::remove_dir_all(&profile)
                };
                removal.map_err(|error| {
                    BusinessError::new(
                        "browser_profile_clear_failed",
                        format!("browser profile could not be removed: {error}"),
                    )
                })?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(BusinessError::new(
                    "browser_profile_clear_failed",
                    format!("browser profile could not be inspected: {error}"),
                ));
            }
        }
        Ok(())
    }

    async fn worker_request(&self, project_id: &str, method: &str) -> Result<(), BusinessError> {
        let worker = self.worker(project_id).await?;
        let state = worker.request::<WorkerState>(method, json!({})).await?;
        let mut projects = self.inner.projects.lock().await;
        if let Some(slot) = projects.get_mut(project_id) {
            slot.state = if state.control_owner == "user" {
                BrowserRuntimeState::UserControlled
            } else if state.started {
                BrowserRuntimeState::Background
            } else {
                BrowserRuntimeState::NotStarted
            };
            slot.control_owner = state.control_owner;
            slot.active_page_count = state.pages.len();
            slot.error = None;
        }
        Ok(())
    }

    async fn worker(&self, project_id: &str) -> Result<Arc<WorkerProcess>, BusinessError> {
        self.inner
            .projects
            .lock()
            .await
            .get(project_id)
            .and_then(|slot| slot.worker.clone())
            .ok_or_else(|| {
                BusinessError::new(
                    "browser_runtime_not_started",
                    "Project browser is not running",
                )
            })
    }

    async fn spawn_worker(&self, scope: &str) -> Result<Arc<WorkerProcess>, BusinessError> {
        logging::debug(
            "browser.cdp",
            format!("connect_begin port={}", self.inner.cdp_port),
        );
        let target_url = (scope != "verify").then(|| browser_target_url(scope));
        let launch = |startup_timeout| {
            WorkerProcess::spawn(WorkerLaunch {
                port: self.inner.cdp_port,
                target_url: target_url.clone(),
                startup_timeout,
                request_timeout: Duration::from_secs(60),
            })
        };
        match launch(Duration::from_secs(2)).await {
            Ok(worker) => Ok(Arc::new(worker)),
            Err(first_error) => {
                logging::write_business_error(
                    "browser.cdp",
                    "connect",
                    &first_error,
                    format!("phase=initial project={scope}"),
                );
                let callback = self
                    .inner
                    .host_callback
                    .read()
                    .ok()
                    .and_then(|value| value.clone());
                let Some(callback) = callback else {
                    return Err(first_error);
                };
                logging::info("browser.host", format!("start_requested project={scope}"));
                let scope = scope.to_owned();
                let callback_scope = scope.clone();
                tokio::task::spawn_blocking(move || callback(&callback_scope))
                    .await
                    .map_err(|error| {
                        BusinessError::unavailable(format!("browser host callback failed: {error}"))
                    })??;
                match launch(Duration::from_secs(15)).await {
                    Ok(worker) => {
                        logging::info("browser.cdp", "connected_after_host_start=true");
                        Ok(Arc::new(worker))
                    }
                    Err(error) => {
                        logging::write_business_error(
                            "browser.cdp",
                            "connect_after_host_start",
                            &error,
                            format!("project={scope}"),
                        );
                        Err(error)
                    }
                }
            }
        }
    }

    async fn fail_slot(&self, project_id: &str, error: &BusinessError) {
        if self.inner.closed.load(Ordering::Acquire) {
            return;
        }
        let mut projects = self.inner.projects.lock().await;
        projects.insert(
            project_id.into(),
            BrowserSlot {
                state: BrowserRuntimeState::Failed,
                error: Some(error.message.clone()),
                worker: None,
                active_page_count: 0,
                control_owner: "agent".into(),
            },
        );
    }

    fn require_enabled(&self) -> Result<(), BusinessError> {
        if !self.inner.host_available {
            return Err(BusinessError::new(
                "browser_host_unavailable",
                "Browser Use is unavailable in this host",
            ));
        }
        if self.inner.closed.load(Ordering::Acquire) {
            return Err(BusinessError::new(
                "agent_shutting_down",
                "agent shutdown is in progress",
            ));
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

    fn verify_integrity(&self) -> Result<(), BusinessError> {
        Ok(())
    }

    fn worker_proxy(&self) -> Value {
        let configuration = self
            .inner
            .proxy
            .read()
            .map(|configuration| configuration.clone())
            .unwrap_or_default();
        match configuration.mode {
            HttpProxyMode::Custom => json!({
                "mode": "custom",
                "server": configuration.url,
                "username": configuration.username,
                "password": configuration.password,
                "bypass": configuration.no_proxy_value(),
            }),
            HttpProxyMode::NoProxy => json!({"mode": "no_proxy"}),
            HttpProxyMode::System => json!({"mode": "system"}),
        }
    }

    fn installation_snapshot(
        &self,
        enabled: bool,
    ) -> (
        PathBuf,
        Option<()>,
        BrowserInstallationState,
        Option<String>,
    ) {
        if !enabled {
            return (
                PathBuf::new(),
                None,
                BrowserInstallationState::Disabled,
                None,
            );
        }
        if !self.inner.host_available {
            return (
                PathBuf::new(),
                None,
                BrowserInstallationState::Unsupported,
                Some("Browser Use is unavailable in this host".into()),
            );
        }
        (PathBuf::new(), None, BrowserInstallationState::Ready, None)
    }

    fn profile_path(&self, project_id: &str) -> PathBuf {
        self.inner
            .data_dir
            .join("browser")
            .join("profiles")
            .join(project_directory_name(project_id))
    }

    fn downloads_path(&self, project_id: &str) -> PathBuf {
        self.inner
            .data_dir
            .join("browser")
            .join("downloads")
            .join(project_directory_name(project_id))
    }

    fn retain_screenshot(&self, result: Value) -> Result<Value, BusinessError> {
        let encoded = result
            .get("base64")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                BusinessError::new(
                    "browser_screenshot_invalid",
                    "Browser screenshot data is missing",
                )
            })?;
        let bytes = STANDARD.decode(encoded).map_err(|_| {
            BusinessError::new(
                "browser_screenshot_invalid",
                "Browser screenshot data is invalid",
            )
        })?;
        if bytes.is_empty() || bytes.len() > 5 * 1024 * 1024 {
            return Err(BusinessError::new(
                "browser_screenshot_too_large",
                "Browser screenshot must be between 1 byte and 5 MiB",
            ));
        }
        let artifact_id = format!("browser-{}", Uuid::new_v4());
        let directory = self.inner.data_dir.join("browser").join("artifacts");
        prepare_managed_directory(&directory, &self.inner.data_dir.join("browser"))
            .map_err(|error| BusinessError::new("browser_artifact_failed", error.message))?;
        fs::write(directory.join(format!("{artifact_id}.png")), &bytes).map_err(|error| {
            BusinessError::new(
                "browser_artifact_failed",
                format!("Browser screenshot could not be stored: {error}"),
            )
        })?;
        Ok(json!({
            "pageId": result.get("pageId").cloned().unwrap_or(Value::Null),
            "artifact_id": artifact_id,
            "mime_type": "image/png",
            "bytes": bytes.len(),
        }))
    }
}

pub(super) fn is_browser_tool(name: &str) -> bool {
    name.starts_with("browser_")
}

pub(super) fn validate_browser_arguments(
    name: &str,
    arguments: &Value,
) -> Result<(), BusinessError> {
    let object = arguments
        .as_object()
        .ok_or_else(|| BusinessError::invalid("Browser tool arguments must be an object"))?;
    if matches!(name, "browser_click" | "browser_fill" | "browser_press")
        && !object.get("target").is_some_and(Value::is_object)
    {
        return Err(BusinessError::invalid("target is required"));
    }
    if name == "browser_fill" && !object.get("value").is_some_and(Value::is_string) {
        return Err(BusinessError::invalid("value is required"));
    }
    if name == "browser_press"
        && object
            .get("key")
            .and_then(Value::as_str)
            .is_none_or(|value| value.is_empty() || value.chars().count() > 100)
    {
        return Err(BusinessError::invalid(
            "key is required and must be at most 100 characters",
        ));
    }
    browser_worker_params(name, arguments).map(|_| ())
}

impl Agent {
    pub fn set_browser_host_callback(&self, callback: Option<BrowserHostCallback>) {
        self.browser.set_host_callback(callback);
    }

    pub fn preview_state(&self, project_id: &str) -> crate::agent::PreviewState {
        self.preview.state(project_id)
    }
    pub fn start_preview(
        &self,
        project_id: &str,
        project_root: &str,
        program: &str,
        args: Vec<String>,
        cwd: Option<String>,
        url: &str,
    ) -> Result<crate::agent::PreviewState, BusinessError> {
        self.preview
            .start(project_id, Path::new(project_root), program, args, cwd, url)
    }
    pub fn stop_preview(&self, project_id: &str) -> crate::agent::PreviewState {
        self.preview.stop(project_id)
    }
    pub async fn browser_runtime_info(
        &self,
        project_id: Option<&str>,
    ) -> Result<BrowserRuntimeInfo, BusinessError> {
        self.browser.info(project_id).await
    }

    pub async fn set_browser_use_enabled(&self, enabled: bool) {
        self.browser.set_enabled(enabled).await;
    }

    pub fn set_browser_proxy_configuration(&self, configuration: HttpProxyConfiguration) {
        self.browser.set_proxy_configuration(configuration);
    }

    pub async fn verify_browser_runtime(&self) -> Result<(), BusinessError> {
        self.browser.verify().await
    }

    pub async fn start_browser_project(&self, project_id: &str) -> Result<(), BusinessError> {
        self.browser.start_project(project_id).await
    }

    pub async fn take_browser_control(&self, project_id: &str) -> Result<(), BusinessError> {
        self.browser.take_control(project_id).await
    }

    pub async fn return_browser_control(&self, project_id: &str) -> Result<(), BusinessError> {
        self.browser.return_control(project_id).await
    }

    pub async fn restart_browser_runtime(&self, project_id: &str) -> Result<(), BusinessError> {
        self.browser.restart(project_id).await
    }

    pub async fn stop_browser_runtime(&self, project_id: &str) -> Result<(), BusinessError> {
        self.browser.stop(project_id).await
    }

    pub async fn clear_browser_profile(&self, project_id: &str) -> Result<(), BusinessError> {
        self.browser.clear_profile(project_id).await
    }
}

fn project_directory_name(project_id: &str) -> String {
    let digest = Sha256::digest(project_id.as_bytes());
    digest[..16]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn browser_target_url(project_id: &str) -> String {
    format!(
        "data:text/html,%3Ctitle%3Esuncode-browser-use-{}%3C%2Ftitle%3E",
        project_directory_name(project_id)
    )
}

fn prepare_managed_directory(path: &Path, root: &Path) -> Result<(), BusinessError> {
    fs::create_dir_all(root).map_err(|error| {
        BusinessError::new(
            "browser_runtime_unavailable",
            format!("Browser data root could not be created: {error}"),
        )
    })?;
    if fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
    {
        return Err(BusinessError::new(
            "browser_data_path_invalid",
            "Browser data directory must not be a symbolic link",
        ));
    }
    fs::create_dir_all(path).map_err(|error| {
        BusinessError::new(
            "browser_runtime_unavailable",
            format!("Browser data directory could not be created: {error}"),
        )
    })?;
    let canonical_root = fs::canonicalize(root).map_err(|error| {
        BusinessError::new(
            "browser_runtime_unavailable",
            format!("Browser data root could not be resolved: {error}"),
        )
    })?;
    let canonical_path = fs::canonicalize(path).map_err(|error| {
        BusinessError::new(
            "browser_runtime_unavailable",
            format!("Browser data directory could not be resolved: {error}"),
        )
    })?;
    if !canonical_path.starts_with(canonical_root) {
        return Err(BusinessError::new(
            "browser_data_path_invalid",
            "Browser data directory escaped its managed root",
        ));
    }
    Ok(())
}

fn directory_size(path: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| {
            let Ok(file_type) = entry.file_type() else {
                return 0;
            };
            if file_type.is_dir() {
                directory_size(&entry.path())
            } else if file_type.is_file() {
                entry.metadata().map(|metadata| metadata.len()).unwrap_or(0)
            } else {
                0
            }
        })
        .sum()
}

fn visibility_capability() -> BrowserVisibilityCapability {
    if cfg!(any(target_os = "macos", target_os = "windows")) {
        BrowserVisibilityCapability::Full
    } else if cfg!(target_os = "linux") {
        BrowserVisibilityCapability::Limited
    } else {
        BrowserVisibilityCapability::Unsupported
    }
}

fn browser_worker_params(name: &str, arguments: &Value) -> Result<Value, BusinessError> {
    let object = arguments
        .as_object()
        .ok_or_else(|| BusinessError::invalid("Browser tool arguments must be an object"))?;
    let mut params = serde_json::Map::new();
    if let Some(page_id) = object.get("page_id") {
        params.insert("pageId".into(), page_id.clone());
    }
    if let Some(target) = object.get("target") {
        params.insert("target".into(), target.clone());
    }
    if let Some(value) = object.get("value") {
        params.insert("value".into(), value.clone());
    }
    if let Some(key) = object.get("key") {
        params.insert("key".into(), key.clone());
    }
    if let Some(timeout) = object.get("timeout_ms") {
        params.insert("timeoutMs".into(), timeout.clone());
    }
    if let Some(full_page) = object.get("full_page") {
        params.insert("fullPage".into(), full_page.clone());
    }
    if let Some(url) = object.get("url").and_then(Value::as_str) {
        validate_browser_url(url)?;
        params.insert("url".into(), Value::String(url.into()));
    } else if name == "browser_navigate" {
        return Err(BusinessError::invalid("url is required"));
    }
    Ok(Value::Object(params))
}

fn validate_browser_url(value: &str) -> Result<(), BusinessError> {
    let url = url::Url::parse(value)
        .map_err(|_| BusinessError::invalid("Browser URL must be an absolute HTTP or HTTPS URL"))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(BusinessError::invalid(
            "Browser URL must be an absolute HTTP or HTTPS URL",
        ));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(BusinessError::invalid(
            "Browser URL must not contain embedded credentials",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn unavailable_host_exposes_no_browser_tools_or_runtime_start() {
        let store = Store::open_memory().unwrap();
        store
            .set_setting(
                "global",
                "global",
                "browser_use_enabled",
                &Value::Bool(true),
            )
            .unwrap();
        let directory = tempfile::tempdir().unwrap();
        let manager = BrowserManager::new(store, directory.path().to_path_buf(), false);

        assert!(manager.catalog().await.is_empty());
        let info = manager.info(None).await.unwrap();
        assert!(info.enabled);
        assert_eq!(
            info.installation_state,
            BrowserInstallationState::Unsupported
        );
        assert_eq!(
            manager.start_project("project-1").await.unwrap_err().code,
            "browser_host_unavailable"
        );
    }

    #[tokio::test]
    async fn enabled_host_advertises_browser_tools_before_cef_starts() {
        let store = Store::open_memory().unwrap();
        store
            .set_setting(
                "global",
                "global",
                "browser_use_enabled",
                &Value::Bool(true),
            )
            .unwrap();
        let directory = tempfile::tempdir().unwrap();
        let manager = BrowserManager::new(store, directory.path().to_path_buf(), true);

        let definitions = manager.catalog().await;
        assert_eq!(definitions.len(), 9);
        assert!(definitions
            .iter()
            .any(|definition| definition.name == "browser_open"));
    }

    #[test]
    fn project_profile_directory_does_not_disclose_project_identifier() {
        let value = project_directory_name("project/with/private/path");
        assert_eq!(value.len(), 32);
        assert!(!value.contains("project"));
        assert!(value.chars().all(|character| character.is_ascii_hexdigit()));
    }

    #[test]
    fn browser_urls_reject_credentials_and_non_http_schemes() {
        assert!(validate_browser_url("https://example.test/path").is_ok());
        assert!(validate_browser_url("http://127.0.0.1:5173").is_ok());
        assert!(validate_browser_url("file:///etc/passwd").is_err());
        assert!(validate_browser_url("https://user:secret@example.test").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn managed_browser_directories_reject_symlinks() {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let profiles = root.path().join("profiles");
        fs::create_dir_all(&profiles).unwrap();
        let project = profiles.join("project");
        symlink(outside.path(), &project).unwrap();

        assert_eq!(
            prepare_managed_directory(&project, &profiles)
                .unwrap_err()
                .code,
            "browser_data_path_invalid"
        );
    }
}
