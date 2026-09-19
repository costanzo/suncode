use super::*;
use std::sync::atomic::{AtomicBool, Ordering};
use suncode_computer::{ComputerAction, ComputerBackend, ComputerExecutor, PermissionState};

pub(super) fn is_computer_call(call: &ToolCall) -> bool {
    call.toolset_name.as_deref() == Some("computer")
}

pub(super) fn risk(name: &str) -> Option<Risk> {
    match name {
        "screenshot" | "zoom" | "cursor_position" | "wait" | "mouse_move" => {
            Some(Risk::ComputerObserve)
        }
        "left_click" | "right_click" | "middle_click" | "double_click" | "triple_click"
        | "left_click_drag" | "left_mouse_down" | "left_mouse_up" | "scroll" | "type" | "key"
        | "hold_key" => Some(Risk::ComputerInput),
        _ => None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComputerRuntimeInfo {
    pub enabled: bool,
    pub backend_available: bool,
    pub target_display: String,
    pub input_width: Option<u32>,
    pub input_height: Option<u32>,
    pub pixel_width: Option<u32>,
    pub pixel_height: Option<u32>,
    pub capture_permission: String,
    pub input_permission: String,
    pub control_owner: String,
    pub error: Option<String>,
}

#[derive(Clone)]
pub(super) struct ComputerManager {
    inner: Arc<Inner>,
}

struct Inner {
    enabled: AtomicBool,
    executor: Mutex<Option<ComputerExecutor<Box<dyn ComputerBackend>>>>,
}

impl ComputerManager {
    pub(super) fn new(store: &Store) -> Self {
        let enabled = store
            .settings(None, None)
            .ok()
            .and_then(|settings| {
                settings
                    .into_iter()
                    .find(|setting| setting.key == "computer_use_enabled")
                    .and_then(|setting| setting.value.as_bool())
            })
            .unwrap_or(false);
        Self {
            inner: Arc::new(Inner {
                enabled: AtomicBool::new(enabled),
                executor: Mutex::new(None),
            }),
        }
    }

    pub(super) fn install_backend(
        &self,
        backend: Box<dyn ComputerBackend>,
    ) -> Result<(), BusinessError> {
        let mut executor =
            self.inner.executor.lock().map_err(|_| {
                BusinessError::unavailable("Computer Use backend lock is unavailable")
            })?;
        *executor = Some(ComputerExecutor::new(backend));
        Ok(())
    }

    pub(super) fn set_enabled(&self, enabled: bool) -> Result<(), BusinessError> {
        self.inner.enabled.store(enabled, Ordering::SeqCst);
        if !enabled {
            if let Ok(mut executor) = self.inner.executor.lock() {
                if let Some(executor) = executor.as_mut() {
                    executor.release_all().map_err(computer_error)?;
                }
            }
        }
        Ok(())
    }

    pub(super) fn emergency_stop(&self) -> Result<(), BusinessError> {
        self.set_enabled(false)
    }

    pub(super) fn info(&self) -> ComputerRuntimeInfo {
        let mut input_width = None;
        let mut input_height = None;
        let mut pixel_width = None;
        let mut pixel_height = None;
        let mut capture_permission = "unknown".to_string();
        let mut input_permission = "unknown".to_string();
        let mut error = None;
        let backend_available = match self.inner.executor.lock() {
            Ok(mut executor) => match executor.as_mut() {
                Some(executor) => {
                    match executor.backend_mut().runtime_info() {
                        Ok(info) => {
                            input_width = Some(info.display.input_width);
                            input_height = Some(info.display.input_height);
                            pixel_width = Some(info.display.pixel_width);
                            pixel_height = Some(info.display.pixel_height);
                            capture_permission = permission_name(info.capture_permission).into();
                            input_permission = permission_name(info.input_permission).into();
                        }
                        Err(runtime_error) => error = Some(runtime_error.to_string()),
                    }
                    true
                }
                None => false,
            },
            Err(_) => {
                error = Some("Computer Use backend lock is unavailable".into());
                false
            }
        };
        let enabled = self.inner.enabled.load(Ordering::SeqCst);
        ComputerRuntimeInfo {
            enabled,
            backend_available,
            target_display: "primary".into(),
            input_width,
            input_height,
            pixel_width,
            pixel_height,
            capture_permission,
            input_permission,
            control_owner: if enabled && backend_available {
                "agent"
            } else {
                "user"
            }
            .into(),
            error,
        }
    }

    pub(super) fn catalog(
        &self,
        model_supports_computer_use: bool,
    ) -> Vec<suncode_llm::ClientToolsetDefinition> {
        let info = self.info();
        if !info.enabled || !info.backend_available || !model_supports_computer_use {
            return Vec::new();
        }
        vec![suncode_llm::ClientToolsetDefinition {
            type_name: "computer_toolset_20260801".into(),
            toolset_name: "computer".into(),
            configuration: json!({}),
        }]
    }

    pub(super) async fn execute(
        &self,
        member: &str,
        input: &Value,
        cancellation: CancellationToken,
    ) -> Result<suncode_computer::ActionOutcome, BusinessError> {
        if !self.inner.enabled.load(Ordering::SeqCst) {
            return Err(BusinessError::new(
                "computer_use_disabled",
                "Computer Use is disabled",
            ));
        }
        let action = ComputerAction::from_member(member, input).map_err(computer_error)?;
        let inner = self.inner.clone();
        tokio::task::spawn_blocking(move || {
            let mut executor = inner.executor.lock().map_err(|_| {
                BusinessError::unavailable("Computer Use backend lock is unavailable")
            })?;
            let executor = executor.as_mut().ok_or_else(|| {
                BusinessError::new(
                    "computer_backend_unavailable",
                    "Computer Use backend is unavailable",
                )
            })?;
            executor
                .execute(&action, &|| {
                    cancellation.is_cancelled() || !inner.enabled.load(Ordering::SeqCst)
                })
                .map_err(computer_error)
        })
        .await
        .map_err(|_| BusinessError::unavailable("Computer Use executor stopped unexpectedly"))?
    }
}

impl Agent {
    pub fn computer_runtime_info(&self) -> ComputerRuntimeInfo {
        self.computer.info()
    }

    pub fn set_computer_use_enabled(&self, enabled: bool) -> Result<(), BusinessError> {
        self.computer.set_enabled(enabled)
    }

    pub fn emergency_stop_computer_use(&self) -> Result<(), BusinessError> {
        self.computer.emergency_stop()
    }

    pub fn install_computer_backend(
        &self,
        backend: Box<dyn ComputerBackend>,
    ) -> Result<(), BusinessError> {
        self.computer.install_backend(backend)
    }
}

fn computer_error(error: suncode_computer::ComputerError) -> BusinessError {
    BusinessError::new("computer_action_failed", error.to_string())
}

const fn permission_name(permission: PermissionState) -> &'static str {
    match permission {
        PermissionState::Allowed => "allowed",
        PermissionState::Denied => "denied",
        PermissionState::Unknown => "unknown",
        PermissionState::Unsupported => "unsupported",
    }
}
