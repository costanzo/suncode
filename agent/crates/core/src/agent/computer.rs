use super::*;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use suncode_computer::{
    ComputerAction, ComputerBackend, ComputerExecutor, EnigoBackend, PermissionState,
};

pub(super) fn is_computer_call(call: &ToolCall) -> bool {
    call.toolset_name.as_deref() == Some("computer")
        || call
            .name
            .strip_prefix("computer_")
            .is_some_and(is_known_member)
}

pub(super) fn risk(name: &str) -> Option<Risk> {
    match computer_member(name)? {
        "screenshot" | "zoom" | "cursor_position" | "wait" | "mouse_move" => {
            Some(Risk::ComputerObserve)
        }
        "left_click" | "right_click" | "middle_click" | "double_click" | "triple_click"
        | "left_click_drag" | "left_mouse_down" | "left_mouse_up" | "scroll" | "type" | "key"
        | "hold_key" => Some(Risk::ComputerInput),
        _ => None,
    }
}

pub(super) fn member_name(call: &ToolCall) -> Option<&str> {
    if call.toolset_name.as_deref() == Some("computer") {
        return is_known_member(&call.name).then_some(call.name.as_str());
    }
    call.name
        .strip_prefix("computer_")
        .filter(|name| is_known_member(name))
}

fn computer_member(name: &str) -> Option<&str> {
    if is_known_member(name) {
        return Some(name);
    }
    name.strip_prefix("computer_")
        .filter(|member| is_known_member(member))
}

fn is_known_member(name: &str) -> bool {
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
pub(super) struct ComputerManager {
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

impl ComputerManager {
    pub(super) fn new(store: &Store, host_available: bool, data_dir: PathBuf) -> Self {
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
                data_dir,
                enabled: AtomicBool::new(enabled),
                host_available,
                control_owner: AtomicU8::new(if enabled { CONTROL_AGENT } else { CONTROL_USER }),
                executor: Mutex::new(None),
            }),
        }
    }

    pub(super) fn install_backend(
        &self,
        backend: Box<dyn ComputerBackend>,
    ) -> Result<(), BusinessError> {
        self.ensure_host_available()?;
        let mut executor =
            self.inner.executor.lock().map_err(|_| {
                BusinessError::unavailable("Computer Use backend lock is unavailable")
            })?;
        *executor = Some(ComputerExecutor::new(backend));
        Ok(())
    }

    pub(super) fn set_enabled(&self, enabled: bool) -> Result<(), BusinessError> {
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

    pub(super) fn emergency_stop(&self) -> Result<(), BusinessError> {
        self.set_enabled(false)
    }

    pub(super) fn take_user_control(&self) -> Result<(), BusinessError> {
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

    pub(super) fn return_agent_control(&self) -> Result<(), BusinessError> {
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

    pub(super) fn info(&self) -> ComputerRuntimeInfo {
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

    pub(super) fn catalog(
        &self,
        model_supports_computer_use: bool,
    ) -> Vec<suncode_llm::ToolDefinition> {
        if !self.inner.host_available {
            return Vec::new();
        }
        let info = self.info();
        if !info.enabled
            || !info.backend_available
            || info.control_owner != "agent"
            || !model_supports_computer_use
        {
            return Vec::new();
        }
        computer_tool_definitions()
    }

    pub(super) async fn execute(
        &self,
        member: &str,
        input: &Value,
        cancellation: CancellationToken,
    ) -> Result<suncode_computer::ActionOutcome, BusinessError> {
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
                    cancellation.is_cancelled()
                        || !inner.enabled.load(Ordering::SeqCst)
                        || inner.control_owner.load(Ordering::SeqCst) != CONTROL_AGENT
                })
                .map_err(computer_error)
        })
        .await
        .map_err(|_| BusinessError::unavailable("Computer Use executor stopped unexpectedly"))?
    }

    pub(super) fn persist_screenshot(&self, png: &[u8]) -> Result<String, BusinessError> {
        if png.is_empty() || png.len() > suncode_computer::MAX_PROVIDER_PNG_BYTES {
            return Err(BusinessError::new(
                "computer_capture_failed",
                "Computer screenshot is outside the supported PNG size limit",
            ));
        }
        let root = self.inner.data_dir.join("computer");
        let directory = root.join("artifacts");
        prepare_artifact_directory(&directory, &root)?;
        let artifact_id = format!("computer-{}", Uuid::new_v4());
        fs::write(directory.join(format!("{artifact_id}.png")), png).map_err(|error| {
            BusinessError::new(
                "computer_artifact_failed",
                format!("Computer screenshot could not be stored: {error}"),
            )
        })?;
        Ok(artifact_id)
    }

    pub(super) fn ensure_host_available(&self) -> Result<(), BusinessError> {
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

fn computer_tool_definitions() -> Vec<suncode_llm::ToolDefinition> {
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
    let definition = |member: &str, description: &str, properties: Value, required: &[&str]| {
        suncode_llm::ToolDefinition {
            name: format!("computer_{member}"),
            description: description.into(),
            parameters: json!({
                "type": "object",
                "properties": properties,
                "required": required,
                "additionalProperties": false
            }),
        }
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

    pub fn take_computer_control(&self) -> Result<ComputerRuntimeInfo, BusinessError> {
        self.computer.take_user_control()?;
        Ok(self.computer.info())
    }

    pub fn return_computer_control(&self) -> Result<ComputerRuntimeInfo, BusinessError> {
        self.computer.return_agent_control()?;
        Ok(self.computer.info())
    }

    pub fn request_computer_capture_permission(
        &self,
    ) -> Result<ComputerRuntimeInfo, BusinessError> {
        self.computer.ensure_host_available()?;
        let _ = EnigoBackend::request_capture_permission();
        Ok(self.computer.info())
    }

    pub fn request_computer_input_permission(&self) -> Result<ComputerRuntimeInfo, BusinessError> {
        self.computer.ensure_host_available()?;
        let _ = EnigoBackend::request_input_permission();
        Ok(self.computer.info())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_host_exposes_no_computer_toolset_or_backend() {
        let store = Store::open_memory().unwrap();
        store
            .set_setting(
                "global",
                "global",
                "computer_use_enabled",
                &Value::Bool(true),
            )
            .unwrap();
        let manager =
            ComputerManager::new(&store, false, tempfile::tempdir().unwrap().path().into());

        assert!(manager.catalog(true).is_empty());
        let info = manager.info();
        assert!(info.enabled);
        assert!(!info.backend_available);
        assert_eq!(info.capture_permission, "unsupported");
        assert_eq!(
            manager.set_enabled(true).unwrap_err().code,
            "computer_host_unavailable"
        );
    }

    #[test]
    fn generic_catalog_uses_stable_namespaced_function_tools() {
        let definitions = computer_tool_definitions();
        assert_eq!(definitions.len(), 17);
        assert!(definitions
            .iter()
            .all(|definition| definition.name.starts_with("computer_")));
        assert!(definitions
            .iter()
            .any(|definition| definition.name == "computer_screenshot"));
        for definition in definitions {
            assert!(
                jsonschema::validator_for(&definition.parameters).is_ok(),
                "invalid schema for {}: {}",
                definition.name,
                definition.parameters
            );
        }
    }

    #[test]
    fn persists_screenshot_artifacts_under_the_managed_computer_directory() {
        let store = Store::open_memory().unwrap();
        let data_dir = tempfile::tempdir().unwrap();
        let manager = ComputerManager::new(&store, false, data_dir.path().into());
        let artifact_id = manager.persist_screenshot(b"png-bytes").unwrap();
        let path = data_dir
            .path()
            .join("computer")
            .join("artifacts")
            .join(format!("{artifact_id}.png"));
        assert_eq!(std::fs::read(path).unwrap(), b"png-bytes");
    }

    #[test]
    fn generic_calls_map_to_existing_computer_members() {
        let call = ToolCall {
            call_id: "call-1".into(),
            name: "computer_left_click".into(),
            arguments: json!({}),
            toolset_name: None,
        };
        assert!(is_computer_call(&call));
        assert_eq!(member_name(&call), Some("left_click"));
        assert_eq!(risk(&call.name), Some(Risk::ComputerInput));
    }
}
