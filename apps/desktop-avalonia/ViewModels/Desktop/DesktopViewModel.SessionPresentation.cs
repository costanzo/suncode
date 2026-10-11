using SunCode.Desktop.Models;

namespace SunCode.Desktop.ViewModels;

public sealed partial class DesktopViewModel
{
    private readonly SessionPresentationCache _sessionPresentationCache = new();

    internal void StoreSessionViewport(string sessionId, double offset, bool following) =>
        _sessionPresentationCache.StoreViewport(sessionId, offset, following);

    internal bool TryGetSessionViewport(string sessionId, out double offset, out bool following) =>
        _sessionPresentationCache.TryGetViewport(sessionId, out offset, out following);

    private void CacheCurrentSessionPresentation(string sessionId)
    {
        SessionLeaving?.Invoke(sessionId);
        var state = new SessionPresentationState
        {
            Messages = Messages,
            ToolActivityTurns = ToolActivity.Turns.ToArray(),
            SelectedToolTurnId = ToolActivity.SelectedTurn?.TurnId,
            SelectedToolCallId = ToolActivity.SelectedTool?.ToolCallId,
            Activities = Activities.ToArray(),
            ChangedPaths = ChangedPaths.ToArray(),
            CurrentTodos = CurrentTodos.ToArray(),
            ActiveTurnId = ActiveTurnId,
            ActiveTurnState = ActiveTurnState,
            LastTurnId = LastTurnId,
            FullControlEnabled = FullControlEnabled
        };
        Messages = [];
        _sessionPresentationCache.Store(sessionId, state);
        PruneSessionPresentationCache();
    }

    private bool RestoreSessionPresentation(string sessionId)
    {
        if (!_sessionPresentationCache.TryTake(sessionId, out var state)) return false;

        _appliedMessageIds.Clear();
        foreach (var message in state.Messages)
        {
            if (message.MessageId.Length > 0) _appliedMessageIds.Add(message.MessageId);
        }
        DisposeMessages();
        Messages = state.Messages;
        ToolActivity.ReplaceAll(state.ToolActivityTurns);
        if (state.SelectedToolTurnId is { } turnId && state.SelectedToolCallId is { } callId)
            ToolActivity.TrySelect(turnId, callId);
        SyncActiveToolRow();
        Activities.ReplaceAll(state.Activities);
        ChangedPaths.ReplaceAll(state.ChangedPaths);
        CurrentTodos.ReplaceAll(state.CurrentTodos);
        FullControlEnabled = state.FullControlEnabled;
        LastTurnId = state.LastTurnId;
        ActiveTurnId = state.ActiveTurnId;
        ActiveTurnState = state.ActiveTurnState;
        var activeActivityTurn = ToolActivity.Turns.LastOrDefault(item => item.TurnId == state.ActiveTurnId);
        UpdateActiveTurnTiming(state.ActiveTurnId, activeActivityTurn?.StartedAt);
        NotifySessionPresentationChanged();
        return true;
    }

    private void PruneSessionPresentationCache() =>
        _sessionPresentationCache.Prune(
            Sessions.Concat(ArchivedSessions).ToArray(),
            RecentContent.Items.Where(item => item.IsSession && item.Session is not null)
                .Select(item => item.Session!.SessionId),
            DateTimeOffset.Now);

    private void NotifySessionPresentationChanged()
    {
        OnPropertyChanged(nameof(HasMessages));
        OnPropertyChanged(nameof(HasActivities));
        OnPropertyChanged(nameof(HasChangedPaths));
        OnPropertyChanged(nameof(TurnChangeSummary));
        OnPropertyChanged(nameof(HasCurrentTodos));
        OnPropertyChanged(nameof(IsReviewTodosVisible));
        OnPropertyChanged(nameof(LatestActivityText));
        NotifyAssistantStreamingChanged();
        NotifyReviewPresentationChanged();
    }
}
