using SunCode.Desktop.Infrastructure;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

public sealed partial class DesktopViewModel
{
    private readonly TimeProvider _timeProvider;
    private ProviderRetryWait? _providerRetryWait;

    private sealed record ProviderRetryWait(long StartedAt, ulong DelayMs, uint Attempt, uint MaxRetries);

    private void StartProviderRetry(AgentEventPayload payload)
    {
        if (_disposed) return;
        _providerRetryWait = new(_timeProvider.GetTimestamp(), payload.DelayMs ?? 0, payload.RetryAttempt ?? 0, payload.MaxRetries ?? 2);
        RefreshProviderRetryStatus();
    }

    internal void RefreshProviderRetryStatus()
    {
        if (_disposed || _providerRetryWait is not { } wait) return;
        var elapsedMs = _timeProvider.GetElapsedTime(wait.StartedAt).TotalMilliseconds;
        var seconds = Math.Max(0, Math.Ceiling((wait.DelayMs - elapsedMs) / 1000));
        ProviderRetryText = seconds > 0
            ? string.Format(LocalizationService.GetString("LocProviderRetryScheduled", "Retrying request in {0}s · retry {1} of {2}"), seconds, wait.Attempt, wait.MaxRetries)
            : string.Format(LocalizationService.GetString("LocProviderRetryStarting", "Waiting for retry to start · retry {0} of {1}"), wait.Attempt, wait.MaxRetries);
    }

    private void ClearProviderRetryStatus()
    {
        _providerRetryWait = null;
        ProviderRetryText = string.Empty;
    }
}
