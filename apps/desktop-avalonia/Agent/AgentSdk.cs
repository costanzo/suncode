using System.Runtime.InteropServices;
using System.Runtime.CompilerServices;
using System.Text.Json;
using System.Text.Json.Nodes;
using SunCode.Desktop.Infrastructure;

namespace SunCode.Desktop.Agent;

public sealed class AgentSdk : IDisposable
{
    private const uint AbiVersion = 4;
    private static readonly object SharedHandleLock = new();
    private static IntPtr _sharedHandle;
    private static int _sharedHandleReferences;
    private IntPtr _handle;
    private bool _disposed;

    private AgentSdk(IntPtr handle) => _handle = handle;

    public static Task<AgentSdk> OpenAsync() => Task.Run(() =>
    {
        try
        {
            lock (SharedHandleLock)
            {
                if (_sharedHandle == IntPtr.Zero)
                {
                    var version = NativeMethods.suncode_agent_sdk_abi_version();
                    if (version != AbiVersion)
                    {
                        DiagnosticLog.Error("sdk.open", $"operation=open abi={version} expected={AbiVersion}");
                        throw new SdkException("abi_mismatch", $"Agent ABI {version} is not supported; expected {AbiVersion}");
                    }

                    _sharedHandle = NativeMethods.suncode_agent_sdk_open_default(out var error);
                    if (_sharedHandle == IntPtr.Zero)
                    {
                        DiagnosticLog.Error("sdk.open", "operation=open native_handle=null");
                        throw new SdkException("agent_unavailable", TakeString(error, true) ?? "SunCode agent could not be started");
                    }
                }

                _sharedHandleReferences++;
                DiagnosticLog.Debug("sdk.open", $"operation=open references={_sharedHandleReferences}");
                return new AgentSdk(_sharedHandle);
            }
        }
        catch (Exception exception)
        {
            DiagnosticLog.Error("sdk.open", exception, "operation=open");
            throw;
        }
    });

    public Task<JsonObject> HealthAsync() => CallAsync(NativeMethods.suncode_agent_sdk_health);
    public Task<JsonObject> DiagnosticsAsync() => CallAsync(NativeMethods.suncode_agent_sdk_diagnostics);
    public Task<JsonObject> ListModelsAsync() => CallAsync(NativeMethods.suncode_agent_sdk_list_models);
    public Task<JsonObject> ListCredentialsAsync() => CallAsync(NativeMethods.suncode_agent_sdk_list_credentials);
    public Task<JsonObject> ListProjectsAsync() => CallAsync(NativeMethods.suncode_agent_sdk_list_projects);

    public Task<JsonObject> ListSettingsAsync() => CallAsync(handle =>
        NativeMethods.suncode_agent_sdk_list_settings(handle, IntPtr.Zero, IntPtr.Zero));

    public Task<JsonObject> ListProjectSettingsAsync(string projectId) => WithUtf8Async(
        [projectId], values =>
            NativeMethods.suncode_agent_sdk_list_settings(_handle, values[0], IntPtr.Zero));

    public Task<JsonObject> ListSessionSettingsAsync(string projectId, string sessionId) => WithUtf8Async(
        [projectId, sessionId], values =>
            NativeMethods.suncode_agent_sdk_list_settings(_handle, values[0], values[1]));

    public Task<JsonObject> SetSettingAsync(string key, object value) => WithUtf8Async(
        ["global", key, $"{{\"value\":{JsonSerializer.Serialize(value)}}}"],
        values => NativeMethods.suncode_agent_sdk_set_setting(
            _handle, values[0], IntPtr.Zero, IntPtr.Zero, values[1], values[2]));

    public Task<JsonObject> SetProjectSettingAsync(string projectId, string key, object value) => WithUtf8Async(
        ["project", projectId, key, $"{{\"value\":{JsonSerializer.Serialize(value)}}}"],
        values => NativeMethods.suncode_agent_sdk_set_setting(
            _handle, values[0], values[1], IntPtr.Zero, values[2], values[3]));

    public Task<JsonObject> SetSessionFullControlAsync(string sessionId, bool enabled) => WithUtf8Async(
        ["session", sessionId, "full_control", $"{{\"value\":{JsonSerializer.Serialize(enabled)}}}"],
        values => NativeMethods.suncode_agent_sdk_set_setting(
            _handle, values[0], IntPtr.Zero, values[1], values[2], values[3]));

    public Task<JsonObject> SetCredentialAsync(string provider, string apiKey) => WithUtf8Async(
        [provider, apiKey], values => NativeMethods.suncode_agent_sdk_set_credential(_handle, values[0], values[1]));

    public Task<JsonObject> RemoveCredentialAsync(string provider) => WithUtf8Async(
        [provider], values => NativeMethods.suncode_agent_sdk_remove_credential(_handle, values[0]));

    public Task<JsonObject> OpenProjectAsync(string path) => WithUtf8Async(
        [path], values => NativeMethods.suncode_agent_sdk_open_project(_handle, values[0], IntPtr.Zero));

    public Task<JsonObject> SelectProjectAsync(string projectId) => WithUtf8Async(
        [projectId], values => NativeMethods.suncode_agent_sdk_select_project(_handle, values[0]));

    public Task<JsonObject> ListProjectDependenciesAsync(string projectId) => WithUtf8Async(
        [projectId], values => NativeMethods.suncode_agent_sdk_list_project_dependencies(_handle, values[0]));

    public Task<JsonObject> AddProjectDependencyAsync(string projectId, string path) => WithUtf8Async(
        [projectId, path], values => NativeMethods.suncode_agent_sdk_add_project_dependency(_handle, values[0], values[1]));

    public Task<JsonObject> RemoveProjectDependencyAsync(string projectId, string dependencyId) => WithUtf8Async(
        [projectId, dependencyId], values => NativeMethods.suncode_agent_sdk_remove_project_dependency(_handle, values[0], values[1]));

    public Task<JsonObject> ListProjectDirectoryAsync(string projectId, string? dependencyId, string path) => WithNullableUtf8Async(
        [projectId, dependencyId, path], values => NativeMethods.suncode_agent_sdk_list_project_directory(_handle, values[0], values[1], values[2]));

    public Task<JsonObject> GitStatusAsync(string projectId) => WithUtf8Async(
        [projectId], values => NativeMethods.suncode_agent_sdk_git_status(_handle, values[0]));

    public Task<JsonObject> GitDiffAsync(string projectId, string scope, string path) => WithUtf8Async(
        [projectId, scope, path], values => NativeMethods.suncode_agent_sdk_git_diff_file(_handle, values[0], values[1], values[2]));

    public Task<JsonObject> ListSessionsAsync(string projectId) => WithUtf8Async(
        [projectId], values => NativeMethods.suncode_agent_sdk_list_sessions(_handle, values[0]));

    public Task<JsonObject> CreateSessionAsync(string projectId, string? title, string? model) => WithNullableUtf8Async(
        [projectId, title, model], values => NativeMethods.suncode_agent_sdk_create_session(_handle, values[0], values[1], values[2]));

    public Task<JsonObject> RenameSessionAsync(string sessionId, string title) => WithUtf8Async(
        [sessionId, title], values => NativeMethods.suncode_agent_sdk_rename_session(_handle, values[0], values[1]));

    public Task<JsonObject> ArchiveSessionAsync(string sessionId) => WithUtf8Async(
        [sessionId], values => NativeMethods.suncode_agent_sdk_archive_session(_handle, values[0]));

    public Task<JsonObject> SetSessionPinnedAsync(string sessionId, bool pinned) => WithUtf8Async(
        [sessionId], values => NativeMethods.suncode_agent_sdk_set_session_pinned(_handle, values[0], pinned ? (byte)1 : (byte)0));

    public Task<JsonObject> SessionSnapshotAsync(string sessionId) => WithUtf8Async(
        [sessionId], values => NativeMethods.suncode_agent_sdk_session_snapshot(_handle, values[0], 0));

    public Task<JsonObject> SessionUsageAsync(string sessionId) => WithUtf8Async(
        [sessionId], values => NativeMethods.suncode_agent_sdk_session_usage(_handle, values[0]));

    public Task<JsonObject> ListProviderExchangesAsync(string sessionId) => WithUtf8Async(
        [sessionId], values => NativeMethods.suncode_agent_sdk_list_provider_exchanges(_handle, values[0]));

    public Task<JsonObject> ProviderExchangeAsync(string sessionId, string exchangeId) => WithUtf8Async(
        [sessionId, exchangeId], values => NativeMethods.suncode_agent_sdk_provider_exchange(_handle, values[0], values[1]));

    public Task<JsonObject> ListCheckpointsAsync(string sessionId) => WithUtf8Async(
        [sessionId], values => NativeMethods.suncode_agent_sdk_list_checkpoints(_handle, values[0]));

    public Task<JsonObject> RestoreCheckpointAsync(string manifestId, string sessionId) => WithUtf8Async(
        [manifestId, sessionId], values => NativeMethods.suncode_agent_sdk_restore_checkpoint(_handle, values[0], values[1]));

    public Task<JsonObject> SubmitTurnAsync(string sessionId, string input, string model, string? reasoningEffort) => WithNullableUtf8Async(
        [sessionId, input, Guid.NewGuid().ToString("N"), model, reasoningEffort],
        values => NativeMethods.suncode_agent_sdk_submit_turn(_handle, values[0], values[1], values[2], values[3], values[4]));

    public Task<JsonObject> CancelTurnAsync(string sessionId, string turnId) => WithUtf8Async(
        [sessionId, turnId], values => NativeMethods.suncode_agent_sdk_cancel_turn(_handle, values[0], values[1]));

    public Task<JsonObject> ResolveApprovalAsync(string approvalId, string decision) => WithUtf8Async(
        [approvalId, decision], values => NativeMethods.suncode_agent_sdk_resolve_approval(_handle, values[0], values[1]));

    public Task<JsonObject> ReplyQuestionAsync(string requestId, JsonArray answers) => WithUtf8Async(
        [requestId, answers.ToJsonString()], values => NativeMethods.suncode_agent_sdk_reply_question(_handle, values[0], values[1]));

    public Task<JsonObject> RejectQuestionAsync(string requestId) => WithUtf8Async(
        [requestId], values => NativeMethods.suncode_agent_sdk_reject_question(_handle, values[0]));

    public IDisposable Subscribe(string sessionId, long after, Action<string> onEvent)
    {
        ThrowIfDisposed();
        DiagnosticLog.Debug("sdk.subscribe", $"begin session={sessionId} after={after}");
        return new Subscription(_handle, sessionId, after, onEvent);
    }

    private Task<JsonObject> CallAsync(Func<IntPtr, IntPtr> call, [CallerMemberName] string operation = "unknown") => Task.Run(() =>
    {
        try
        {
            ThrowIfDisposed();
            return ParseEnvelope(call(_handle));
        }
        catch (Exception exception)
        {
            DiagnosticLog.Error("sdk.call", exception, $"operation={operation}");
            throw;
        }
    });

    private Task<JsonObject> WithUtf8Async(string[] values, Func<IntPtr[], IntPtr> call, [CallerMemberName] string operation = "unknown") =>
        WithNullableUtf8Async(values, call, operation);

    private Task<JsonObject> WithNullableUtf8Async(string?[] values, Func<IntPtr[], IntPtr> call, [CallerMemberName] string operation = "unknown") => Task.Run(() =>
    {
        try
        {
            ThrowIfDisposed();
            var pointers = values.Select(value => value is null ? IntPtr.Zero : Marshal.StringToCoTaskMemUTF8(value)).ToArray();
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
            DiagnosticLog.Error("sdk.call", exception, $"operation={operation}");
            throw;
        }
    });

    private static JsonObject ParseEnvelope(IntPtr response)
    {
        var json = TakeString(response, true) ?? throw new SdkException("invalid_response", "Agent returned no response");
        var envelope = JsonNode.Parse(json) as JsonObject
            ?? throw new SdkException("invalid_response", "Agent returned malformed JSON");
        if (envelope["ok"]?.GetValue<bool>() == true)
        {
            return envelope["body"] as JsonObject ?? [];
        }
        var error = envelope["error"] as JsonObject;
        throw new SdkException(
            error?["code"]?.GetValue<string>() ?? "agent_unavailable",
            error?["message"]?.GetValue<string>() ?? "Agent SDK call failed");
    }

    private static string? TakeString(IntPtr value, bool free)
    {
        if (value == IntPtr.Zero) return null;
        try { return Marshal.PtrToStringUTF8(value); }
        finally { if (free) NativeMethods.suncode_agent_sdk_string_free(value); }
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
                    DiagnosticLog.Info("sdk.close", "native_handle closed");
                    _sharedHandle = IntPtr.Zero;
                }
            }
            _handle = IntPtr.Zero;
        }
    }

    private sealed class Subscription : IDisposable
    {
        private static readonly NativeMethods.EventCallback Callback = Receive;
        private GCHandle _callbackHandle;
        private IntPtr _subscription;

        public Subscription(IntPtr agent, string sessionId, long after, Action<string> onEvent)
        {
            _sessionId = sessionId;
            _callbackHandle = GCHandle.Alloc(onEvent);
            var session = Marshal.StringToCoTaskMemUTF8(sessionId);
            try
            {
                _subscription = NativeMethods.suncode_agent_sdk_subscribe_session(
                    agent, session, after, Callback, GCHandle.ToIntPtr(_callbackHandle), out var error);
                if (_subscription == IntPtr.Zero)
                {
                    var message = TakeString(error, true) ?? "Session events could not be subscribed";
                    DiagnosticLog.Error("sdk.subscribe", $"failed session={sessionId} error={message}");
                    _callbackHandle.Free();
                    throw new SdkException("subscription_failed", message);
                }

                DiagnosticLog.Info("sdk.subscribe", $"ready session={sessionId}");
            }
            finally
            {
                Marshal.FreeCoTaskMem(session);
            }
        }

        private readonly string _sessionId;
        private static void Receive(IntPtr eventJson, IntPtr userData)
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
                    DiagnosticLog.Error("sdk.subscription.callback", exception, "native_callback=true");
                }
            }
        }

        public void Dispose()
        {
            DiagnosticLog.Debug("sdk.subscription", $"dispose begin session={_sessionId} native={_subscription != IntPtr.Zero}");
            if (_subscription != IntPtr.Zero)
            {
                NativeMethods.suncode_agent_sdk_subscription_close(_subscription);
                _subscription = IntPtr.Zero;
            }
            if (_callbackHandle.IsAllocated) _callbackHandle.Free();
            DiagnosticLog.Debug("sdk.subscription", $"dispose end session={_sessionId}");
        }
    }
}
