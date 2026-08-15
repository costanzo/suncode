#pragma once

extern "C" {
enum { SUNCODE_RUNTIME_SDK_ABI_VERSION = 1 };

struct SunCodeRuntimeHandle;
struct SunCodeRuntimeSubscriptionHandle;

unsigned int suncode_runtime_sdk_abi_version();
SunCodeRuntimeHandle *suncode_runtime_sdk_open_default(char **error_out);
void suncode_runtime_sdk_close(SunCodeRuntimeHandle *handle);
char *suncode_runtime_sdk_health(SunCodeRuntimeHandle *handle);
char *suncode_runtime_sdk_diagnostics(SunCodeRuntimeHandle *handle);
char *suncode_runtime_sdk_list_models(SunCodeRuntimeHandle *handle);
char *suncode_runtime_sdk_list_settings(SunCodeRuntimeHandle *handle, const char *project_id, const char *session_id);
char *suncode_runtime_sdk_set_setting(SunCodeRuntimeHandle *handle, const char *scope, const char *project_id, const char *session_id, const char *key, const char *value_json);
char *suncode_runtime_sdk_list_credentials(SunCodeRuntimeHandle *handle);
char *suncode_runtime_sdk_set_credential(SunCodeRuntimeHandle *handle, const char *provider, const char *api_key);
char *suncode_runtime_sdk_remove_credential(SunCodeRuntimeHandle *handle, const char *provider);
char *suncode_runtime_sdk_list_projects(SunCodeRuntimeHandle *handle);
char *suncode_runtime_sdk_open_project(SunCodeRuntimeHandle *handle, const char *path, const char *display_name);
char *suncode_runtime_sdk_select_project(SunCodeRuntimeHandle *handle, const char *project_id);
char *suncode_runtime_sdk_git_status(SunCodeRuntimeHandle *handle, const char *project_id);
char *suncode_runtime_sdk_git_diff_file(SunCodeRuntimeHandle *handle, const char *project_id, const char *scope, const char *path);
char *suncode_runtime_sdk_list_sessions(SunCodeRuntimeHandle *handle, const char *project_id);
char *suncode_runtime_sdk_create_session(SunCodeRuntimeHandle *handle, const char *project_id, const char *title, const char *model);
char *suncode_runtime_sdk_rename_session(SunCodeRuntimeHandle *handle, const char *session_id, const char *title);
char *suncode_runtime_sdk_archive_session(SunCodeRuntimeHandle *handle, const char *session_id);
char *suncode_runtime_sdk_reopen_session(SunCodeRuntimeHandle *handle, const char *session_id);
char *suncode_runtime_sdk_session_snapshot(SunCodeRuntimeHandle *handle, const char *session_id, long long after);
char *suncode_runtime_sdk_session_usage(SunCodeRuntimeHandle *handle, const char *session_id);
char *suncode_runtime_sdk_list_checkpoints(SunCodeRuntimeHandle *handle, const char *session_id);
char *suncode_runtime_sdk_checkpoint_manifest(SunCodeRuntimeHandle *handle, const char *manifest_id);
char *suncode_runtime_sdk_restore_checkpoint(SunCodeRuntimeHandle *handle, const char *manifest_id, const char *session_id);
char *suncode_runtime_sdk_submit_turn(SunCodeRuntimeHandle *handle, const char *session_id, const char *input, const char *idempotency_key, const char *model);
char *suncode_runtime_sdk_cancel_turn(SunCodeRuntimeHandle *handle, const char *session_id, const char *turn_id);
char *suncode_runtime_sdk_get_approval(SunCodeRuntimeHandle *handle, const char *approval_id);
char *suncode_runtime_sdk_resolve_approval(SunCodeRuntimeHandle *handle, const char *approval_id, const char *decision);
SunCodeRuntimeSubscriptionHandle *suncode_runtime_sdk_subscribe_session(
    SunCodeRuntimeHandle *handle,
    const char *session_id,
    long long after,
    void (*callback)(const char *, void *),
    void *user_data,
    char **error_out);
void suncode_runtime_sdk_subscription_close(SunCodeRuntimeSubscriptionHandle *subscription);
void suncode_runtime_sdk_string_free(char *value);
}
