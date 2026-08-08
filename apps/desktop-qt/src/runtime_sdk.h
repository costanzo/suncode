#pragma once

extern "C" {
struct SuncodeRuntimeHandle;
struct SuncodeRuntimeSubscriptionHandle;

SuncodeRuntimeHandle *suncode_runtime_sdk_open_default(char **error_out);
void suncode_runtime_sdk_close(SuncodeRuntimeHandle *handle);
char *suncode_runtime_sdk_request_json(SuncodeRuntimeHandle *handle, const char *method, const char *path, const char *body_json);
SuncodeRuntimeSubscriptionHandle *suncode_runtime_sdk_subscribe_session(
    SuncodeRuntimeHandle *handle,
    const char *session_id,
    long long after,
    void (*callback)(const char *, void *),
    void *user_data,
    char **error_out);
void suncode_runtime_sdk_subscription_close(SuncodeRuntimeSubscriptionHandle *subscription);
void suncode_runtime_sdk_string_free(char *value);
}
