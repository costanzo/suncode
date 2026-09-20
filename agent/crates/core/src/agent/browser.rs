use super::*;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    sync::atomic::{AtomicBool, Ordering},
};
use suncode_browser::{RuntimeLayout, RuntimeLock, WorkerLaunch, WorkerProcess};
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
    pub node_version: String,
    pub playwright_version: String,
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

struct Inner {
    data_dir: PathBuf,
    runtime_root: PathBuf,
    enabled: AtomicBool,
    verified: AtomicBool,
    verification_error: std::sync::RwLock<Option<String>>,
    proxy: std::sync::RwLock<HttpProxyConfiguration>,
    projects: AsyncMutex<HashMap<String, BrowserSlot>>,
    verification_worker: AsyncMutex<Option<Arc<WorkerProcess>>>,
    closed: AtomicBool,
    host_available: bool,
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
        Self::new_with_runtime_root(data_dir, default_runtime_root(), enabled, host_available)
    }

    fn new_with_runtime_root(
        data_dir: PathBuf,
        runtime_root: PathBuf,
        enabled: bool,
        host_available: bool,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                data_dir,
                runtime_root,
                enabled: AtomicBool::new(enabled),
                verified: AtomicBool::new(false),
                verification_error: std::sync::RwLock::new(None),
                proxy: std::sync::RwLock::new(HttpProxyConfiguration::default()),
                projects: AsyncMutex::new(HashMap::new()),
                verification_worker: AsyncMutex::new(None),
                closed: AtomicBool::new(false),
                host_available,
            }),
        }
    }

    pub(super) async fn info(
        &self,
        project_id: Option<&str>,
    ) -> Result<BrowserRuntimeInfo, BusinessError> {
        let enabled = self.inner.enabled.load(Ordering::SeqCst);
        let (layout, lock, installation_state, error) = if self.inner.host_available {
            self.installation_snapshot(enabled)
        } else {
            let layout = runtime_layout(&self.inner.runtime_root);
            (
                layout,
                read_lock(&self.inner.runtime_root),
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
            target: suncode_browser::target_name().into(),
            node_path: layout.node_path.to_string_lossy().into_owned(),
            node_version: lock
                .as_ref()
                .map(|value| value.node.version.clone())
                .unwrap_or_default(),
            playwright_version: lock
                .as_ref()
                .map(|value| value.playwright.version.clone())
                .unwrap_or_default(),
            chromium_path: chromium_path(&self.inner.runtime_root, lock.as_ref())
                .to_string_lossy()
                .into_owned(),
            chromium_version: lock
                .as_ref()
                .map(|value| value.chromium.version.clone())
                .unwrap_or_default(),
            chromium_revision: lock
                .as_ref()
                .map(|value| value.chromium.revision.clone())
                .unwrap_or_default(),
            worker_protocol_version: lock
                .as_ref()
                .map(|value| value.protocol_version)
                .unwrap_or(suncode_browser::PROTOCOL_VERSION),
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
            return Vec::new();
        }
        let enabled = self.inner.enabled.load(Ordering::SeqCst);
        if !enabled {
            return Vec::new();
        }
        let verification_failed = self
            .inner
            .verification_error
            .read()
            .map(|error| error.is_some())
            .unwrap_or(true);
        if !self.inner.verified.load(Ordering::SeqCst)
            && !verification_failed
            && self.verify().await.is_err()
        {
            return Vec::new();
        }
        if !self.inner.verified.load(Ordering::SeqCst) {
            return Vec::new();
        }
        suncode_tool::definitions::browser()
            .into_iter()
            .map(|definition| suncode_llm::ToolDefinition {
                name: definition.name.into(),
                description: definition.description.into(),
                parameters: definition.parameters,
            })
            .collect()
    }

    pub(super) async fn call(
        &self,
        project_id: &str,
        name: &str,
        arguments: Value,
        cancellation: CancellationToken,
    ) -> Result<Value, BusinessError> {
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
            self.inner.verified.store(false, Ordering::SeqCst);
            if let Ok(mut current) = self.inner.verification_error.write() {
                *current = Some(error.message.clone());
            }
            return Err(error);
        }
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
        let layout = runtime_layout(&self.inner.runtime_root);
        let environment = BTreeMap::from([(
            "PLAYWRIGHT_BROWSERS_PATH".into(),
            self.inner
                .runtime_root
                .join("browsers")
                .to_string_lossy()
                .into_owned(),
        )]);
        WorkerProcess::spawn(WorkerLaunch {
            layout,
            working_directory: self.inner.data_dir.join("browser").join("runtime").join(
                if scope == "verify" {
                    "verify".into()
                } else {
                    project_directory_name(scope)
                },
            ),
            startup_timeout: Duration::from_secs(15),
            request_timeout: Duration::from_secs(60),
            environment,
        })
        .await
        .map(Arc::new)
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
        suncode_browser::verify_runtime_tree(&self.inner.runtime_root).map(|_| ())
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
        RuntimeLayout,
        Option<RuntimeLock>,
        BrowserInstallationState,
        Option<String>,
    ) {
        let layout = runtime_layout(&self.inner.runtime_root);
        if !enabled {
            return (
                layout,
                read_lock(&self.inner.runtime_root),
                BrowserInstallationState::Disabled,
                None,
            );
        }
        if suncode_browser::target_name() == "unsupported" {
            return (
                layout,
                read_lock(&self.inner.runtime_root),
                BrowserInstallationState::Unsupported,
                Some("Browser Use is unavailable on this platform".into()),
            );
        }
        let lock = match RuntimeLock::read(&layout.runtime_lock_path) {
            Ok(lock) => lock,
            Err(error) => {
                return (
                    layout,
                    None,
                    BrowserInstallationState::Missing,
                    Some(error.message),
                )
            }
        };
        if !layout.node_path.is_file()
            || !layout.worker_path.is_file()
            || !self
                .inner
                .runtime_root
                .join("runtime-manifest.json")
                .is_file()
            || !chromium_path(&self.inner.runtime_root, Some(&lock)).is_file()
            || !self
                .inner
                .runtime_root
                .join("browsers")
                .join(format!("ffmpeg-{}", lock.ffmpeg.revision))
                .is_dir()
        {
            return (
                layout,
                Some(lock),
                BrowserInstallationState::Missing,
                Some("Bundled Node.js or browser worker is missing".into()),
            );
        }
        if let Some(error) = self
            .inner
            .verification_error
            .read()
            .ok()
            .and_then(|error| error.clone())
        {
            return (
                layout,
                Some(lock),
                BrowserInstallationState::Invalid,
                Some(error),
            );
        }
        (layout, Some(lock), BrowserInstallationState::Ready, None)
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

fn default_runtime_root() -> PathBuf {
    let executable = std::env::current_exe().unwrap_or_default();
    let directory = executable.parent().unwrap_or_else(|| Path::new("."));
    let sibling = directory.join("browser-runtime");
    if sibling.is_dir() {
        return sibling;
    }
    let mac_resources = directory.join("../Resources/browser-runtime");
    if mac_resources.is_dir() {
        return mac_resources;
    }
    #[cfg(debug_assertions)]
    {
        return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../browser-runtime");
    }
    #[cfg(not(debug_assertions))]
    sibling
}

fn runtime_layout(root: &Path) -> RuntimeLayout {
    RuntimeLayout {
        node_path: if cfg!(windows) {
            root.join("node/node.exe")
        } else {
            root.join("node/bin/node")
        },
        worker_path: root.join("worker/index.mjs"),
        runtime_lock_path: root.join("runtime-lock.json"),
    }
}

fn chromium_path(root: &Path, lock: Option<&RuntimeLock>) -> PathBuf {
    let revision = lock
        .map(|lock| lock.chromium.revision.as_str())
        .unwrap_or("unknown");
    let directory = root.join("browsers").join(format!("chromium-{revision}"));
    match suncode_browser::target_name() {
        "darwin-arm64" => directory.join("chrome-mac/Chromium.app/Contents/MacOS/Chromium"),
        "win32-x64" => directory.join("chrome-win/chrome.exe"),
        "linux-x64" => directory.join("chrome-linux/chrome"),
        _ => directory,
    }
}

fn read_lock(root: &Path) -> Option<RuntimeLock> {
    RuntimeLock::read(&root.join("runtime-lock.json")).ok()
}

fn project_directory_name(project_id: &str) -> String {
    let digest = Sha256::digest(project_id.as_bytes());
    digest[..16]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
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

    #[test]
    fn project_profile_directory_does_not_disclose_project_identifier() {
        let value = project_directory_name("project/with/private/path");
        assert_eq!(value.len(), 32);
        assert!(!value.contains("project"));
        assert!(value.chars().all(|character| character.is_ascii_hexdigit()));
    }

    #[test]
    fn runtime_layout_is_fixed_beneath_the_runtime_root() {
        let layout = runtime_layout(Path::new("/app/browser-runtime"));
        assert!(layout.worker_path.ends_with("worker/index.mjs"));
        assert!(layout.runtime_lock_path.ends_with("runtime-lock.json"));
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
