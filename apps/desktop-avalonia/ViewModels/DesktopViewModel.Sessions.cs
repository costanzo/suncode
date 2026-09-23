using System.Collections.Specialized;
using SunCode.Desktop.Models;

namespace SunCode.Desktop.ViewModels;

public sealed partial class DesktopViewModel
{
    internal void AttachSessionCollectionListeners() =>
        Sessions.CollectionChanged += OnSessionsCollectionChanged;
    
    private void OnSessionsCollectionChanged(object? sender, NotifyCollectionChangedEventArgs e) =>
        NotifyRunningSessionsChanged();
    
    public int RunningSessionCount => CountRunningSessions();
    public bool HasRunningSessions => RunningSessionCount > 0;
    public string RunningSessionText => RunningSessionCount switch
    {
        0 => string.Empty,
        1 => "1 running",
        _ => $"{RunningSessionCount} running"
    };
    public int WaitingSessionCount => CountWaitingSessions();
    public bool HasWaitingSessions => WaitingSessionCount > 0;
    public string WaitingSessionText => WaitingSessionCount switch
    {
        0 => string.Empty,
        1 => "1 waiting",
        _ => $"{WaitingSessionCount} waiting"
    };
    
    private int CountRunningSessions()
    {
        var count = 0;
        foreach (var session in Sessions)
        {
            if (session.IsTurnActive) count++;
        }
        return count;
    }
    
    private int CountWaitingSessions()
    {
        var count = 0;
        foreach (var session in Sessions)
        {
            if (session.IsWaitingOnUser) count++;
        }
        return count;
    }

    internal void NotifyRunningSessionsChanged()
    {
        OnPropertyChanged(nameof(RunningSessionCount));
        OnPropertyChanged(nameof(HasRunningSessions));
        OnPropertyChanged(nameof(RunningSessionText));
        OnPropertyChanged(nameof(WaitingSessionCount));
        OnPropertyChanged(nameof(HasWaitingSessions));
        OnPropertyChanged(nameof(WaitingSessionText));
    }

    internal void SyncSelectedSessionAgentState(string agentState)
    {
        if (SelectedSession is not { } selected) return;
        if (string.Equals(selected.AgentState, agentState, StringComparison.Ordinal)) return;
        var index = Sessions.IndexOf(selected);
        if (index < 0) return;
        var updated = selected with { AgentState = agentState };
        Sessions[index] = updated;
        SelectedSession = updated;
        NotifyRunningSessionsChanged();
    }

    internal void SelectComposerModel(ModelItem model)
    {
        SelectedModel = model;
        if (SelectedSession is not { } selected) return;
        var reasoningEffort = SelectedReasoningEffort ?? string.Empty;
        if (string.Equals(selected.ModelId, model.Id, StringComparison.Ordinal) &&
            string.Equals(selected.ReasoningEffort, reasoningEffort, StringComparison.Ordinal)) return;
        var updated = selected with { ModelId = model.Id, ReasoningEffort = reasoningEffort };
        var index = Sessions.IndexOf(selected);
        if (index >= 0) Sessions[index] = updated;
        SelectedSession = updated;
    }

    internal void SyncSelectedSessionAgentStateFromReview()
    {
        var state = HasPendingApproval
            ? "approval"
            : HasPendingQuestion
                ? "question"
                : HasFailedTurn
                    ? "failed"
                    : IsTurnActive
                        ? "running"
                        : "idle";
        SyncSelectedSessionAgentState(state);
    }
}