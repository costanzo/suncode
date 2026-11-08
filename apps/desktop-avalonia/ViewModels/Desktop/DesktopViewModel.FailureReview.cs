using SunCode.Desktop.Models;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

public sealed partial class DesktopViewModel
{
    private TurnFailureItem? _latestProviderCall;
    private TurnFailureItem? _reviewFailure;

    public TurnFailureItem? ReviewFailure
    {
        get => _reviewFailure;
        private set => SetProperty(ref _reviewFailure, value);
    }

    private void ObserveProviderFailure(string type, AgentEventPayload payload)
    {
        if (type == AgentEventTypes.ProviderExchangeStarted)
        {
            _latestProviderCall = new(payload.TurnId ?? string.Empty, string.Empty, string.Empty,
                payload.ExchangeId ?? string.Empty, payload.Provider ?? string.Empty, payload.ModelId ?? string.Empty);
        }
        else if (type == AgentEventTypes.ProviderExchangeFailed && payload.Error is { } error)
        {
            var call = _latestProviderCall?.ExchangeId == payload.ExchangeId ? _latestProviderCall : null;
            _latestProviderCall = new(payload.TurnId ?? string.Empty, error.Code, error.Message,
                payload.ExchangeId ?? string.Empty, call?.Provider ?? string.Empty, call?.ModelId ?? string.Empty);
        }
        else if (type == AgentEventTypes.ProviderExchangeCompleted)
        {
            _latestProviderCall = null;
        }
    }

    private void SetReviewFailure(string turnId, string? code)
    {
        var failure = _latestProviderCall;
        ReviewFailure = failure is { Message.Length: > 0 } && failure.TurnId == turnId && failure.Code == code
            ? failure
            : new(turnId, code ?? string.Empty, string.Empty);
    }

    internal void RefreshReviewFailure()
    {
        if (!HasFailedTurn || ReviewFailure is not { } current) return;
        var turn = ProviderTrace.Turns.FirstOrDefault(item => item.TurnId == current.TurnId);
        var code = current.Code.Length > 0 ? current.Code : turn?.ErrorCode ?? string.Empty;
        var failure = ProviderTrace.FindFailure(current.TurnId, code, current.ExchangeId);
        ReviewFailure = failure ?? current with { Code = code };
    }

    public async Task OpenFailureDetailsAsync()
    {
        if (!HasFailedTurn || ReviewFailure is not { } failure) return;
        Layout.OpenBottomDrawer(WorkspaceBottomDrawer.ProviderTrace);
        await ProviderTrace.RevealFailureAsync(failure.TurnId, failure.ExchangeId, failure.Code);
    }
}
