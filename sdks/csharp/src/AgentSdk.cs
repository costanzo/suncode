using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text.Json;
using SunCode.Sdk.Models;

namespace SunCode.Sdk;

public sealed partial class AgentSdk : IDisposable
{
    private sealed record SettingEnvelope(JsonElement Value);

    private const uint AbiVersion = 15;
    private static readonly object SharedHandleLock = new();
    private static IntPtr _sharedHandle;
    private static int _sharedHandleReferences;
    private static string? _sharedUserId;
    private IntPtr _handle;
    private bool _disposed;

    private AgentSdk(IntPtr handle) => _handle = handle;

    public static Task<AgentSdk> OpenAsync(string userId) => Task.Run(() =>
    {
        try
        {
            lock (SharedHandleLock)
            {
                if (string.IsNullOrWhiteSpace(userId)) throw new ArgumentException("User ID is required", nameof(userId));
                if (_sharedHandle != IntPtr.Zero && !string.Equals(_sharedUserId, userId, StringComparison.Ordinal))
                    throw new SdkException("agent_already_active", "The embedded agent is already open for another user");
                if (_sharedHandle == IntPtr.Zero)
                {
                    var version = NativeMethods.suncode_agent_sdk_abi_version();
                    if (version != AbiVersion)
                    {
                        SdkDiagnosticLog.Error("sdk.open", $"operation=open abi={version} expected={AbiVersion}");
                        throw new SdkException("abi_mismatch", $"Agent ABI {version} is not supported; expected {AbiVersion}");
                    }

                    var userIdMemory = Marshal.StringToCoTaskMemUTF8(userId);
                    IntPtr error;
                    try { _sharedHandle = NativeMethods.suncode_agent_sdk_open_default(userIdMemory, out error); }
                    finally { Marshal.FreeCoTaskMem(userIdMemory); }
                    if (_sharedHandle == IntPtr.Zero)
                    {
                        SdkDiagnosticLog.Error("sdk.open", "operation=open native_handle=null");
                        throw new SdkException("agent_unavailable", TakeString(error, true) ?? "SunCode agent could not be started");
                    }
                }

                _sharedUserId = userId;
                _sharedHandleReferences++;
                SdkDiagnosticLog.Debug("sdk.open", $"operation=open references={_sharedHandleReferences}");
                return new AgentSdk(_sharedHandle);
            }
        }
        catch (Exception exception)
        {
            SdkDiagnosticLog.Error("sdk.open", exception, "operation=open");
            throw;
        }
    });

    private static Task<JsonElement> RawVersionAsync() => Task.Run(() =>
        ParseEnvelope(NativeMethods.suncode_agent_sdk_version()));

    private Task<JsonElement> RawHealthAsync() => CallAsync(NativeMethods.suncode_agent_sdk_health);
    private Task<JsonElement> RawDiagnosticsAsync() => CallAsync(NativeMethods.suncode_agent_sdk_diagnostics);
    private Task<JsonElement> RawListModelsAsync() => CallAsync(NativeMethods.suncode_agent_sdk_list_models);
    private Task<JsonElement> RawListCredentialsAsync() => CallAsync(NativeMethods.suncode_agent_sdk_list_credentials);
    private Task<JsonElement> RawListProjectsAsync() => CallAsync(NativeMethods.suncode_agent_sdk_list_projects);

    private Task<JsonElement> RawListAttentionCandidatesAsync(string? since, nuint limit) =>
        WithNullableUtf8Async(
            [since],
            values => NativeMethods.suncode_agent_sdk_list_attention_candidates(_handle, values[0], limit));

    private Task<JsonElement> RawComputerRuntimeInfoAsync() =>
        CallAsync(NativeMethods.suncode_agent_sdk_computer_runtime_info);

    private Task<JsonElement> RawRequestComputerCapturePermissionAsync() =>
        CallAsync(NativeMethods.suncode_agent_sdk_request_computer_capture_permission);

    private Task<JsonElement> RawRequestComputerInputPermissionAsync() =>
        CallAsync(NativeMethods.suncode_agent_sdk_request_computer_input_permission);

    private Task<JsonElement> RawTakeComputerControlAsync() =>
        CallAsync(NativeMethods.suncode_agent_sdk_take_computer_control);

    private Task<JsonElement> RawReturnComputerControlAsync() =>
        CallAsync(NativeMethods.suncode_agent_sdk_return_computer_control);

    private Task<JsonElement> RawSetComputerUseEnabledAsync(bool enabled) =>
        CallAsync(handle => NativeMethods.suncode_agent_sdk_set_computer_use_enabled(handle, enabled ? (byte)1 : (byte)0));

    private Task<JsonElement> RawEmergencyStopComputerUseAsync() =>
        CallAsync(NativeMethods.suncode_agent_sdk_emergency_stop_computer_use);

    private Task<JsonElement> RawBrowserRuntimeInfoAsync(string? projectId) => WithNullableUtf8Async(
        [projectId], values => NativeMethods.suncode_agent_sdk_browser_runtime_info(_handle, values[0]));

    private Task<JsonElement> RawSetBrowserUseEnabledAsync(bool enabled) =>
        CallAsync(handle => NativeMethods.suncode_agent_sdk_set_browser_use_enabled(handle, enabled ? (byte)1 : (byte)0));

    private Task<JsonElement> RawVerifyBrowserRuntimeAsync(string? projectId) => WithNullableUtf8Async(
        [projectId], values => NativeMethods.suncode_agent_sdk_verify_browser_runtime(_handle, values[0]));

    private Task<JsonElement> RawStartBrowserProjectAsync(string projectId) => WithUtf8Async(
        [projectId], values => NativeMethods.suncode_agent_sdk_start_browser_project(_handle, values[0]));

    private Task<JsonElement> RawTakeBrowserControlAsync(string projectId) => WithUtf8Async(
        [projectId], values => NativeMethods.suncode_agent_sdk_take_browser_control(_handle, values[0]));

    private Task<JsonElement> RawReturnBrowserControlAsync(string projectId) => WithUtf8Async(
        [projectId], values => NativeMethods.suncode_agent_sdk_return_browser_control(_handle, values[0]));

    private Task<JsonElement> RawRestartBrowserRuntimeAsync(string projectId) => WithUtf8Async(
        [projectId], values => NativeMethods.suncode_agent_sdk_restart_browser_runtime(_handle, values[0]));

    private Task<JsonElement> RawStopBrowserRuntimeAsync(string projectId) => WithUtf8Async(
        [projectId], values => NativeMethods.suncode_agent_sdk_stop_browser_runtime(_handle, values[0]));

    private Task<JsonElement> RawClearBrowserProfileAsync(string projectId) => WithUtf8Async(
        [projectId], values => NativeMethods.suncode_agent_sdk_clear_browser_profile(_handle, values[0]));

    private Task<JsonElement> RawListMcpServersAsync(string? projectId) => WithNullableUtf8Async(
        [projectId], values => NativeMethods.suncode_agent_sdk_list_mcp_servers(_handle, values[0]));

    private Task<JsonElement> RawCreateMcpServerAsync(string? projectId, string idempotencyKey, string requestJson) => WithNullableUtf8Async(
        [projectId, idempotencyKey, requestJson],
        values => NativeMethods.suncode_agent_sdk_create_mcp_server(_handle, values[0], values[1], values[2]));

    private Task<JsonElement> RawUpdateMcpServerAsync(string? projectId, string serverId, ulong expectedRevision, string idempotencyKey, string requestJson) => WithNullableUtf8Async(
        [projectId, serverId, idempotencyKey, requestJson],
        values => NativeMethods.suncode_agent_sdk_update_mcp_server(_handle, values[0], values[1], expectedRevision, values[2], values[3]));

    private Task<JsonElement> RawSetMcpServerEnabledAsync(string? projectId, string serverId, ulong expectedRevision, string idempotencyKey, bool enabled) => WithNullableUtf8Async(
        [projectId, serverId, idempotencyKey],
        values => NativeMethods.suncode_agent_sdk_set_mcp_server_enabled(_handle, values[0], values[1], expectedRevision, values[2], enabled ? (byte)1 : (byte)0));

    private Task<JsonElement> RawDeleteMcpServerAsync(string serverId, ulong expectedRevision, string idempotencyKey) => WithUtf8Async(
        [serverId, idempotencyKey],
        values => NativeMethods.suncode_agent_sdk_delete_mcp_server(_handle, values[0], expectedRevision, values[1]));

    private Task<JsonElement> RawRetryMcpServerAsync(string projectId, string serverId) => WithUtf8Async(
        [projectId, serverId],
        values => NativeMethods.suncode_agent_sdk_retry_mcp_server(_handle, values[0], values[1]));

    private Task<JsonElement> RawStartMcpProjectAsync(string projectId) => WithUtf8Async(
        [projectId], values => NativeMethods.suncode_agent_sdk_start_mcp_project(_handle, values[0]));

    private Task<JsonElement> RawMcpLoadProgressAsync(string projectId) => WithUtf8Async(
        [projectId], values => NativeMethods.suncode_agent_sdk_mcp_load_progress(_handle, values[0]));

    private Task<JsonElement> RawListLanguageServersAsync(string? projectId) => WithNullableUtf8Async(
        [projectId], values => NativeMethods.suncode_agent_sdk_list_language_servers(_handle, values[0]));

    private Task<JsonElement> RawCreateLanguageServerAsync(string? projectId, string idempotencyKey, string requestJson) => WithNullableUtf8Async(
        [projectId, idempotencyKey, requestJson],
        values => NativeMethods.suncode_agent_sdk_create_language_server(_handle, values[0], values[1], values[2]));

    private Task<JsonElement> RawUpdateLanguageServerAsync(string? projectId, string languageServerId, ulong expectedRevision, string idempotencyKey, string requestJson) => WithNullableUtf8Async(
        [projectId, languageServerId, idempotencyKey, requestJson],
        values => NativeMethods.suncode_agent_sdk_update_language_server(_handle, values[0], values[1], expectedRevision, values[2], values[3]));

    private Task<JsonElement> RawSetLanguageServerEnabledAsync(string? projectId, string languageServerId, ulong expectedRevision, string idempotencyKey, bool enabled) => WithNullableUtf8Async(
        [projectId, languageServerId, idempotencyKey],
        values => NativeMethods.suncode_agent_sdk_set_language_server_enabled(_handle, values[0], values[1], expectedRevision, values[2], enabled ? (byte)1 : (byte)0));

    private Task<JsonElement> RawDeleteLanguageServerAsync(string languageServerId, ulong expectedRevision, string idempotencyKey) => WithUtf8Async(
        [languageServerId, idempotencyKey],
        values => NativeMethods.suncode_agent_sdk_delete_language_server(_handle, values[0], expectedRevision, values[1]));

    private Task<JsonElement> RawRetryLanguageServerAsync(string projectId, string languageServerId) => WithUtf8Async(
        [projectId, languageServerId],
        values => NativeMethods.suncode_agent_sdk_retry_language_server(_handle, values[0], values[1]));

    private Task<JsonElement> RawStartLanguageServerProjectAsync(string projectId) => WithUtf8Async(
        [projectId], values => NativeMethods.suncode_agent_sdk_start_language_server_project(_handle, values[0]));

    private Task<JsonElement> RawListSettingsAsync(string? projectId, string? sessionId) =>
        projectId is null && sessionId is null
            ? CallAsync(handle => NativeMethods.suncode_agent_sdk_list_settings(handle, IntPtr.Zero, IntPtr.Zero))
            : WithNullableUtf8Async(
                [projectId, sessionId],
                values => NativeMethods.suncode_agent_sdk_list_settings(_handle, values[0], values[1]));

    private Task<JsonElement> RawSetSettingAsync(
        string scope,
        string? projectId,
        string? sessionId,
        string key,
        JsonElement value) => WithNullableUtf8Async(
        [scope, projectId, sessionId, key, JsonSerializer.Serialize(new SettingEnvelope(value), TypedJsonOptions)],
        values => NativeMethods.suncode_agent_sdk_set_setting(
            _handle, values[0], values[1], values[2], values[3], values[4]));

    private Task<JsonElement> RawSetProxyConfigurationAsync(string requestJson) => WithUtf8Async(
        [requestJson],
        values => NativeMethods.suncode_agent_sdk_set_proxy_configuration(_handle, values[0]));

    private Task<JsonElement> RawSetCredentialAsync(string provider, string apiKey) => WithUtf8Async(
        [provider, apiKey],
        values => NativeMethods.suncode_agent_sdk_set_credential(_handle, values[0], values[1]));

    private Task<JsonElement> RawRemoveCredentialAsync(string provider) => WithUtf8Async(
        [provider], values => NativeMethods.suncode_agent_sdk_remove_credential(_handle, values[0]));

    private Task<JsonElement> RawSetProviderEndpointAsync(string provider, string endpoint) => WithUtf8Async(
        [provider, endpoint],
        values => NativeMethods.suncode_agent_sdk_set_provider_endpoint(_handle, values[0], values[1]));

    private Task<JsonElement> RawOpenProjectAsync(string path, string? displayName) => WithNullableUtf8Async(
        [path, displayName],
        values => NativeMethods.suncode_agent_sdk_open_project(_handle, values[0], values[1]));

    private Task<JsonElement> RawSelectProjectAsync(string projectId) => WithUtf8Async(
        [projectId], values => NativeMethods.suncode_agent_sdk_select_project(_handle, values[0]));

    private Task<JsonElement> RawListProjectDependenciesAsync(string projectId) => WithUtf8Async(
        [projectId], values => NativeMethods.suncode_agent_sdk_list_project_dependencies(_handle, values[0]));

    private Task<JsonElement> RawAddProjectDependencyAsync(string projectId, string path) => WithUtf8Async(
        [projectId, path],
        values => NativeMethods.suncode_agent_sdk_add_project_dependency(_handle, values[0], values[1]));

    private Task<JsonElement> RawRemoveProjectDependencyAsync(string projectId, string dependencyId) => WithUtf8Async(
        [projectId, dependencyId],
        values => NativeMethods.suncode_agent_sdk_remove_project_dependency(_handle, values[0], values[1]));

    private Task<JsonElement> RawListProjectDirectoryAsync(string projectId, string? dependencyId, string path) => WithNullableUtf8Async(
        [projectId, dependencyId, path],
        values => NativeMethods.suncode_agent_sdk_list_project_directory(_handle, values[0], values[1], values[2]));

    private Task<JsonElement> RawReadProjectFileAsync(string projectId, string? dependencyId, string path) => WithNullableUtf8Async(
        [projectId, dependencyId, path],
        values => NativeMethods.suncode_agent_sdk_read_project_file(_handle, values[0], values[1], values[2]));

    private Task<JsonElement> RawGitStatusAsync(string projectId) => WithUtf8Async(
        [projectId], values => NativeMethods.suncode_agent_sdk_git_status(_handle, values[0]));

    private Task<JsonElement> RawGitDiffAsync(string projectId, string scope, string path) => WithUtf8Async(
        [projectId, scope, path],
        values => NativeMethods.suncode_agent_sdk_git_diff_file(_handle, values[0], values[1], values[2]));

    private Task<JsonElement> RawListSessionsAsync(string projectId) => WithUtf8Async(
        [projectId], values => NativeMethods.suncode_agent_sdk_list_sessions(_handle, values[0]));

    private Task<JsonElement> RawListAgentsAsync() => CallAsync(
        handle => NativeMethods.suncode_agent_sdk_list_agents(handle));

    private Task<JsonElement> RawListChildSessionsAsync(string parentSessionId) => WithUtf8Async(
        [parentSessionId], values => NativeMethods.suncode_agent_sdk_list_child_sessions(_handle, values[0]));

    private Task<JsonElement> RawCreateSessionAsync(string projectId, string? title, string? model) => WithNullableUtf8Async(
        [projectId, title, model],
        values => NativeMethods.suncode_agent_sdk_create_session(_handle, values[0], values[1], values[2]));

    private Task<JsonElement> RawRenameSessionAsync(string sessionId, string title) => WithUtf8Async(
        [sessionId, title],
        values => NativeMethods.suncode_agent_sdk_rename_session(_handle, values[0], values[1]));

    private Task<JsonElement> RawArchiveSessionAsync(string sessionId) => WithUtf8Async(
        [sessionId], values => NativeMethods.suncode_agent_sdk_archive_session(_handle, values[0]));

    private Task<JsonElement> RawSetSessionPinnedAsync(string sessionId, bool pinned) => WithUtf8Async(
        [sessionId],
        values => NativeMethods.suncode_agent_sdk_set_session_pinned(_handle, values[0], pinned ? (byte)1 : (byte)0));

    private Task<JsonElement> RawReopenSessionAsync(string sessionId) => WithUtf8Async(
        [sessionId], values => NativeMethods.suncode_agent_sdk_reopen_session(_handle, values[0]));

    private Task<JsonElement> RawDeleteSessionAsync(string sessionId) => WithUtf8Async(
        [sessionId], values => NativeMethods.suncode_agent_sdk_delete_session(_handle, values[0]));

    private Task<JsonElement> RawListSessionImagesAsync(string sessionId) => WithUtf8Async(
        [sessionId], values => NativeMethods.suncode_agent_sdk_list_session_images(_handle, values[0]));

    private Task<JsonElement> RawAddSessionImageAsync(string sessionId, string imageJson) => WithUtf8Async(
        [sessionId, imageJson],
        values => NativeMethods.suncode_agent_sdk_add_session_image(_handle, values[0], values[1]));

    private Task<JsonElement> RawRemoveSessionImageAsync(string sessionId, string imageId) => WithUtf8Async(
        [sessionId, imageId],
        values => NativeMethods.suncode_agent_sdk_remove_session_image(_handle, values[0], values[1]));

    private Task<JsonElement> RawSessionSnapshotAsync(string sessionId) => WithUtf8Async(
        [sessionId],
        values => NativeMethods.suncode_agent_sdk_session_snapshot(_handle, values[0], 0));

    private Task<JsonElement> RawSessionUsageAsync(string sessionId) => WithUtf8Async(
        [sessionId], values => NativeMethods.suncode_agent_sdk_session_usage(_handle, values[0]));

    private Task<JsonElement> RawListProviderExchangesAsync(string sessionId) => WithUtf8Async(
        [sessionId], values => NativeMethods.suncode_agent_sdk_list_provider_exchanges(_handle, values[0]));

    private Task<JsonElement> RawProviderExchangeAsync(string sessionId, string exchangeId) => WithUtf8Async(
        [sessionId, exchangeId],
        values => NativeMethods.suncode_agent_sdk_provider_exchange(_handle, values[0], values[1]));

    private Task<JsonElement> RawListCheckpointsAsync(string sessionId) => WithUtf8Async(
        [sessionId], values => NativeMethods.suncode_agent_sdk_list_checkpoints(_handle, values[0]));

    private Task<JsonElement> RawCheckpointManifestAsync(string manifestId) => WithUtf8Async(
        [manifestId],
        values => NativeMethods.suncode_agent_sdk_checkpoint_manifest(_handle, values[0]));

    private Task<JsonElement> RawRestoreCheckpointAsync(string manifestId, string sessionId) => WithUtf8Async(
        [manifestId, sessionId],
        values => NativeMethods.suncode_agent_sdk_restore_checkpoint(_handle, values[0], values[1]));

    private Task<JsonElement> RawSubmitTurnAsync(SubmitTurnRequest request) => WithNullableUtf8Async(
        [request.SessionId, request.Input, request.IdempotencyKey, request.Model, request.ReasoningEffort],
        values => NativeMethods.suncode_agent_sdk_submit_turn(_handle, values[0], values[1], values[2], values[3], values[4]));

    private Task<JsonElement> RawSubmitTurnWithAttachmentsAsync(SubmitTurnRequest request) => WithNullableUtf8Async(
        [
            request.SessionId,
            request.Input,
            request.IdempotencyKey,
            request.Model,
            request.ReasoningEffort,
            JsonSerializer.Serialize(request.ImageIds, TypedJsonOptions)
        ],
        values => NativeMethods.suncode_agent_sdk_submit_turn_with_attachments(
            _handle, values[0], values[1], values[2], values[3], values[4], values[5]));

    private Task<JsonElement> RawCancelTurnAsync(string sessionId, string turnId) => WithUtf8Async(
        [sessionId, turnId],
        values => NativeMethods.suncode_agent_sdk_cancel_turn(_handle, values[0], values[1]));

    private Task<JsonElement> RawRetryLastTurnAsync(string sessionId) => WithUtf8Async(
        [sessionId], values => NativeMethods.suncode_agent_sdk_retry_last_turn(_handle, values[0]));

    private Task<JsonElement> RawResolveApprovalAsync(string approvalId, string decision) => WithUtf8Async(
        [approvalId, decision],
        values => NativeMethods.suncode_agent_sdk_resolve_approval(_handle, values[0], values[1]));

    private Task<JsonElement> RawGetApprovalAsync(string approvalId) => WithUtf8Async(
        [approvalId],
        values => NativeMethods.suncode_agent_sdk_get_approval(_handle, values[0]));

    private Task<JsonElement> RawReplyQuestionAsync(string requestId, string answersJson) => WithUtf8Async(
        [requestId, answersJson],
        values => NativeMethods.suncode_agent_sdk_reply_question(_handle, values[0], values[1]));

    private Task<JsonElement> RawRejectQuestionAsync(string requestId) => WithUtf8Async(
        [requestId],
        values => NativeMethods.suncode_agent_sdk_reject_question(_handle, values[0]));

    private IDisposable RawSubscribe(string sessionId, long after, Action<string> onEvent)
    {
        ThrowIfDisposed();
        SdkDiagnosticLog.Debug("sdk.subscribe", $"begin session={sessionId} after={after}");
        return new Subscription(_handle, sessionId, after, onEvent);
    }

    private Task<RawSessionWatch> RawWatchSessionAsync(string sessionId, Action<string> onEvent) => Task.Run(() =>
    {
        ThrowIfDisposed();
        SdkDiagnosticLog.Debug("sdk.watch", $"begin session={sessionId}");
        return new RawSessionWatch(_handle, sessionId, onEvent);
    });

    private Task<JsonElement> CallAsync(
        Func<IntPtr, IntPtr> call,
        [CallerMemberName] string operation = "unknown") => Task.Run(() =>
    {
        try
        {
            ThrowIfDisposed();
            return ParseEnvelope(call(_handle));
        }
        catch (Exception exception)
        {
            SdkDiagnosticLog.Error("sdk.call", exception, $"operation={operation}");
            throw;
        }
    });

    private Task<JsonElement> WithUtf8Async(
        string[] values,
        Func<IntPtr[], IntPtr> call,
        [CallerMemberName] string operation = "unknown") =>
        WithNullableUtf8Async(values, call, operation);

    private Task<JsonElement> WithNullableUtf8Async(
        string?[] values,
        Func<IntPtr[], IntPtr> call,
        [CallerMemberName] string operation = "unknown") => Task.Run(() =>
    {
        try
        {
            ThrowIfDisposed();
            var pointers = values
                .Select(value => value is null ? IntPtr.Zero : Marshal.StringToCoTaskMemUTF8(value))
                .ToArray();
            try
            {
                return ParseEnvelope(call(pointers));
            }
            finally
            {
                foreach (var pointer in pointers)
                {
                    if (pointer != IntPtr.Zero) Marshal.FreeCoTaskMem(pointer);
                }
            }
        }
        catch (Exception exception)
        {
            SdkDiagnosticLog.Error("sdk.call", exception, $"operation={operation}");
            throw;
        }
    });

    private static JsonElement ParseEnvelope(IntPtr response)
    {
        var json = TakeString(response, true) ?? throw new SdkException("invalid_response", "Agent returned no response");
        using var document = JsonDocument.Parse(json);
        var envelope = document.RootElement;
        if (envelope.TryGetProperty("ok", out var ok) && ok.ValueKind == JsonValueKind.True)
            return envelope.GetProperty("body").Clone();

        var code = "agent_unavailable";
        var message = "Agent SDK call failed";
        if (envelope.TryGetProperty("error", out var error) && error.ValueKind == JsonValueKind.Object)
        {
            if (error.TryGetProperty("code", out var errorCode) && errorCode.ValueKind == JsonValueKind.String)
                code = errorCode.GetString() ?? code;
            if (error.TryGetProperty("message", out var errorMessage) && errorMessage.ValueKind == JsonValueKind.String)
                message = errorMessage.GetString() ?? message;
        }
        throw new SdkException(code, message);
    }

    private static string? TakeString(IntPtr value, bool free)
    {
        if (value == IntPtr.Zero) return null;
        try { return Marshal.PtrToStringUTF8(value); }
        finally
        {
            if (free) NativeMethods.suncode_agent_sdk_string_free(value);
        }
    }

    private void ThrowIfDisposed()
    {
        if (_disposed || _handle == IntPtr.Zero) throw new ObjectDisposedException(nameof(AgentSdk));
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;
        lock (SharedHandleLock)
        {
            if (_handle != IntPtr.Zero && _handle == _sharedHandle)
            {
                _sharedHandleReferences--;
                if (_sharedHandleReferences == 0)
                {
                    NativeMethods.suncode_agent_sdk_close(_sharedHandle);
                    SdkDiagnosticLog.Info("sdk.close", "native_handle closed");
                    _sharedHandle = IntPtr.Zero;
                    _sharedUserId = null;
                }
            }
            _handle = IntPtr.Zero;
        }
    }

    private static readonly NativeMethods.EventCallback SubscriptionCallback = ReceiveSubscriptionEvent;

    private static void ReceiveSubscriptionEvent(IntPtr eventJson, IntPtr userData)
    {
        if (eventJson == IntPtr.Zero || userData == IntPtr.Zero) return;
        var json = Marshal.PtrToStringUTF8(eventJson);
        if (json is null) return;
        var handle = GCHandle.FromIntPtr(userData);
        if (handle.Target is Action<string> callback)
        {
            try
            {
                callback(json);
            }
            catch (Exception exception)
            {
                SdkDiagnosticLog.Error("sdk.subscription.callback", exception, "native_callback=true");
            }
        }
    }

    private sealed class Subscription : IDisposable
    {
        private GCHandle _callbackHandle;
        private IntPtr _subscription;
        private readonly string _sessionId;

        public Subscription(IntPtr agent, string sessionId, long after, Action<string> onEvent)
        {
            _sessionId = sessionId;
            _callbackHandle = GCHandle.Alloc(onEvent);
            var session = Marshal.StringToCoTaskMemUTF8(sessionId);
            try
            {
                _subscription = NativeMethods.suncode_agent_sdk_subscribe_session(
                    agent,
                    session,
                    after,
                    SubscriptionCallback,
                    GCHandle.ToIntPtr(_callbackHandle),
                    out var error);
                if (_subscription == IntPtr.Zero)
                {
                    var message = TakeString(error, true) ?? "Session events could not be subscribed";
                    SdkDiagnosticLog.Error("sdk.subscribe", $"failed session={sessionId} error={message}");
                    _callbackHandle.Free();
                    throw new SdkException("subscription_failed", message);
                }

                SdkDiagnosticLog.Info("sdk.subscribe", $"ready session={sessionId}");
            }
            finally
            {
                Marshal.FreeCoTaskMem(session);
            }
        }

        public void Dispose()
        {
            SdkDiagnosticLog.Debug("sdk.subscription", $"dispose begin session={_sessionId} native={_subscription != IntPtr.Zero}");
            if (_subscription != IntPtr.Zero)
            {
                NativeMethods.suncode_agent_sdk_subscription_close(_subscription);
                _subscription = IntPtr.Zero;
            }
            if (_callbackHandle.IsAllocated) _callbackHandle.Free();
            SdkDiagnosticLog.Debug("sdk.subscription", $"dispose end session={_sessionId}");
        }
    }

    private sealed class AttentionSubscription : IDisposable
    {
        private GCHandle _callbackHandle;
        private IntPtr _subscription;

        public AttentionSubscription(IntPtr agent, Action<string> onEvent)
        {
            _callbackHandle = GCHandle.Alloc(onEvent);
            _subscription = NativeMethods.suncode_agent_sdk_subscribe_attention(
                agent,
                SubscriptionCallback,
                GCHandle.ToIntPtr(_callbackHandle),
                out var error);
            if (_subscription != IntPtr.Zero) return;
            var message = TakeString(error, true) ?? "Attention events could not be subscribed";
            _callbackHandle.Free();
            throw new SdkException("subscription_failed", message);
        }

        public void Dispose()
        {
            if (_subscription != IntPtr.Zero)
            {
                NativeMethods.suncode_agent_sdk_attention_subscription_close(_subscription);
                _subscription = IntPtr.Zero;
            }
            if (_callbackHandle.IsAllocated) _callbackHandle.Free();
        }
    }

    private sealed class RawSessionWatch : IDisposable
    {
        private GCHandle _callbackHandle;
        private IntPtr _subscription;
        private readonly string _sessionId;
        private bool _disposed;

        public RawSessionWatch(IntPtr agent, string sessionId, Action<string> onEvent)
        {
            _sessionId = sessionId;
            _callbackHandle = GCHandle.Alloc(onEvent);
            var session = Marshal.StringToCoTaskMemUTF8(sessionId);
            try
            {
                _subscription = NativeMethods.suncode_agent_sdk_watch_session(
                    agent,
                    session,
                    SubscriptionCallback,
                    GCHandle.ToIntPtr(_callbackHandle),
                    out var snapshot,
                    out var error);
                if (_subscription == IntPtr.Zero)
                {
                    var message = TakeString(error, true) ?? "Session watch could not be created";
                    _callbackHandle.Free();
                    throw new SdkException("subscription_failed", message);
                }

                var snapshotJson = TakeString(snapshot, true)
                    ?? throw new SdkException("invalid_response", "Session watch returned no snapshot");
                using var document = JsonDocument.Parse(snapshotJson);
                Snapshot = document.RootElement.Clone();
                SdkDiagnosticLog.Info("sdk.watch", $"ready session={sessionId} state=dormant");
            }
            catch
            {
                Dispose();
                throw;
            }
            finally
            {
                Marshal.FreeCoTaskMem(session);
            }
        }

        public JsonElement Snapshot { get; }

        public void Start()
        {
            if (_disposed || _subscription == IntPtr.Zero) throw new ObjectDisposedException(nameof(RawSessionWatch));
            if (NativeMethods.suncode_agent_sdk_subscription_start(_subscription, out var error) == 0)
            {
                var message = TakeString(error, true) ?? "Session event delivery could not be started";
                throw new SdkException("subscription_failed", message);
            }
            SdkDiagnosticLog.Info("sdk.watch", $"started session={_sessionId}");
        }

        public void Dispose()
        {
            if (_disposed) return;
            _disposed = true;
            SdkDiagnosticLog.Debug("sdk.watch", $"dispose begin session={_sessionId} native={_subscription != IntPtr.Zero}");
            if (_subscription != IntPtr.Zero)
            {
                NativeMethods.suncode_agent_sdk_subscription_close(_subscription);
                _subscription = IntPtr.Zero;
            }
            if (_callbackHandle.IsAllocated) _callbackHandle.Free();
            SdkDiagnosticLog.Debug("sdk.watch", $"dispose end session={_sessionId}");
        }
    }
}
