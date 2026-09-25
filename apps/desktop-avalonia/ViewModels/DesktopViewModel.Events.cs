using System.Collections.ObjectModel;
using System.Diagnostics;
using System.Globalization;
using System.Text.Json;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Threading;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Sdk;
using SunCode.Sdk.Models;

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
            var sdk = await AgentSdk.OpenAsync($"os:{Environment.UserName}");
            await sdk.GetHealthAsync();
            _sdk = sdk;
            ConnectionState = "connected";
            StatusText = "Connected to local agent";
            await LoadModelsAsync();
            await LoadAgentsAsync();
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
        CloseEditor();
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
        ResetContextUsage();
        PendingApproval = null;
        FullControlEnabled = false;
        ActiveTurnId = string.Empty;
        ActiveTurnState = string.Empty;
        UpdateActiveTurnTiming(string.Empty, null);
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

    private static ProviderTraceTurnItem ProviderTraceTurnFromSdk(
        SessionTraceTurn item,
        int sequence,
        IReadOnlyList<ProviderTraceItem> calls) =>
        new(
            item.TurnId,
            item.State,
            item.ModelId ?? string.Empty,
            item.CreatedAt,
            item.StartedAt ?? string.Empty,
            item.CompletedAt ?? string.Empty,
            (long)item.InputTokens,
            (long)item.OutputTokens,
            (long)item.TotalTokens,
            sequence,
            calls);

    private static ProviderTraceItem ProviderTraceFromSdk(ProviderExchange item)
    {
        var usage = item.Usage;
        var result = new ProviderTraceItem(
            item.ExchangeId,
            item.TurnId,
            item.Provider,
            item.ModelId,
            item.WireModel,
            item.ProviderRequestId ?? string.Empty,
            item.ProviderResponseId ?? string.Empty,
            item.State,
            item.Iteration,
            item.StartedAt,
            item.CompletedAt ?? string.Empty,
            usage is null ? null : (long)usage.InputTokens,
            usage is null ? null : (long)usage.OutputTokens,
            usage?.CacheReadTokens is { } cacheRead ? (long)cacheRead : null,
            usage?.CacheWriteTokens is { } cacheWrite ? (long)cacheWrite : null,
            usage is null ? null : (long)usage.TotalTokens,
            item.FinishReason ?? string.Empty,
            JsonSerializer.Serialize(item.InputMessages, DisplayJson.Options),
            MessageDisplayText(item.OutputMessage),
            JsonSerializer.Serialize(item.ToolCalls, DisplayJson.Options),
            Pretty(item.Error),
            [],
            []);
        return result;
    }

    private static ProviderTraceItem ProviderTraceFromSdk(ProviderExchangeDetails item)
    {
        var usage = item.Usage;
        var messages = item.Messages.Select(message => new ProviderTraceMessageItem(
            message.MessageId,
            message.Role,
            MessageText(message.Message),
            message.CreatedAt)).ToList();
        var tools = item.ToolUses.Select(tool => new ProviderTraceToolItem(
            tool.ToolCallId,
            tool.Name,
            tool.State,
            Pretty(tool.Request),
            Pretty(tool.Result),
            tool.ErrorCode ?? string.Empty,
            tool.CreatedAt)).ToList();
        var result = new ProviderTraceItem(
            item.ExchangeId,
            item.TurnId,
            item.Provider,
            item.ModelId,
            item.WireModel,
            item.ProviderRequestId ?? string.Empty,
            item.ProviderResponseId ?? string.Empty,
            item.State,
            item.Iteration,
            item.StartedAt,
            item.CompletedAt ?? string.Empty,
            usage is null ? null : (long)usage.InputTokens,
            usage is null ? null : (long)usage.OutputTokens,
            usage?.CacheReadTokens is { } cacheRead ? (long)cacheRead : null,
            usage?.CacheWriteTokens is { } cacheWrite ? (long)cacheWrite : null,
            usage is null ? null : (long)usage.TotalTokens,
            item.FinishReason ?? string.Empty,
            JsonSerializer.Serialize(item.InputMessages, DisplayJson.Options),
            MessageDisplayText(item.OutputMessage),
            JsonSerializer.Serialize(item.ToolCalls, DisplayJson.Options),
            Pretty(item.Error),
            messages,
            tools);
        return result;
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
        var result = await _sdk!.GetProviderExchangeAsync(sessionId, exchangeId);
        return ProviderTraceFromSdk(result);
    }

    private static string MessageDisplayText(AgentMessage? message)
    {
        if (message is null) return string.Empty;
        var text = MessageText(message);
        return string.IsNullOrWhiteSpace(text)
            ? JsonSerializer.Serialize(message, DisplayJson.Options)
            : text;
    }

    private static string Pretty(JsonElement? node) =>
        node is null || node.Value.ValueKind is JsonValueKind.Undefined or JsonValueKind.Null
            ? string.Empty
            : node.Value.GetRawText();

    private void NotifyGitDiffPresentationChanged()
    {
        OnPropertyChanged(nameof(HasGitDiffLines));
        OnPropertyChanged(nameof(ShowGitDiffEmpty));
        OnPropertyChanged(nameof(ShowGitDiffStats));
        OnPropertyChanged(nameof(GitEmptyMessage));
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
        _conversationDurationTimer.Stop();
        _conversationDurationTimer.Tick -= ConversationDurationTick;
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
