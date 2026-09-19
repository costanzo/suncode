use serde::Serialize;
use serde_json::{json, Value};
use std::{
    ffi::{CStr, CString},
    os::raw::{c_char, c_void},
    panic::{catch_unwind, AssertUnwindSafe},
    ptr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    thread::JoinHandle,
};
use suncode_common::BusinessError;
use suncode_sdk::logging_module::{self as logging, Level};
use suncode_sdk::{
    AgentEvent, AgentSdk, LanguageServerWriteRequest, McpServerWriteRequest, SdkResult,
    SessionEventStream, SessionEventStreamControl, SubscriptionError,
    SUNCODE_AGENT_SDK_ABI_VERSION,
};

pub type SunCodeEventCallback = unsafe extern "C" fn(*const c_char, *mut c_void);

pub struct SunCodeAgentHandle {
    sdk: AgentSdk,
}

pub struct SunCodeAgentSubscriptionHandle {
    control: SessionEventStreamControl,
    stream: Mutex<Option<SessionEventStream>>,
    join: Mutex<Option<JoinHandle<()>>>,
    callback: SunCodeEventCallback,
    user_data: usize,
    session_id: String,
    started: AtomicBool,
}

impl SunCodeAgentSubscriptionHandle {
    fn new(stream: SessionEventStream, callback: SunCodeEventCallback, user_data: usize) -> Self {
        let session_id = stream.session_id().to_string();
        let control = stream.control();
        Self {
            control,
            stream: Mutex::new(Some(stream)),
            join: Mutex::new(None),
            callback,
            user_data,
            session_id,
            started: AtomicBool::new(false),
        }
    }

    fn start(&self) -> SdkResult<()> {
        if self.started.swap(true, Ordering::AcqRel) {
            return Err(BusinessError::new(
                "conflict",
                "session subscription is already started",
            ));
        }
        let stream = self
            .stream
            .lock()
            .map_err(|_| BusinessError::unavailable("session subscription state unavailable"))?
            .take()
            .ok_or_else(|| BusinessError::new("conflict", "session subscription is unavailable"))?;
        let callback = self.callback;
        let user_data = self.user_data;
        let session_id = self.session_id.clone();
        let join = std::thread::Builder::new()
            .name("suncode-sdk-c-events".into())
            .spawn(move || run_subscription(stream, callback, user_data, session_id))
            .map_err(|error| {
                BusinessError::unavailable(format!(
                    "session event callback thread could not start: {error}"
                ))
            });
        match join {
            Ok(join) => {
                *self.join.lock().map_err(|_| {
                    BusinessError::unavailable("session subscription join state unavailable")
                })? = Some(join);
                Ok(())
            }
            Err(error) => {
                self.control.close();
                Err(error)
            }
        }
    }
}

impl Drop for SunCodeAgentSubscriptionHandle {
    fn drop(&mut self) {
        self.control.close();
        if let Ok(mut join) = self.join.lock() {
            if let Some(join) = join.take() {
                if join.thread().id() != std::thread::current().id() {
                    let _ = join.join();
                }
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn suncode_agent_sdk_abi_version() -> u32 {
    SUNCODE_AGENT_SDK_ABI_VERSION
}

#[no_mangle]
pub extern "C" fn suncode_agent_sdk_version() -> *mut c_char {
    match catch_unwind(AgentSdk::version) {
        Ok(version) => result_envelope::<_>(Ok(version)),
        Err(_) => result_envelope::<suncode_sdk::VersionResult>(Err(BusinessError::unavailable(
            "SDK version query panicked",
        ))),
    }
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_open_default(
    user_id: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut SunCodeAgentHandle {
    write_error_out(error_out, ptr::null_mut());
    match catch_unwind(AssertUnwindSafe(|| {
        let user_id = c_string(user_id, "user_id")?;
        AgentSdk::open_default(&user_id)
    })) {
        Ok(Ok(sdk)) => Box::into_raw(Box::new(SunCodeAgentHandle { sdk })),
        Ok(Err(error)) => {
            logging::write_business_error("sdk.open", "open_default", &error, "phase=initialize");
            write_error_out(error_out, into_c_string(error.to_string()));
            ptr::null_mut()
        }
        Err(_) => {
            logging::write(
                Level::Error,
                "sdk.open",
                "operation=open_default panic=true",
            );
            write_error_out(
                error_out,
                into_c_string("agent_unavailable: agent initialization panicked".to_string()),
            );
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_close(handle: *mut SunCodeAgentHandle) {
    if !handle.is_null() {
        logging::write(Level::Info, "sdk.close", "handle_close begin");
        let _ = catch_unwind(AssertUnwindSafe(|| drop(Box::from_raw(handle))));
        logging::write(Level::Info, "sdk.close", "handle_close end");
    }
}

macro_rules! ffi_no_args {
    ($function:ident, $method:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $function(handle: *mut SunCodeAgentHandle) -> *mut c_char {
            ffi_call(handle, |sdk| sdk.$method())
        }
    };
}

ffi_no_args!(suncode_agent_sdk_health, health);
ffi_no_args!(suncode_agent_sdk_diagnostics, diagnostics);
ffi_no_args!(suncode_agent_sdk_list_models, list_models);
ffi_no_args!(suncode_agent_sdk_list_credentials, list_credentials);
ffi_no_args!(suncode_agent_sdk_list_projects, list_projects);
ffi_no_args!(suncode_agent_sdk_list_agents, list_agents);
ffi_no_args!(
    suncode_agent_sdk_computer_runtime_info,
    computer_runtime_info
);
ffi_no_args!(
    suncode_agent_sdk_request_computer_capture_permission,
    request_computer_capture_permission
);
ffi_no_args!(
    suncode_agent_sdk_request_computer_input_permission,
    request_computer_input_permission
);
ffi_no_args!(
    suncode_agent_sdk_take_computer_control,
    take_computer_control
);
ffi_no_args!(
    suncode_agent_sdk_return_computer_control,
    return_computer_control
);

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_set_computer_use_enabled(
    handle: *mut SunCodeAgentHandle,
    enabled: u8,
) -> *mut c_char {
    ffi_call(handle, |sdk| sdk.set_computer_use_enabled(enabled != 0))
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_emergency_stop_computer_use(
    handle: *mut SunCodeAgentHandle,
) -> *mut c_char {
    ffi_call(handle, |sdk| sdk.emergency_stop_computer_use())
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_browser_runtime_info(
    handle: *mut SunCodeAgentHandle,
    project_id: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        let project_id = optional_c_string(project_id, "project_id")?;
        sdk.browser_runtime_info(project_id.as_deref())
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_set_browser_use_enabled(
    handle: *mut SunCodeAgentHandle,
    enabled: u8,
) -> *mut c_char {
    ffi_call(handle, |sdk| sdk.set_browser_use_enabled(enabled != 0))
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_verify_browser_runtime(
    handle: *mut SunCodeAgentHandle,
    project_id: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        let project_id = optional_c_string(project_id, "project_id")?;
        sdk.verify_browser_runtime(project_id.as_deref())
    })
}

macro_rules! browser_project_ffi {
    ($function:ident, $method:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $function(
            handle: *mut SunCodeAgentHandle,
            project_id: *const c_char,
        ) -> *mut c_char {
            ffi_call(handle, |sdk| {
                sdk.$method(&c_string(project_id, "project_id")?)
            })
        }
    };
}

browser_project_ffi!(
    suncode_agent_sdk_start_browser_project,
    start_browser_project
);
browser_project_ffi!(suncode_agent_sdk_take_browser_control, take_browser_control);
browser_project_ffi!(
    suncode_agent_sdk_return_browser_control,
    return_browser_control
);
browser_project_ffi!(
    suncode_agent_sdk_restart_browser_runtime,
    restart_browser_runtime
);
browser_project_ffi!(suncode_agent_sdk_stop_browser_runtime, stop_browser_runtime);
browser_project_ffi!(
    suncode_agent_sdk_clear_browser_profile,
    clear_browser_profile
);

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_list_mcp_servers(
    handle: *mut SunCodeAgentHandle,
    project_id: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        let project_id = optional_c_string(project_id, "project_id")?;
        sdk.list_mcp_servers(project_id.as_deref())
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_create_mcp_server(
    handle: *mut SunCodeAgentHandle,
    project_id: *const c_char,
    idempotency_key: *const c_char,
    request_json: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        let project_id = optional_c_string(project_id, "project_id")?;
        let request = typed_json_from_c::<McpServerWriteRequest>(request_json, "request_json")?;
        sdk.create_mcp_server(
            project_id.as_deref(),
            &c_string(idempotency_key, "idempotency_key")?,
            request,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_update_mcp_server(
    handle: *mut SunCodeAgentHandle,
    project_id: *const c_char,
    server_id: *const c_char,
    expected_revision: u64,
    idempotency_key: *const c_char,
    request_json: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        let project_id = optional_c_string(project_id, "project_id")?;
        sdk.update_mcp_server(
            project_id.as_deref(),
            &c_string(server_id, "server_id")?,
            expected_revision,
            &c_string(idempotency_key, "idempotency_key")?,
            typed_json_from_c::<McpServerWriteRequest>(request_json, "request_json")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_set_mcp_server_enabled(
    handle: *mut SunCodeAgentHandle,
    project_id: *const c_char,
    server_id: *const c_char,
    expected_revision: u64,
    idempotency_key: *const c_char,
    enabled: u8,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        let project_id = optional_c_string(project_id, "project_id")?;
        sdk.set_mcp_server_enabled(
            project_id.as_deref(),
            &c_string(server_id, "server_id")?,
            expected_revision,
            &c_string(idempotency_key, "idempotency_key")?,
            enabled != 0,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_delete_mcp_server(
    handle: *mut SunCodeAgentHandle,
    server_id: *const c_char,
    expected_revision: u64,
    idempotency_key: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.delete_mcp_server(
            &c_string(server_id, "server_id")?,
            expected_revision,
            &c_string(idempotency_key, "idempotency_key")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_retry_mcp_server(
    handle: *mut SunCodeAgentHandle,
    project_id: *const c_char,
    server_id: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.retry_mcp_server(
            &c_string(project_id, "project_id")?,
            &c_string(server_id, "server_id")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_start_mcp_project(
    handle: *mut SunCodeAgentHandle,
    project_id: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.start_mcp_project(&c_string(project_id, "project_id")?)
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_mcp_load_progress(
    handle: *mut SunCodeAgentHandle,
    project_id: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        Ok(sdk.mcp_load_progress(&c_string(project_id, "project_id")?))
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_list_language_servers(
    handle: *mut SunCodeAgentHandle,
    project_id: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        let project_id = optional_c_string(project_id, "project_id")?;
        sdk.list_language_servers(project_id.as_deref())
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_create_language_server(
    handle: *mut SunCodeAgentHandle,
    project_id: *const c_char,
    idempotency_key: *const c_char,
    request_json: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        let project_id = optional_c_string(project_id, "project_id")?;
        sdk.create_language_server(
            project_id.as_deref(),
            &c_string(idempotency_key, "idempotency_key")?,
            typed_json_from_c::<LanguageServerWriteRequest>(request_json, "request_json")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_update_language_server(
    handle: *mut SunCodeAgentHandle,
    project_id: *const c_char,
    language_server_id: *const c_char,
    expected_revision: u64,
    idempotency_key: *const c_char,
    request_json: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        let project_id = optional_c_string(project_id, "project_id")?;
        sdk.update_language_server(
            project_id.as_deref(),
            &c_string(language_server_id, "language_server_id")?,
            expected_revision,
            &c_string(idempotency_key, "idempotency_key")?,
            typed_json_from_c::<LanguageServerWriteRequest>(request_json, "request_json")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_set_language_server_enabled(
    handle: *mut SunCodeAgentHandle,
    project_id: *const c_char,
    language_server_id: *const c_char,
    expected_revision: u64,
    idempotency_key: *const c_char,
    enabled: u8,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        let project_id = optional_c_string(project_id, "project_id")?;
        sdk.set_language_server_enabled(
            project_id.as_deref(),
            &c_string(language_server_id, "language_server_id")?,
            expected_revision,
            &c_string(idempotency_key, "idempotency_key")?,
            enabled != 0,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_delete_language_server(
    handle: *mut SunCodeAgentHandle,
    language_server_id: *const c_char,
    expected_revision: u64,
    idempotency_key: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.delete_language_server(
            &c_string(language_server_id, "language_server_id")?,
            expected_revision,
            &c_string(idempotency_key, "idempotency_key")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_retry_language_server(
    handle: *mut SunCodeAgentHandle,
    project_id: *const c_char,
    language_server_id: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.retry_language_server(
            &c_string(project_id, "project_id")?,
            &c_string(language_server_id, "language_server_id")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_start_language_server_project(
    handle: *mut SunCodeAgentHandle,
    project_id: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.start_language_server_project(&c_string(project_id, "project_id")?)
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_list_project_dependencies(
    handle: *mut SunCodeAgentHandle,
    project_id: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.list_project_dependencies(&c_string(project_id, "project_id")?)
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_add_project_dependency(
    handle: *mut SunCodeAgentHandle,
    project_id: *const c_char,
    path: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.add_project_dependency(
            &c_string(project_id, "project_id")?,
            &c_string(path, "path")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_remove_project_dependency(
    handle: *mut SunCodeAgentHandle,
    project_id: *const c_char,
    dependency_id: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.remove_project_dependency(
            &c_string(project_id, "project_id")?,
            &c_string(dependency_id, "dependency_id")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_list_project_directory(
    handle: *mut SunCodeAgentHandle,
    project_id: *const c_char,
    dependency_id: *const c_char,
    path: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        let dependency_id = optional_c_string(dependency_id, "dependency_id")?;
        sdk.list_project_directory(
            &c_string(project_id, "project_id")?,
            dependency_id.as_deref(),
            &c_string(path, "path")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_read_project_file(
    handle: *mut SunCodeAgentHandle,
    project_id: *const c_char,
    dependency_id: *const c_char,
    path: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        let dependency_id = optional_c_string(dependency_id, "dependency_id")?;
        sdk.read_project_file(
            &c_string(project_id, "project_id")?,
            dependency_id.as_deref(),
            &c_string(path, "path")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_list_settings(
    handle: *mut SunCodeAgentHandle,
    project_id: *const c_char,
    session_id: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        let project_id = optional_c_string(project_id, "project_id")?;
        let session_id = optional_c_string(session_id, "session_id")?;
        sdk.list_settings(project_id.as_deref(), session_id.as_deref())
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_set_setting(
    handle: *mut SunCodeAgentHandle,
    scope: *const c_char,
    project_id: *const c_char,
    session_id: *const c_char,
    key: *const c_char,
    value_json: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        let scope = c_string(scope, "scope")?;
        let project_id = optional_c_string(project_id, "project_id")?;
        let session_id = optional_c_string(session_id, "session_id")?;
        let key = c_string(key, "key")?;
        let value_document = json_from_c(value_json, "value_json")?;
        let value = value_document
            .get("value")
            .cloned()
            .unwrap_or(value_document);
        sdk.set_setting(
            &scope,
            project_id.as_deref(),
            session_id.as_deref(),
            &key,
            &value,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_set_proxy_configuration(
    handle: *mut SunCodeAgentHandle,
    request_json: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        let request = serde_json::from_value::<suncode_sdk::ProxyConfigurationRequest>(
            json_from_c(request_json, "request_json")?,
        )
        .map_err(|_| BusinessError::invalid("proxy configuration request is invalid"))?;
        sdk.set_proxy_configuration(request)
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_set_credential(
    handle: *mut SunCodeAgentHandle,
    provider: *const c_char,
    api_key: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.set_credential(
            &c_string(provider, "provider")?,
            &c_string(api_key, "api_key")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_remove_credential(
    handle: *mut SunCodeAgentHandle,
    provider: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.remove_credential(&c_string(provider, "provider")?)
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_set_provider_endpoint(
    handle: *mut SunCodeAgentHandle,
    provider: *const c_char,
    endpoint: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.set_provider_endpoint(
            &c_string(provider, "provider")?,
            &c_string(endpoint, "endpoint")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_open_project(
    handle: *mut SunCodeAgentHandle,
    path: *const c_char,
    display_name: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        let path = c_string(path, "path")?;
        let display_name = optional_c_string(display_name, "display_name")?;
        sdk.open_project(&path, display_name.as_deref())
    })
}

macro_rules! ffi_one_string {
    ($function:ident, $method:ident, $argument:literal) => {
        #[no_mangle]
        pub unsafe extern "C" fn $function(
            handle: *mut SunCodeAgentHandle,
            value: *const c_char,
        ) -> *mut c_char {
            ffi_call(handle, |sdk| sdk.$method(&c_string(value, $argument)?))
        }
    };
}

ffi_one_string!(
    suncode_agent_sdk_select_project,
    select_project,
    "project_id"
);
ffi_one_string!(suncode_agent_sdk_git_status, git_status, "project_id");
ffi_one_string!(suncode_agent_sdk_list_sessions, list_sessions, "project_id");
ffi_one_string!(
    suncode_agent_sdk_list_child_sessions,
    list_child_sessions,
    "parent_session_id"
);
ffi_one_string!(
    suncode_agent_sdk_archive_session,
    archive_session,
    "session_id"
);

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_set_session_pinned(
    handle: *mut SunCodeAgentHandle,
    session_id: *const c_char,
    pinned: u8,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.set_session_pinned(&c_string(session_id, "session_id")?, pinned != 0)
    })
}
ffi_one_string!(
    suncode_agent_sdk_reopen_session,
    reopen_session,
    "session_id"
);
ffi_one_string!(
    suncode_agent_sdk_list_checkpoints,
    list_checkpoints,
    "session_id"
);
ffi_one_string!(suncode_agent_sdk_session_usage, session_usage, "session_id");
ffi_one_string!(
    suncode_agent_sdk_list_provider_exchanges,
    list_provider_exchanges,
    "session_id"
);
ffi_one_string!(
    suncode_agent_sdk_checkpoint_manifest,
    checkpoint_manifest,
    "manifest_id"
);
ffi_one_string!(suncode_agent_sdk_get_approval, get_approval, "approval_id");

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_create_session(
    handle: *mut SunCodeAgentHandle,
    project_id: *const c_char,
    title: *const c_char,
    model: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        let project_id = c_string(project_id, "project_id")?;
        let title = optional_c_string(title, "title")?;
        let model = optional_c_string(model, "model")?;
        sdk.create_session(&project_id, title.as_deref(), model.as_deref())
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_git_diff_file(
    handle: *mut SunCodeAgentHandle,
    project_id: *const c_char,
    scope: *const c_char,
    path: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.git_diff_file(
            &c_string(project_id, "project_id")?,
            &c_string(scope, "scope")?,
            &c_string(path, "path")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_rename_session(
    handle: *mut SunCodeAgentHandle,
    session_id: *const c_char,
    title: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.rename_session(
            &c_string(session_id, "session_id")?,
            &c_string(title, "title")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_session_snapshot(
    handle: *mut SunCodeAgentHandle,
    session_id: *const c_char,
    after: i64,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.session_snapshot(&c_string(session_id, "session_id")?, after)
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_list_session_images(
    handle: *mut SunCodeAgentHandle,
    session_id: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.list_session_images(&c_string(session_id, "session_id")?)
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_add_session_image(
    handle: *mut SunCodeAgentHandle,
    session_id: *const c_char,
    image_json: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.add_session_image(
            &c_string(session_id, "session_id")?,
            &json_from_c(image_json, "image_json")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_remove_session_image(
    handle: *mut SunCodeAgentHandle,
    session_id: *const c_char,
    image_id: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.remove_session_image(
            &c_string(session_id, "session_id")?,
            &c_string(image_id, "image_id")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_provider_exchange(
    handle: *mut SunCodeAgentHandle,
    session_id: *const c_char,
    exchange_id: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.provider_exchange(
            &c_string(session_id, "session_id")?,
            &c_string(exchange_id, "exchange_id")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_restore_checkpoint(
    handle: *mut SunCodeAgentHandle,
    manifest_id: *const c_char,
    session_id: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.restore_checkpoint(
            &c_string(manifest_id, "manifest_id")?,
            &c_string(session_id, "session_id")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_submit_turn(
    handle: *mut SunCodeAgentHandle,
    session_id: *const c_char,
    input: *const c_char,
    idempotency_key: *const c_char,
    model: *const c_char,
    reasoning_effort: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        let session_id = c_string(session_id, "session_id")?;
        let input = c_string(input, "input")?;
        let idempotency_key = c_string(idempotency_key, "idempotency_key")?;
        let model = optional_c_string(model, "model")?;
        let reasoning_effort = optional_c_string(reasoning_effort, "reasoning_effort")?;
        sdk.submit_turn(
            &session_id,
            &input,
            &idempotency_key,
            model.as_deref(),
            reasoning_effort.as_deref(),
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_submit_turn_with_attachments(
    handle: *mut SunCodeAgentHandle,
    session_id: *const c_char,
    input: *const c_char,
    idempotency_key: *const c_char,
    model: *const c_char,
    reasoning_effort: *const c_char,
    image_ids_json: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        let session_id = c_string(session_id, "session_id")?;
        let input = c_string(input, "input")?;
        let idempotency_key = c_string(idempotency_key, "idempotency_key")?;
        let model = optional_c_string(model, "model")?;
        let reasoning_effort = optional_c_string(reasoning_effort, "reasoning_effort")?;
        let image_ids_json = c_string(image_ids_json, "image_ids_json")?;
        let image_ids: Vec<String> = serde_json::from_str(&image_ids_json)
            .map_err(|_| BusinessError::invalid("image_ids_json must be an array of strings"))?;
        sdk.submit_turn_with_attachments(
            &session_id,
            &input,
            &idempotency_key,
            model.as_deref(),
            reasoning_effort.as_deref(),
            &image_ids,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_cancel_turn(
    handle: *mut SunCodeAgentHandle,
    session_id: *const c_char,
    turn_id: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.cancel_turn(
            &c_string(session_id, "session_id")?,
            &c_string(turn_id, "turn_id")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_retry_last_turn(
    handle: *mut SunCodeAgentHandle,
    session_id: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.retry_last_turn(&c_string(session_id, "session_id")?)
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_resolve_approval(
    handle: *mut SunCodeAgentHandle,
    approval_id: *const c_char,
    decision: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.resolve_approval(
            &c_string(approval_id, "approval_id")?,
            &c_string(decision, "decision")?,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_reply_question(
    handle: *mut SunCodeAgentHandle,
    request_id: *const c_char,
    answers_json: *const c_char,
) -> *mut c_char {
    ffi_call(handle, |sdk| {
        sdk.reply_question(
            &c_string(request_id, "request_id")?,
            &json_from_c(answers_json, "answers_json")?,
        )
    })
}

ffi_one_string!(
    suncode_agent_sdk_reject_question,
    reject_question,
    "request_id"
);

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_subscribe_session(
    handle: *mut SunCodeAgentHandle,
    session_id: *const c_char,
    _after: i64,
    callback: Option<SunCodeEventCallback>,
    user_data: *mut c_void,
    error_out: *mut *mut c_char,
) -> *mut SunCodeAgentSubscriptionHandle {
    write_error_out(error_out, ptr::null_mut());
    let result = catch_unwind(AssertUnwindSafe(|| -> SdkResult<_> {
        let handle = handle
            .as_ref()
            .ok_or_else(|| BusinessError::unavailable("agent handle is null"))?;
        let callback = callback.ok_or_else(|| BusinessError::invalid("callback is null"))?;
        let session_id = c_string(session_id, "session_id")?;
        let stream = handle.sdk.subscribe_session_events(&session_id)?;
        let subscription =
            SunCodeAgentSubscriptionHandle::new(stream, callback, user_data as usize);
        subscription.start()?;
        Ok(subscription)
    }));
    match result {
        Ok(Ok(subscription)) => Box::into_raw(Box::new(subscription)),
        Ok(Err(error)) => {
            logging::write_business_error(
                "sdk.subscribe",
                "subscribe_session",
                &error,
                "boundary=native",
            );
            write_error_out(error_out, into_c_string(error.to_string()));
            ptr::null_mut()
        }
        Err(_) => {
            logging::write(
                Level::Error,
                "sdk.subscribe",
                "operation=subscribe_session panic=true",
            );
            write_error_out(
                error_out,
                into_c_string("agent_unavailable: subscription panicked".to_string()),
            );
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_watch_session(
    handle: *mut SunCodeAgentHandle,
    session_id: *const c_char,
    callback: Option<SunCodeEventCallback>,
    user_data: *mut c_void,
    snapshot_out: *mut *mut c_char,
    error_out: *mut *mut c_char,
) -> *mut SunCodeAgentSubscriptionHandle {
    write_error_out(snapshot_out, ptr::null_mut());
    write_error_out(error_out, ptr::null_mut());
    let result = catch_unwind(AssertUnwindSafe(|| -> SdkResult<_> {
        let handle = handle
            .as_ref()
            .ok_or_else(|| BusinessError::unavailable("agent handle is null"))?;
        let callback = callback.ok_or_else(|| BusinessError::invalid("callback is null"))?;
        let session_id = c_string(session_id, "session_id")?;
        let watch = handle.sdk.watch_session(&session_id)?;
        let snapshot = serde_json::to_string(&watch.snapshot).map_err(|error| {
            BusinessError::unavailable(format!("session snapshot serialization failed: {error}"))
        })?;
        Ok((
            SunCodeAgentSubscriptionHandle::new(watch.events, callback, user_data as usize),
            snapshot,
        ))
    }));
    match result {
        Ok(Ok((subscription, snapshot))) => {
            write_error_out(snapshot_out, into_c_string(snapshot));
            Box::into_raw(Box::new(subscription))
        }
        Ok(Err(error)) => {
            logging::write_business_error("sdk.watch", "watch_session", &error, "boundary=native");
            write_error_out(error_out, into_c_string(error.to_string()));
            ptr::null_mut()
        }
        Err(_) => {
            logging::write(
                Level::Error,
                "sdk.watch",
                "operation=watch_session panic=true",
            );
            write_error_out(
                error_out,
                into_c_string("agent_unavailable: session watch panicked".to_string()),
            );
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_subscription_start(
    subscription: *mut SunCodeAgentSubscriptionHandle,
    error_out: *mut *mut c_char,
) -> u8 {
    write_error_out(error_out, ptr::null_mut());
    let result = catch_unwind(AssertUnwindSafe(|| -> SdkResult<()> {
        let subscription = subscription
            .as_ref()
            .ok_or_else(|| BusinessError::unavailable("subscription handle is null"))?;
        subscription.start()
    }));
    match result {
        Ok(Ok(())) => 1,
        Ok(Err(error)) => {
            logging::write_business_error(
                "sdk.subscription",
                "subscription_start",
                &error,
                "boundary=native",
            );
            write_error_out(error_out, into_c_string(error.to_string()));
            0
        }
        Err(_) => {
            write_error_out(
                error_out,
                into_c_string("agent_unavailable: subscription start panicked".to_string()),
            );
            0
        }
    }
}

fn run_subscription(
    mut stream: SessionEventStream,
    callback: SunCodeEventCallback,
    user_data: usize,
    session_id: String,
) {
    loop {
        match stream.blocking_recv() {
            Ok(event) => emit_agent_event(callback, user_data, &event),
            Err(SubscriptionError::Lagged { missed }) => {
                logging::write(
                    Level::Warn,
                    "sdk.subscribe",
                    format!("lagged session={session_id} missed={missed}"),
                );
                emit_resync_required(callback, user_data, &session_id, missed);
                break;
            }
            Err(SubscriptionError::Closed) => {
                logging::write(
                    Level::Debug,
                    "sdk.subscribe",
                    format!("thread_exit session={session_id} reason=closed"),
                );
                break;
            }
            Err(SubscriptionError::Empty) => continue,
        }
    }
}

fn emit_agent_event(callback: SunCodeEventCallback, user_data: usize, event: &AgentEvent) {
    let value = json!({
        "session_id": event.session_id,
        "occurred_at": event.occurred_at,
        "event_type": event.event_type().as_str(),
        "payload": event.payload.clone().into_value(),
    });
    emit_event_json(callback, user_data, value);
}

fn emit_resync_required(
    callback: SunCodeEventCallback,
    user_data: usize,
    session_id: &str,
    missed: u64,
) {
    emit_event_json(
        callback,
        user_data,
        json!({
            "session_id": session_id,
            "occurred_at": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            "event_type": "resync.required",
            "payload": {"reason": "subscriber_lagged", "missed": missed},
        }),
    );
}

fn emit_event_json(callback: SunCodeEventCallback, user_data: usize, value: Value) {
    let Ok(value) = CString::new(value.to_string()) else {
        logging::write(
            Level::Error,
            "sdk.event",
            "operation=marshal_event failed=true boundary=native",
        );
        return;
    };
    unsafe { callback(value.as_ptr(), user_data as *mut c_void) };
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_subscription_close(
    subscription: *mut SunCodeAgentSubscriptionHandle,
) {
    if !subscription.is_null() {
        let _ = catch_unwind(AssertUnwindSafe(|| drop(Box::from_raw(subscription))));
    }
}

#[no_mangle]
pub unsafe extern "C" fn suncode_agent_sdk_string_free(value: *mut c_char) {
    if !value.is_null() {
        drop(CString::from_raw(value));
    }
}

unsafe fn ffi_call<F, T>(handle: *mut SunCodeAgentHandle, call: F) -> *mut c_char
where
    F: FnOnce(&AgentSdk) -> SdkResult<T>,
    T: Serialize,
{
    let result = catch_unwind(AssertUnwindSafe(|| {
        let handle = handle
            .as_ref()
            .ok_or_else(|| BusinessError::unavailable("agent handle is null"))?;
        call(&handle.sdk)
    }));
    match result {
        Ok(Ok(value)) => result_envelope(Ok(value)),
        Ok(Err(error)) => {
            logging::write_business_error("sdk.ffi", "ffi_call", &error, "boundary=native");
            result_envelope::<T>(Err(error))
        }
        Err(_) => {
            logging::write(
                Level::Error,
                "sdk.ffi",
                "operation=ffi_call panic=true boundary=native",
            );
            result_envelope::<T>(Err(BusinessError::unavailable("SDK call panicked")))
        }
    }
}

fn result_envelope<T: Serialize>(result: SdkResult<T>) -> *mut c_char {
    let value = match result {
        Ok(body) => match serde_json::to_value(body) {
            Ok(body) => json!({"ok": true, "body": body}),
            Err(error) => {
                logging::write(
                    Level::Error,
                    "sdk.envelope",
                    format!(
                        "operation=serialize_response code=serialization_error error_type={}",
                        std::any::type_name::<T>()
                    ),
                );
                json!({
                    "ok": false,
                    "error": BusinessError::unavailable(error.to_string())
                })
            }
        },
        Err(error) => json!({"ok": false, "error": error}),
    };
    into_c_string(value.to_string())
}

fn c_string(pointer: *const c_char, name: &str) -> SdkResult<String> {
    if pointer.is_null() {
        return Err(BusinessError::invalid(format!("{name} is null")));
    }
    unsafe {
        CStr::from_ptr(pointer)
            .to_str()
            .map(str::to_string)
            .map_err(|error| BusinessError::invalid(format!("{name} is not UTF-8: {error}")))
    }
}

fn optional_c_string(pointer: *const c_char, name: &str) -> SdkResult<Option<String>> {
    if pointer.is_null() {
        return Ok(None);
    }
    c_string(pointer, name).map(Some)
}

fn json_from_c(pointer: *const c_char, name: &str) -> SdkResult<Value> {
    let value = c_string(pointer, name)?;
    serde_json::from_str(&value)
        .map_err(|error| BusinessError::invalid(format!("{name} is invalid: {error}")))
}

fn typed_json_from_c<T: serde::de::DeserializeOwned>(
    pointer: *const c_char,
    name: &str,
) -> SdkResult<T> {
    let value = c_string(pointer, name)?;
    serde_json::from_str(&value)
        .map_err(|error| BusinessError::invalid(format!("{name} is invalid: {error}")))
}

unsafe fn write_error_out(error_out: *mut *mut c_char, value: *mut c_char) {
    if !error_out.is_null() {
        *error_out = value;
    }
}

fn into_c_string(value: String) -> *mut c_char {
    CString::new(value)
        .unwrap_or_else(|_| CString::new("string contained an interior nul byte").unwrap())
        .into_raw()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex as StdMutex, OnceLock};

    unsafe extern "C" fn collect_event(event_json: *const c_char, user_data: *mut c_void) {
        let sender = &*(user_data as *const std::sync::mpsc::Sender<String>);
        let value = CStr::from_ptr(event_json).to_string_lossy().to_string();
        let _ = sender.send(value);
    }

    #[test]
    fn exposes_the_current_abi_version() {
        assert_eq!(suncode_agent_sdk_abi_version(), 10);
    }

    #[test]
    fn exposes_the_agent_core_version() {
        let expected = AgentSdk::version().version;
        let pointer = suncode_agent_sdk_version();
        assert!(!pointer.is_null());
        let response = unsafe { CStr::from_ptr(pointer) }
            .to_str()
            .unwrap()
            .to_owned();
        unsafe { suncode_agent_sdk_string_free(pointer) };
        let response: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(response["body"]["version"], expected);
    }

    #[test]
    fn typed_event_preserves_the_legacy_json_envelope() {
        let (sender, receiver) = std::sync::mpsc::channel::<String>();
        let event = AgentEvent {
            session_id: "session-1".into(),
            occurred_at: "2026-09-19T00:00:00.000Z".into(),
            payload: suncode_sdk::AgentEventPayload::TurnState(
                suncode_sdk::events::TurnStatePayload {
                    turn_id: "turn-1".into(),
                    state: "calling_model".into(),
                    model_id: Some("gpt-5.5".into()),
                    submission_idempotency_key: Some("submission-1".into()),
                    reason: None,
                },
            ),
        };
        emit_agent_event(collect_event, &sender as *const _ as usize, &event);
        let envelope: Value = serde_json::from_str(&receiver.recv().unwrap()).unwrap();
        assert_eq!(envelope["session_id"], "session-1");
        assert_eq!(envelope["event_type"], "turn.state");
        assert_eq!(envelope["payload"]["turn_id"], "turn-1");
    }

    #[test]
    fn native_watch_is_dormant_until_started_and_start_is_single_use() {
        static ENVIRONMENT: OnceLock<StdMutex<()>> = OnceLock::new();
        let _guard = ENVIRONMENT
            .get_or_init(|| StdMutex::new(()))
            .lock()
            .unwrap();
        let directory = tempfile::tempdir().unwrap();
        let data_directory = directory.path().join("data");
        std::env::set_var("SUNCODE_DATA_DIRECTORY", &data_directory);
        std::env::remove_var("SUNCODE_DATABASE_PATH");
        std::env::set_var("SUNCODE_NON_INTERACTIVE", "false");

        let sdk = AgentSdk::open_default("test-native-watch").unwrap();
        let project_root = directory.path().join("project");
        std::fs::create_dir_all(&project_root).unwrap();
        let project = sdk
            .open_project(project_root.to_str().unwrap(), Some("Project"))
            .unwrap();
        let session = sdk
            .create_session(&project.project_id, Some("Session"), None)
            .unwrap();
        let mut handle = SunCodeAgentHandle { sdk };
        let session_id = CString::new(session.session_id).unwrap();
        let (sender, _receiver) = std::sync::mpsc::channel::<String>();
        let mut snapshot = ptr::null_mut();
        let mut error = ptr::null_mut();

        let subscription = unsafe {
            suncode_agent_sdk_watch_session(
                &mut handle,
                session_id.as_ptr(),
                Some(collect_event),
                &sender as *const _ as *mut c_void,
                &mut snapshot,
                &mut error,
            )
        };
        assert!(!subscription.is_null());
        assert!(error.is_null());
        assert!(!snapshot.is_null());
        let snapshot_json = unsafe { CStr::from_ptr(snapshot) }.to_str().unwrap();
        let snapshot_value: Value = serde_json::from_str(snapshot_json).unwrap();
        assert_eq!(
            snapshot_value["session"]["sessionId"],
            session_id.to_str().unwrap()
        );
        unsafe { suncode_agent_sdk_string_free(snapshot) };
        assert!(!unsafe { &*subscription }.started.load(Ordering::Acquire));
        assert!(unsafe { &*subscription }.join.lock().unwrap().is_none());

        assert_eq!(
            unsafe { suncode_agent_sdk_subscription_start(subscription, &mut error) },
            1
        );
        assert!(error.is_null());
        assert!(unsafe { &*subscription }.started.load(Ordering::Acquire));
        assert!(unsafe { &*subscription }.join.lock().unwrap().is_some());

        assert_eq!(
            unsafe { suncode_agent_sdk_subscription_start(subscription, &mut error) },
            0
        );
        assert!(!error.is_null());
        unsafe { suncode_agent_sdk_string_free(error) };
        unsafe { suncode_agent_sdk_subscription_close(subscription) };

        let mut dormant_snapshot = ptr::null_mut();
        let mut dormant_error = ptr::null_mut();
        let dormant = unsafe {
            suncode_agent_sdk_watch_session(
                &mut handle,
                session_id.as_ptr(),
                Some(collect_event),
                &sender as *const _ as *mut c_void,
                &mut dormant_snapshot,
                &mut dormant_error,
            )
        };
        assert!(!dormant.is_null());
        assert!(dormant_error.is_null());
        assert!(!unsafe { &*dormant }.started.load(Ordering::Acquire));
        unsafe { suncode_agent_sdk_string_free(dormant_snapshot) };
        unsafe { suncode_agent_sdk_subscription_close(dormant) };

        std::env::remove_var("SUNCODE_DATA_DIRECTORY");
        std::env::remove_var("SUNCODE_NON_INTERACTIVE");
    }
}
