using System.Collections.ObjectModel;
using System.Diagnostics;
using System.Globalization;
using System.Text.Json;
using System.Text.Json.Nodes;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Threading;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Desktop.Agent;

namespace SunCode.Desktop.ViewModels;

public sealed partial class DesktopViewModel : ObservableObject, IDisposable
{
    private async Task RunAsync(Func<Task> operation, string? success = null, [System.Runtime.CompilerServices.CallerMemberName] string operationName = "unknown")
    {
        IsBusy = true;
        try
        {
            await operation();
            if (success is not null) StatusText = success;
            ConnectionState = "connected";
        }
        catch (Exception exception)
        {
            DiagnosticLog.Error("viewmodel.operation", exception, $"operation={operationName}");
            ReportError(exception);
        }
        finally
        {
            IsBusy = false;
        }
    }

    private bool EnsureSdk()
    {
        if (_sdk is not null && !_disposed) return true;
        StatusText = "Agent SDK is not connected";
        ConnectionState = "error";
        return false;
    }

    private async Task InitializeCoreAsync()
    {
        ConnectionState = "connecting";
        StatusText = "Starting local agent...";
        try
        {
            var sdk = await AgentSdk.OpenAsync();
            await sdk.HealthAsync();
            _sdk = sdk;
            ConnectionState = "connected";
            StatusText = "Connected to local agent";
            await LoadModelsAsync();
            await LoadSettingsAsync();
            await LoadCredentialsAsync();
            await LoadProjectsAsync();
            await RefreshDiagnosticsAsync();
        }
        catch (Exception exception)
        {
            ReportError(exception);
        }
        finally
        {
            lock (_initializationGate)
            {
                _initializationTask = null;
            }
        }
    }

    private async Task<bool> EnsureSdkReadyAsync()
    {
        if (_disposed) return false;
        if (_sdk is null) await InitializeAsync();
        return _sdk is not null && !_disposed;
    }

    private void ReportError(Exception exception)
    {
        DiagnosticLog.Error("viewmodel", exception, $"session={SelectedSession?.SessionId ?? "none"}");
        ConnectionState = "error";
        StatusText = exception.Message;
    }

    private void SetTheme(string mode)
    {
        ThemeMode = mode;
        ThemeChanged?.Invoke(mode);
    }

    private void CloseSubscription()
    {
        var hadSubscription = _subscription is not null;
        if (hadSubscription) DiagnosticLog.Debug("session", "close_subscription.dispose.begin");
        _subscription?.Dispose();
        _subscription = null;
        if (hadSubscription) DiagnosticLog.Debug("session", "close_subscription.dispose.end");
    }

    private void ClearSession(bool clearSelection = true)
    {
        Interlocked.Increment(ref _sessionLoadVersion);
        _loadedSessionId = null;
        IsSessionLoading = false;
        SessionLoadError = string.Empty;
        DisposeMessages();
        Messages = [];
        _appliedMessageIds.Clear();
        Activities.Clear();
        ChangedPaths.Clear();
        Checkpoints.Clear();
        DiffLines.Clear();
        DisposeSubmittedAttachments();
        ReplaceComposerAttachments([]);
        ClearProviderTraces();
        PendingApproval = null;
        FullControlEnabled = false;
        ActiveTurnId = string.Empty;
        SessionTotalTokens = 0;
        if (clearSelection) SelectedSession = null;
        OnPropertyChanged(nameof(HasActivities));
        OnPropertyChanged(nameof(HasCheckpoints));
    }

    private bool IsCurrentSessionLoad(string sessionId, long loadVersion) =>
        !_disposed
        && _sessionLoadVersion == loadVersion
        && SelectedSession?.SessionId == sessionId;

    private async Task RevealSessionLoadingAsync(string sessionId, long loadVersion)
    {
        await Task.Delay(120);
        if (IsSessionLoading && IsCurrentSessionLoad(sessionId, loadVersion))
        {
            IsSessionLoadingVisible = true;
            LogSession("loading", sessionId, $"visible=true version={loadVersion}");
        }
    }

    private bool IsSessionContextCurrent(string sessionId, long? loadVersion) =>
        !_disposed
        && SelectedSession?.SessionId == sessionId
        && (loadVersion is null || _sessionLoadVersion == loadVersion.Value);

    private string DescribeSessionContext() =>
        $"selected={SelectedSession?.SessionId ?? "<none>"},loaded={_loadedSessionId ?? "<none>"},version={_sessionLoadVersion},loading={IsSessionLoading}";

    private static void LogSession(string operationId, string sessionId, string message) =>
        DiagnosticLog.Write(SessionLogLevel(operationId, message), "session", $"op={operationId} session={sessionId} {message}");

    private static DiagnosticLogLevel SessionLogLevel(string operationId, string message)
    {
        if (message.Contains("failed", StringComparison.OrdinalIgnoreCase)) return DiagnosticLogLevel.Error;
        if (message.Contains("discard", StringComparison.OrdinalIgnoreCase)
            || message.Contains("ignored", StringComparison.OrdinalIgnoreCase)
            || message.Contains("stale", StringComparison.OrdinalIgnoreCase)
            || message.Contains("resync", StringComparison.OrdinalIgnoreCase)) return DiagnosticLogLevel.Warn;
        if (operationId == "event") return DiagnosticLogLevel.Trace;
        if (message.Contains(".begin", StringComparison.Ordinal)
            || message.Contains(".end", StringComparison.Ordinal)
            || message.Contains(".completed", StringComparison.Ordinal)
            || message.Contains(".selected", StringComparison.Ordinal)
            || message.Contains("visible=", StringComparison.Ordinal)) return DiagnosticLogLevel.Debug;
        return DiagnosticLogLevel.Info;
    }

    private void ClearGit()
    {
        GitFiles.Clear();
        FilteredGitFiles.Clear();
        DiffLines.Clear();
        SelectedGitFile = null;
        GitState = "idle";
        GitError = string.Empty;
        GitDiffState = "idle";
        GitDiffError = string.Empty;
        GitPatch = string.Empty;
        GitBranch = string.Empty;
        GitChangedFiles = 0;
        GitAdditions = 0;
        GitDeletions = 0;
        GitStatusTruncated = false;
        GitDiffBinary = false;
        GitDiffTruncated = false;
        GitDiffAdditions = 0;
        GitDiffDeletions = 0;
    }

    private void ClearProviderTraces()
    {
        ProviderTraces.Clear();
        ProviderTraceTurns.Clear();
        FilteredProviderTraceTurns.Clear();
        SelectedProviderTrace = null;
        SelectedProviderTraceDetails = null;
        SelectedProviderTraceContent = null;
        _providerTraceDetails.Clear();
        _providerTraceDetailLoads.Clear();
        ProviderTraceState = "idle";
        ProviderTraceError = string.Empty;
        ProviderTraceFilter = string.Empty;
        OnPropertyChanged(nameof(HasProviderTraces));
        OnPropertyChanged(nameof(HasFilteredProviderTraces));
        OnPropertyChanged(nameof(ProviderTraceCountText));
        OnPropertyChanged(nameof(ProviderTraceSummary));
        OnPropertyChanged(nameof(ProviderTraceEmptyMessage));
    }

    private void ApplyGitFilter()
    {
        var selectedPath = SelectedGitFile?.Path;
        FilteredGitFiles.Clear();
        foreach (var file in GitFiles.Where(file =>
                     (GitScope == "all" || GitScope == "staged" && file.Staged || GitScope == "unstaged" && file.Unstaged) &&
                     (GitFilter.Length == 0 || file.Path.Contains(GitFilter, StringComparison.OrdinalIgnoreCase))))
        {
            FilteredGitFiles.Add(file);
        }
        SelectedGitFile = FilteredGitFiles.FirstOrDefault(file => file.Path == selectedPath)
            ?? FilteredGitFiles.FirstOrDefault();
        OnPropertyChanged(nameof(HasFilteredGitFiles));
        OnPropertyChanged(nameof(GitFileCountText));
        OnPropertyChanged(nameof(GitEmptyMessage));
    }

    private void ApplyProviderTraceFilter()
    {
        var selectedId = SelectedProviderTrace?.ExchangeId;
        FilteredProviderTraceTurns.Clear();
        foreach (var turn in ProviderTraceTurns)
        {
            var turnMatches = ProviderTraceTurnMatches(turn, ProviderTraceFilter);
            var calls = turnMatches
                ? turn.Calls
                : turn.Calls.Where(trace => ProviderTraceMatches(trace, ProviderTraceFilter)).ToList();
            if (turnMatches || calls.Count > 0)
            {
                FilteredProviderTraceTurns.Add(turn with { Calls = calls });
            }
        }
        var visibleCalls = FilteredProviderTraceTurns.SelectMany(turn => turn.Calls).ToList();
        SelectedProviderTrace = visibleCalls.FirstOrDefault(item => item.ExchangeId == selectedId)
            ?? visibleCalls.FirstOrDefault();
        OnPropertyChanged(nameof(HasProviderTraces));
        OnPropertyChanged(nameof(HasFilteredProviderTraces));
        OnPropertyChanged(nameof(ProviderTraceCountText));
        OnPropertyChanged(nameof(ProviderTraceSummary));
        OnPropertyChanged(nameof(ProviderTraceEmptyMessage));
    }

    private static bool ProviderTraceMatches(ProviderTraceItem trace, string filter)
    {
        if (string.IsNullOrWhiteSpace(filter)) return true;
        return trace.ExchangeId.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || trace.TurnId.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || trace.Provider.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || trace.ModelId.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || trace.WireModel.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || trace.ProviderRequestId.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || trace.ProviderResponseId.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || trace.InputText.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || trace.OutputText.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || trace.ToolCallsText.Contains(filter, StringComparison.OrdinalIgnoreCase);
    }

    private static bool ProviderTraceTurnMatches(ProviderTraceTurnItem turn, string filter)
    {
        if (string.IsNullOrWhiteSpace(filter)) return true;
        return turn.TurnId.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || turn.ModelId.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || turn.State.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || turn.Sequence.ToString().Contains(filter, StringComparison.OrdinalIgnoreCase);
    }

    private static ProviderTraceTurnItem ProviderTraceTurnFromJson(JsonObject item, int sequence, IReadOnlyList<ProviderTraceItem> calls) =>
        new(
            item.String("turnId", "turn_id"),
            item.String("state"),
            item.String("modelId", "model_id"),
            item.String("createdAt", "created_at"),
            item.String("startedAt", "started_at"),
            item.String("completedAt", "completed_at"),
            item.Long("inputTokens", "input_tokens"),
            item.Long("outputTokens", "output_tokens"),
            item.Long("totalTokens", "total_tokens"),
            sequence,
            calls);

    private static ProviderTraceItem ProviderTraceFromJson(JsonObject item)
    {
        var usage = item.Object("usage");
        var messages = item.Array("messages").OfType<JsonObject>().Select(message => new ProviderTraceMessageItem(
            message.String("messageId", "message_id"),
            message.String("role"),
            MessageText(message.Object("message")),
            message.String("createdAt", "created_at"))).ToList();
        var tools = item.Array("toolUses", "tool_uses").OfType<JsonObject>().Select(tool => new ProviderTraceToolItem(
            tool.String("toolCallId", "tool_call_id"),
            tool.String("name"),
            tool.String("state"),
            Pretty(tool["request"]),
            Pretty(tool["result"]),
            tool.String("errorCode", "error_code"),
            tool.String("createdAt", "created_at"))).ToList();
        return new ProviderTraceItem(
            item.String("exchangeId", "exchange_id"),
            item.String("turnId", "turn_id"),
            item.String("provider"),
            item.String("modelId", "model_id"),
            item.String("wireModel", "wire_model"),
            item.String("providerRequestId", "provider_request_id"),
            item.String("providerResponseId", "provider_response_id"),
            item.String("state"),
            item.Int("iteration"),
            item.String("startedAt", "started_at"),
            item.String("completedAt", "completed_at"),
            OptionalLong(usage, "input_tokens"),
            OptionalLong(usage, "output_tokens"),
            OptionalLong(usage, "cache_read_tokens"),
            OptionalLong(usage, "cache_write_tokens"),
            OptionalLong(usage, "total_tokens"),
            item.String("finishReason", "finish_reason"),
            Pretty(item["inputMessages"] ?? item["input_messages"]),
            OutputText(item["outputMessage"] ?? item["output_message"]),
            Pretty(item["toolCalls"] ?? item["tool_calls"]),
            Pretty(item["error"]),
            messages,
            tools);
    }

    private async Task<ProviderTraceItem> GetProviderTraceDetailsAsync(string sessionId, string exchangeId)
    {
        if (_providerTraceDetails.TryGetValue(exchangeId, out var cached)) return cached;
        if (!_providerTraceDetailLoads.TryGetValue(exchangeId, out var loading))
        {
            loading = LoadProviderTraceDetailsCoreAsync(sessionId, exchangeId);
            _providerTraceDetailLoads[exchangeId] = loading;
        }
        try
        {
            var details = await loading;
            _providerTraceDetails[exchangeId] = details;
            return details;
        }
        finally
        {
            _providerTraceDetailLoads.Remove(exchangeId);
        }
    }

    private async Task<ProviderTraceItem> LoadProviderTraceDetailsCoreAsync(string sessionId, string exchangeId)
    {
        var result = await _sdk!.ProviderExchangeAsync(sessionId, exchangeId);
        return ProviderTraceFromJson(result);
    }

    private static void PopulateProviderTraceContents(ProviderTraceItem trace, ProviderTraceItem details)
    {
        if (trace.ContentsLoaded) return;
        var contents = new List<ProviderTraceContentItem>();
        var identities = new HashSet<string>(StringComparer.Ordinal);

        void AddMessage(string role, string content, string createdAt)
        {
            if (role is not ("user" or "assistant" or "thinking") || string.IsNullOrWhiteSpace(content)) return;
            var identity = $"{role}\n{content}";
            if (!identities.Add(identity)) return;
            contents.Add(new ProviderTraceContentItem(
                trace.ExchangeId,
                role,
                role switch
                {
                    "user" => "User message",
                    "assistant" => "Assistant message",
                    _ => "Thinking message"
                },
                Preview(content),
                content,
                string.Empty,
                string.Empty,
                string.Empty,
                createdAt));
        }

        JsonArray? inputMessages = null;
        if (!string.IsNullOrWhiteSpace(details.InputText))
        {
            try
            {
                inputMessages = JsonNode.Parse(details.InputText) as JsonArray;
            }
            catch (JsonException)
            {
                // The raw request remains available in the call overview.
            }
        }
        if (inputMessages is not null)
        {
            foreach (var message in inputMessages.OfType<JsonObject>())
                AddMessage(message.String("role"), MessageText(message), string.Empty);
        }
        foreach (var message in details.Messages)
            AddMessage(message.Role, message.Content, message.CreatedAt);
        AddMessage("assistant", details.OutputText, details.CompletedAt);

        contents.AddRange(details.Tools.Select(tool => new ProviderTraceContentItem(
            trace.ExchangeId,
            "tool",
            tool.Name,
            tool.StateText,
            string.Empty,
            tool.Request,
            tool.Result,
            tool.ErrorCode,
            tool.CreatedAt)));

        trace.Contents.Clear();
        foreach (var content in contents) trace.Contents.Add(content);
        if (trace.Contents.Count == 0)
            trace.Contents.Add(ProviderTraceContentItem.Placeholder("No messages or tool uses"));
        trace.ContentsLoaded = true;
    }

    private static string Preview(string value)
    {
        var compact = string.Join(" ", value.Split((char[]?)null, StringSplitOptions.RemoveEmptyEntries));
        return compact.Length <= 72 ? compact : $"{compact[..72]}…";
    }

    private static long? OptionalLong(JsonObject value, string name)
    {
        if (value.Count == 0) return null;
        if (value[name] is not JsonValue item) return null;
        return item.TryGetValue<long>(out var result) ? result : null;
    }

    private static string OutputText(JsonNode? node)
    {
        if (node is not JsonObject message) return string.Empty;
        var text = MessageText(message);
        return string.IsNullOrWhiteSpace(text) ? Pretty(node) : text;
    }

    private static string Pretty(JsonNode? node)
    {
        if (node is null) return string.Empty;
        return node.ToJsonString(DisplayJson.Options);
    }

    private void NotifyGitDiffPresentationChanged()
    {
        OnPropertyChanged(nameof(HasGitDiffLines));
        OnPropertyChanged(nameof(ShowGitDiffEmpty));
        OnPropertyChanged(nameof(ShowGitDiffStats));
        OnPropertyChanged(nameof(GitEmptyMessage));
    }

    private static string CompactNumber(long value)
    {
        if (value < 1_000) return value.ToString("N0");
        if (value < 1_000_000) return $"{value / 1_000d:0.#}k";
        return $"{value / 1_000_000d:0.#}m";
    }

    private void ReplaceComposerAttachments(IEnumerable<ComposerAttachment> attachments)
    {
        foreach (var attachment in ComposerAttachments)
        {
            attachment.Dispose();
        }
        ComposerAttachments.Clear();
        foreach (var attachment in attachments)
        {
            ComposerAttachments.Add(attachment);
        }
    }

    private void DisposeMessages()
    {
        foreach (var message in Messages) message.Dispose();
    }

    private void DisposeSubmittedAttachments()
    {
        foreach (var attachment in _submittedAttachments.Where(attachment => !ComposerAttachments.Contains(attachment)))
            attachment.Dispose();
        _submittedAttachments = [];
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;
        Interlocked.Increment(ref _sessionLoadVersion);
        CloseSubscription();
        DisposeMessages();
        DisposeSubmittedAttachments();
        ReplaceComposerAttachments([]);
        _sdk?.Dispose();
        _sdk = null;
        lock (_initializationGate)
        {
            _initializationTask = null;
        }
    }
}
