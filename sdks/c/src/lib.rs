use serde::Serialize;
use serde_json::{json, Value};
use std::{
    ffi::{CStr, CString},
    os::raw::{c_char, c_void},
    panic::{catch_unwind, AssertUnwindSafe},
    ptr,
};
use suncode_common::BusinessError;
use suncode_sdk::logging_module::{self as logging, Level};
use suncode_sdk::{
    AgentSdk, AgentSubscription, McpServerWriteRequest, SdkResult, SunCodeEventCallback,
    SUNCODE_AGENT_SDK_ABI_VERSION,
};

pub struct SunCodeAgentHandle {
    sdk: AgentSdk,
}

pub struct SunCodeAgentSubscriptionHandle {
    _subscription: AgentSubscription,
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
    error_out: *mut *mut c_char,
) -> *mut SunCodeAgentHandle {
    write_error_out(error_out, ptr::null_mut());
    match catch_unwind(AssertUnwindSafe(AgentSdk::open_default)) {
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
    after: i64,
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
        handle.sdk.subscribe_session_events(
            c_string(session_id, "session_id")?,
            after,
            callback,
            user_data,
        )
    }));
    match result {
        Ok(Ok(subscription)) => Box::into_raw(Box::new(SunCodeAgentSubscriptionHandle {
            _subscription: subscription,
        })),
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

    #[test]
    fn exposes_the_current_abi_version() {
        assert_eq!(suncode_agent_sdk_abi_version(), 6);
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
}
