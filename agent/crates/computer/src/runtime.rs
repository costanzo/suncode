//! Computer Use runtime: enablement, control ownership, the action executor,
//! screenshot artifacts, and the model-facing tool catalog.

use crate::{
    ActionOutcome, ComputerAction, ComputerBackend, ComputerExecutor, EnigoBackend,
    PermissionState, MAX_PROVIDER_PNG_BYTES,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::task::Poll;
use suncode_common::blocking::run_blocking;
use suncode_common::tool::{Cancellation, ToolDefinition};
use suncode_common::BusinessError;

pub(crate) fn is_known_member(name: &str) -> bool {
    matches!(
        name,
        "screenshot"
            | "zoom"
            | "left_click"
            | "right_click"
            | "middle_click"
            | "double_click"
            | "triple_click"
            | "left_click_drag"
            | "mouse_move"
            | "left_mouse_down"
            | "left_mouse_up"
            | "cursor_position"
            | "scroll"
            | "type"
            | "key"
            | "hold_key"
            | "wait"
    )
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
pub struct ComputerRuntime {
    inner: Arc<Inner>,
}

struct Inner {
    data_dir: PathBuf,
    enabled: AtomicBool,
    host_available: bool,
    control_owner: AtomicU8,
    executor: Mutex<Option<ComputerExecutor<Box<dyn ComputerBackend>>>>,
}

const CONTROL_USER: u8 = 0;
const CONTROL_AGENT: u8 = 1;

impl ComputerRuntime {
    /// `enabled` is the persisted user setting; `host_available` is the
    /// host's capability ceiling.
    pub fn new(enabled: bool, host_available: bool, data_dir: PathBuf) -> Self {
        Self {
            inner: Arc::new(Inner {
                data_dir,
                enabled: AtomicBool::new(enabled),
                host_available,
                control_owner: AtomicU8::new(if enabled { CONTROL_AGENT } else { CONTROL_USER }),
                executor: Mutex::new(None),
            }),
        }
    }

    pub fn install_backend(&self, backend: Box<dyn ComputerBackend>) -> Result<(), BusinessError> {
        self.ensure_host_available()?;
        let mut executor =
            self.inner.executor.lock().map_err(|_| {
                BusinessError::unavailable("Computer Use backend lock is unavailable")
            })?;
        *executor = Some(ComputerExecutor::new(backend));
        Ok(())
    }

    pub fn set_enabled(&self, enabled: bool) -> Result<(), BusinessError> {
        if enabled {
            self.ensure_host_available()?;
        }
        self.inner.enabled.store(enabled, Ordering::SeqCst);
        self.inner.control_owner.store(
            if enabled { CONTROL_AGENT } else { CONTROL_USER },
            Ordering::SeqCst,
        );
        if !enabled {
            if let Ok(mut executor) = self.inner.executor.lock() {
                if let Some(executor) = executor.as_mut() {
                    executor.release_all().map_err(computer_error)?;
                    executor.retire_frame();
                }
            }
        }
        Ok(())
    }

    pub fn emergency_stop(&self) -> Result<(), BusinessError> {
        self.set_enabled(false)
    }

    pub fn take_user_control(&self) -> Result<(), BusinessError> {
        self.ensure_host_available()?;
        self.inner
            .control_owner
            .store(CONTROL_USER, Ordering::SeqCst);
        let mut executor =
            self.inner.executor.lock().map_err(|_| {
                BusinessError::unavailable("Computer Use backend lock is unavailable")
            })?;
        if let Some(executor) = executor.as_mut() {
            executor.release_all().map_err(computer_error)?;
            executor.retire_frame();
        }
        Ok(())
    }

    pub fn return_agent_control(&self) -> Result<(), BusinessError> {
        self.ensure_host_available()?;
        if !self.inner.enabled.load(Ordering::SeqCst) {
            return Err(BusinessError::new(
                "computer_use_disabled",
                "Computer Use is disabled",
            ));
        }
        let mut executor =
            self.inner.executor.lock().map_err(|_| {
                BusinessError::unavailable("Computer Use backend lock is unavailable")
            })?;
        if let Some(executor) = executor.as_mut() {
            executor.retire_frame();
        }
        self.inner
            .control_owner
            .store(CONTROL_AGENT, Ordering::SeqCst);
        Ok(())
    }

    pub fn info(&self) -> ComputerRuntimeInfo {
        if !self.inner.host_available {
            return ComputerRuntimeInfo {
                enabled: self.inner.enabled.load(Ordering::SeqCst),
                backend_available: false,
                target_display: "primary".into(),
                input_width: None,
                input_height: None,
                pixel_width: None,
                pixel_height: None,
                capture_permission: "unsupported".into(),
                input_permission: "unsupported".into(),
                control_owner: "user".into(),
                error: Some("Computer Use is unavailable in this host".into()),
            };
        }
        let mut input_width = None;
        let mut input_height = None;
        let mut pixel_width = None;
        let mut pixel_height = None;
        let (capture_status, input_status) = EnigoBackend::permission_info();
        let mut capture_permission = permission_name(capture_status).to_string();
        let mut input_permission = permission_name(input_status).to_string();
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
            control_owner: if enabled
                && backend_available
                && self.inner.control_owner.load(Ordering::SeqCst) == CONTROL_AGENT
            {
                "agent"
            } else {
                "user"
            }
            .into(),
            error,
        }
    }

    /// Tools are advertised only when the host, setting, backend, control
    /// owner, and selected model all allow Computer Use.
    pub(crate) fn advertises_tools(&self, model_supports_computer_use: bool) -> bool {
        if !self.inner.host_available || !model_supports_computer_use {
            return false;
        }
        let info = self.info();
        info.enabled && info.backend_available && info.control_owner == "agent"
    }

    /// Runs one action on the executor, honoring cancellation and control.
    pub(crate) async fn run_action(
        &self,
        member: &str,
        input: &Value,
        cancellation: &dyn Cancellation,
    ) -> Result<ActionOutcome, BusinessError> {
        self.ensure_host_available()?;
        if !self.inner.enabled.load(Ordering::SeqCst) {
            return Err(BusinessError::new(
                "computer_use_disabled",
                "Computer Use is disabled",
            ));
        }
        if self.inner.control_owner.load(Ordering::SeqCst) != CONTROL_AGENT {
            return Err(BusinessError::new(
                "computer_control_unavailable",
                "Computer Use is under user control",
            ));
        }
        let action = ComputerAction::from_member(member, input).map_err(computer_error)?;
        // The executor polls this flag; cancellation flips it from here.
        let cancelled = Arc::new(AtomicBool::new(false));
        let inner = self.inner.clone();
        let work = {
            let cancelled = cancelled.clone();
            run_blocking("suncode-computer", move || {
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
                        cancelled.load(Ordering::SeqCst)
                            || !inner.enabled.load(Ordering::SeqCst)
                            || inner.control_owner.load(Ordering::SeqCst) != CONTROL_AGENT
                    })
                    .map_err(computer_error)
            })
        };
        let mut work = std::pin::pin!(work);
        let mut cancel = cancellation.cancelled();
        let first = std::future::poll_fn(|cx| {
            if let Poll::Ready(result) = work.as_mut().poll(cx) {
                return Poll::Ready(Some(result));
            }
            cancel.as_mut().poll(cx).map(|()| None)
        })
        .await;
        let result = match first {
            Some(result) => result,
            None => {
                cancelled.store(true, Ordering::SeqCst);
                work.await
            }
        };
        result.unwrap_or_else(|| {
            Err(BusinessError::unavailable(
                "Computer Use executor stopped unexpectedly",
            ))
        })
    }

    pub(crate) fn persist_screenshot(&self, png: &[u8]) -> Result<String, BusinessError> {
        if png.is_empty() || png.len() > MAX_PROVIDER_PNG_BYTES {
            return Err(BusinessError::new(
                "computer_capture_failed",
                "Computer screenshot is outside the supported PNG size limit",
            ));
        }
        let root = self.inner.data_dir.join("computer");
        let directory = root.join("artifacts");
        prepare_artifact_directory(&directory, &root)?;
        let artifact_id = suncode_common::id::with_prefix("computer");
        fs::write(directory.join(format!("{artifact_id}.png")), png).map_err(|error| {
            BusinessError::new(
                "computer_artifact_failed",
                format!("Computer screenshot could not be stored: {error}"),
            )
        })?;
        Ok(artifact_id)
    }

    pub fn ensure_host_available(&self) -> Result<(), BusinessError> {
        if self.inner.host_available {
            Ok(())
        } else {
            Err(BusinessError::new(
                "computer_host_unavailable",
                "Computer Use is unavailable in this host",
            ))
        }
    }
}

pub(crate) fn computer_tool_definitions() -> Vec<ToolDefinition> {
    let empty = || json!({});
    let point = || {
        json!({
            "type": "object",
            "properties": {
                "x": {"type": "integer", "minimum": 0},
                "y": {"type": "integer", "minimum": 0}
            },
            "required": ["x", "y"],
            "additionalProperties": false
        })
    };
    let region = || {
        json!({
            "type": "object",
            "properties": {
                "x": {"type": "integer", "minimum": 0},
                "y": {"type": "integer", "minimum": 0},
                "width": {"type": "integer", "minimum": 1},
                "height": {"type": "integer", "minimum": 1}
            },
            "required": ["x", "y", "width", "height"],
            "additionalProperties": false
        })
    };
    let modifiers = || {
        json!({
            "type": "array",
            "items": {"type": "string", "enum": ["shift", "control", "alt", "super"]},
            "uniqueItems": true
        })
    };
    let definition =
        |member: &str, description: &str, properties: Value, required: &[&str]| ToolDefinition {
            name: format!("computer_{member}"),
            description: description.into(),
            parameters: json!({
                "type": "object",
                "properties": properties,
                "required": required,
                "additionalProperties": false
            }),
        };
    vec![
        definition(
            "screenshot",
            "Capture the current primary display and return it as an image.",
            empty(),
            &[],
        ),
        definition(
            "zoom",
            "Crop a region from the most recent screenshot.",
            json!({"region": region()}),
            &["region"],
        ),
        definition(
            "left_click",
            "Click the left mouse button, optionally at a screenshot coordinate.",
            json!({"coordinate": point(), "modifiers": modifiers()}),
            &[],
        ),
        definition(
            "right_click",
            "Click the right mouse button, optionally at a screenshot coordinate.",
            json!({"coordinate": point(), "modifiers": modifiers()}),
            &[],
        ),
        definition(
            "middle_click",
            "Click the middle mouse button, optionally at a screenshot coordinate.",
            json!({"coordinate": point(), "modifiers": modifiers()}),
            &[],
        ),
        definition(
            "double_click",
            "Double-click the left mouse button, optionally at a screenshot coordinate.",
            json!({"coordinate": point(), "modifiers": modifiers()}),
            &[],
        ),
        definition(
            "triple_click",
            "Triple-click the left mouse button, optionally at a screenshot coordinate.",
            json!({"coordinate": point(), "modifiers": modifiers()}),
            &[],
        ),
        definition(
            "left_click_drag",
            "Drag the left mouse button between two screenshot coordinates.",
            json!({"start_coordinate": point(), "coordinate": point(), "modifiers": modifiers()}),
            &["start_coordinate", "coordinate"],
        ),
        definition(
            "mouse_move",
            "Move the pointer to a screenshot coordinate.",
            json!({"coordinate": point()}),
            &["coordinate"],
        ),
        definition(
            "left_mouse_down",
            "Press and hold the left mouse button.",
            empty(),
            &[],
        ),
        definition(
            "left_mouse_up",
            "Release the left mouse button.",
            empty(),
            &[],
        ),
        definition(
            "cursor_position",
            "Return the current pointer position.",
            empty(),
            &[],
        ),
        definition(
            "scroll",
            "Scroll at an optional screenshot coordinate.",
            json!({"direction": {"type": "string", "enum": ["up", "down", "left", "right"]}, "scroll_amount": {"type": "integer", "minimum": 1}, "coordinate": point(), "modifiers": modifiers()}),
            &["direction", "scroll_amount"],
        ),
        definition(
            "type",
            "Type text into the focused desktop application.",
            json!({"text": {"type": "string"}}),
            &["text"],
        ),
        definition(
            "key",
            "Press a key or key chord one or more times.",
            json!({"text": {"type": "string"}, "repeat": {"type": "integer", "minimum": 1, "maximum": 100}}),
            &["text"],
        ),
        definition(
            "hold_key",
            "Hold a key chord for a bounded duration.",
            json!({"text": {"type": "string"}, "duration_seconds": {"type": "number", "minimum": 0, "maximum": 300}}),
            &["text", "duration_seconds"],
        ),
        definition(
            "wait",
            "Wait for a bounded duration before the next desktop action.",
            json!({"duration_seconds": {"type": "number", "minimum": 0, "maximum": 300}}),
            &["duration_seconds"],
        ),
    ]
}

fn computer_error(error: crate::ComputerError) -> BusinessError {
    BusinessError::new("computer_action_failed", error.to_string())
}

fn prepare_artifact_directory(path: &Path, root: &Path) -> Result<(), BusinessError> {
    fs::create_dir_all(root).map_err(|error| {
        BusinessError::new(
            "computer_artifact_failed",
            format!("Computer data root could not be created: {error}"),
        )
    })?;
    if fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
    {
        return Err(BusinessError::new(
            "computer_artifact_failed",
            "Computer artifact directory must not be a symbolic link",
        ));
    }
    fs::create_dir_all(path).map_err(|error| {
        BusinessError::new(
            "computer_artifact_failed",
            format!("Computer artifact directory could not be created: {error}"),
        )
    })?;
    let canonical_root = fs::canonicalize(root).map_err(|error| {
        BusinessError::new(
            "computer_artifact_failed",
            format!("Computer data root could not be resolved: {error}"),
        )
    })?;
    let canonical_path = fs::canonicalize(path).map_err(|error| {
        BusinessError::new(
            "computer_artifact_failed",
            format!("Computer artifact directory could not be resolved: {error}"),
        )
    })?;
    if !canonical_path.starts_with(canonical_root) {
        return Err(BusinessError::new(
            "computer_artifact_failed",
            "Computer artifact directory escaped its managed root",
        ));
    }
    Ok(())
}

const fn permission_name(permission: PermissionState) -> &'static str {
    match permission {
        PermissionState::Allowed => "allowed",
        PermissionState::Denied => "denied",
        PermissionState::Unknown => "unknown",
        PermissionState::Unsupported => "unsupported",
    }
}
