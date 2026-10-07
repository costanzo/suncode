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
            await Remote.StartPollingAsync();
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

    // Background refreshes must not surface as unobserved task exceptions.
    // Failures are logged; the next event or explicit reload retries.
    private static async void RunInBackground(Func<Task> operation, string operationName)
    {
        try
        {
            await operation();
        }
        catch (Exception exception)
        {
            DiagnosticLog.Error("viewmodel.background", exception, $"operation={operationName}");
        }
    }

    AgentSdk? IViewModelHost.Sdk => _sdk;
    bool IViewModelHost.EnsureSdk() => EnsureSdk();
    Task<bool> IViewModelHost.EnsureSdkReadyAsync() => EnsureSdkReadyAsync();
    void IViewModelHost.ReportError(Exception exception) => ReportError(exception);
    void IViewModelHost.SetBusy(bool busy) => IsBusy = busy;
    void IViewModelHost.ReportPresentationError(string message) => ReportPresentationError(message);

    string? IProviderTraceHost.SelectedSessionId => SelectedSession?.SessionId;
    string? IProviderTraceHost.SelectedModelId => SelectedModel?.Id;
    bool IProviderTraceHost.IsSessionContextCurrent(string sessionId, long? loadVersion) => IsSessionContextCurrent(sessionId, loadVersion);
    void IProviderTraceHost.UpdateContextUsage(AgentUsage usage) => UpdateContextUsage(usage);

    void IViewModelHost.ReportSuccess(string message)
    {
        StatusText = message;
        ConnectionState = "connected";
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
        Git.ClearDiffLines();
        DisposeSubmittedAttachments();
        ReplaceComposerAttachments([]);
        ProviderTrace.Clear();
        ContextUsage.Reset();
        PendingApproval = null;
        FullControlEnabled = false;
        PendingQuestion = null;
        CurrentTodos.Clear();
        ClearSelectedChildSession();
        ChildSessions.Clear();
        ActiveTurnId = string.Empty;
        ActiveTurnState = string.Empty;
        UpdateActiveTurnTiming(string.Empty, null);
        ProviderTraffic.Clear();
        if (clearSelection) SelectedSession = null;
        OnPropertyChanged(nameof(HasActivities));
        OnPropertyChanged(nameof(HasCheckpoints));
        OnPropertyChanged(nameof(HasChangedPaths));
        OnPropertyChanged(nameof(TurnChangeSummary));
        OnPropertyChanged(nameof(HasCurrentTodos));
        OnPropertyChanged(nameof(HasChildSessions));
        NotifyReviewPresentationChanged();
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

    private static string Pretty(JsonElement? node) => SessionSnapshotProjector.JsonText(node);

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
        Remote.Dispose();
        AppSettings.LanguageChanged -= OnAppLanguageChanged;
        _conversationDurationTimer.Tick -= ConversationDurationTick;
        ProviderTraffic.Clear();
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
