using Avalonia.Threading;
using SunCode.Sdk;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Infrastructure;

internal sealed record SystemNotification(
    string Id,
    string Title,
    string Body,
    DesktopActivationRequest Activation);

internal interface ISystemNotificationBackend : IDisposable
{
    event Action<DesktopActivationRequest>? Activated;
    Task InitializeAsync(CancellationToken cancellationToken);
    Task ShowAsync(SystemNotification notification, CancellationToken cancellationToken);
}

internal sealed class SessionAttentionCoordinator : IDisposable
{
    private readonly Func<bool> _isForeground;
    private readonly Func<DesktopActivationRequest, Task> _activate;
    private readonly NotificationLedger _ledger;
    private readonly ISystemNotificationBackend _notifications;
    private readonly CancellationTokenSource _shutdown = new();
    private readonly HashSet<string> _inFlight = new(StringComparer.Ordinal);
    private readonly object _deliveryGate = new();
    private AgentSdk? _sdk;
    private IDisposable? _subscription;
    private bool _disposed;

    public SessionAttentionCoordinator(
        Func<bool> isForeground,
        Func<DesktopActivationRequest, Task> activate,
        ISystemNotificationBackend notifications,
        string dataDirectory)
    {
        _isForeground = isForeground;
        _activate = activate;
        _notifications = notifications;
        _ledger = new NotificationLedger(dataDirectory);
        _notifications.Activated += NotificationActivated;
    }

    public async Task StartAsync()
    {
        if (_disposed || _sdk is not null) return;
        try
        {
            try
            {
                await _notifications.InitializeAsync(_shutdown.Token);
            }
            catch (Exception exception)
            {
                DiagnosticLog.Warn("notification.backend", $"initialize_failed=true error={exception.Message}");
            }
            _sdk = await AgentSdk.OpenAsync($"os:{Environment.UserName}");
            Subscribe();
            await ReconcileAsync(DateTimeOffset.UtcNow.ToString("O"));
        }
        catch (Exception exception)
        {
            DiagnosticLog.Warn("notification.coordinator", $"start_failed=true error={exception.Message}");
        }
    }

    private void Subscribe()
    {
        _subscription?.Dispose();
        _subscription = _sdk?.SubscribeAttention(message =>
        {
            if (_disposed) return;
            if (message.RequiresResync)
            {
                _ = Task.Run(ReconcileAndResubscribeAsync);
                return;
            }
            if (message.Event is { } attention) _ = HandleAsync(attention);
        });
    }

    private async Task ReconcileAndResubscribeAsync()
    {
        try
        {
            await ReconcileAsync();
            if (!_disposed) Subscribe();
        }
        catch (Exception exception)
        {
            DiagnosticLog.Warn("notification.coordinator", $"resync_failed=true error={exception.Message}");
        }
    }

    private async Task ReconcileAsync(string? since = null)
    {
        if (_sdk is null) return;
        var candidates = await _sdk.ListAttentionCandidatesAsync(since, 512);
        foreach (var candidate in candidates.Candidates) await HandleAsync(candidate);
    }

    private async Task HandleAsync(AttentionEvent attention)
    {
        var ledgerKey = new NotificationLedgerKey(attention.Kind, attention.CorrelationId);
        var deliveryKey = ledgerKey.ToString();
        lock (_deliveryGate)
        {
            if (_disposed || _ledger.Contains(ledgerKey) || !_inFlight.Add(deliveryKey)) return;
        }
        try
        {
            if (_isForeground())
            {
                _ledger.Record(ledgerKey, NotificationDisposition.SuppressedForeground);
                return;
            }
            var notification = BuildNotification(attention);
            await _notifications.ShowAsync(notification, _shutdown.Token);
            _ledger.Record(ledgerKey, NotificationDisposition.Delivered);
        }
        catch (Exception exception) when (exception is not OperationCanceledException)
        {
            DiagnosticLog.Warn("notification.delivery", $"kind={attention.Kind} failed=true error={exception.Message}");
        }
        finally
        {
            lock (_deliveryGate) _inFlight.Remove(deliveryKey);
        }
    }

    internal static SystemNotification BuildNotification(AttentionEvent attention)
    {
        var project = Bounded(attention.ProjectDisplayName, "Project", 80);
        var session = Bounded(attention.SessionTitle, "Session", 100);
        var body = attention.Kind switch
        {
            "primary_turn_completed" => $"{session} completed",
            "primary_turn_failed" => $"{session} failed",
            "approval_requested" => $"{session} needs your approval",
            "question_asked" => $"{session} needs your answer",
            _ => throw new InvalidDataException("Unknown attention event kind")
        };
        var activation = DesktopActivationRequest.Session(
            attention.ProjectId,
            attention.SessionId,
            attention.CorrelationId,
            "notification",
            attention.SessionKind == "child" ? attention.ParentSessionId : null,
            attention.SessionKind == "child" ? attention.SessionId : null);
        activation.Validate();
        return new SystemNotification(attention.CorrelationId, $"SunCode · {project}", body, activation);
    }

    private static string Bounded(string value, string fallback, int maximum)
    {
        var text = string.IsNullOrWhiteSpace(value) ? fallback : value.Trim();
        return text.Length <= maximum ? text : text[..(maximum - 1)] + "…";
    }

    private void NotificationActivated(DesktopActivationRequest request) =>
        Dispatcher.UIThread.Post(() => _ = _activate(request));

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;
        _shutdown.Cancel();
        _subscription?.Dispose();
        _subscription = null;
        _notifications.Activated -= NotificationActivated;
        _notifications.Dispose();
        _sdk?.Dispose();
        _sdk = null;
        _shutdown.Dispose();
    }
}
