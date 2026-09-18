using System.Text.Json;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

public sealed partial class DesktopViewModel
{
    public async Task LoadChildSessionsAsync()
    {
        ChildSessions.Clear();
        OnPropertyChanged(nameof(HasChildSessions));
        if (_sdk is null || SelectedSession is null) return;
        var result = await _sdk.ListChildSessionsAsync(SelectedSession.SessionId);
        foreach (var child in ProjectChildSessions(result)) ChildSessions.Add(child);
        foreach (var saved in SavedUiProjectState.RecentContent.Where(item => item.Kind == "child-session" && !string.IsNullOrWhiteSpace(item.SessionId)))
        {
            var child = ChildSessions.FirstOrDefault(item => item.SessionId == saved.SessionId);
            if (child is not null && RecentContents.All(item => item.ContentId != $"child-session:{child.SessionId}"))
                RecentContents.Add(RecentContentItem.FromChildSession(child));
        }
        OnPropertyChanged(nameof(HasChildSessions));
        RefreshRecentChildSessionReferences();
        var current = SavedUiProjectState;
        if (current.CurrentContentKind == "child-session" && current.CurrentSessionId is { Length: > 0 } childSessionId)
        {
            var child = ChildSessions.FirstOrDefault(item => item.SessionId == childSessionId);
            if (child is not null) await SelectChildSessionAsync(child);
        }
    }

    public async Task SelectChildSessionAsync(ChildSessionItem child)
    {
        if (_sdk is null) return;
        if (SelectedSession?.SessionId != child.ParentSessionId)
        {
            var parent = Sessions.FirstOrDefault(item => item.SessionId == child.ParentSessionId);
            if (parent is null) return;
            await SelectSessionAsync(parent);
            child = ChildSessions.FirstOrDefault(item => item.SessionId == child.SessionId) ?? child;
        }
        CloseEditor();
        SelectedChildSession = child;
        ChildPendingApproval = null;
        RememberRecentChildSession(child);
        ChildSessionsVisible = true;
        ChildSessionTimeline.Clear();
        if (!string.IsNullOrWhiteSpace(child.Task))
            ChildSessionTimeline.Add(new ChildSessionTimelineItem("task", "Task from Main Agent", child.Task, child.StateText, child.CreatedAt));
        var snapshot = await _sdk.GetSessionSnapshotAsync(child.SessionId);
        foreach (var turn in snapshot.ConversationTurns)
        {
            foreach (var message in turn.Messages.Where(message => message.Role is "assistant" or "thinking"))
            {
                var text = message.Message.Text;
                if (!string.IsNullOrWhiteSpace(text))
                    ChildSessionTimeline.Add(new ChildSessionTimelineItem("message", child.AgentDisplayName, text, turn.State, message.CreatedAt));
            }
            foreach (var tool in turn.ToolUses)
            {
                var detail = tool.Result?.ToString() ?? tool.Request?.ToString() ?? string.Empty;
                ChildSessionTimeline.Add(new ChildSessionTimelineItem("tool", tool.Name, detail, tool.State, tool.CreatedAt));
            }
        }
        if (child.HasApproval)
        {
            try
            {
                ChildPendingApproval = ApprovalItem.FromSdk(await _sdk.GetApprovalAsync(child.ApprovalId));
            }
            catch
            {
                ChildPendingApproval = null;
            }
        }
    }

    public async Task ResolveChildApprovalAsync(string decision)
    {
        if (!EnsureSdk() || ChildPendingApproval is null || SelectedChildSession is null) return;
        var approval = ChildPendingApproval;
        var childSessionId = SelectedChildSession.SessionId;
        await RunAsync(async () =>
        {
            await _sdk!.ResolveApprovalAsync(new ApprovalDecisionRequest(
                approval.ApprovalId,
                decision == "allow_once" ? ApprovalDecision.AllowOnce : ApprovalDecision.Deny));
            ChildPendingApproval = null;
            await LoadChildSessionsAsync();
        }, decision == "allow_once" ? "Child-session action approved" : "Child-session action denied");
        _ = MonitorChildSessionAsync(childSessionId);
    }

    private async Task MonitorChildSessionAsync(string childSessionId)
    {
        try
        {
            for (var attempt = 0; attempt < 30; attempt++)
            {
                await Task.Delay(TimeSpan.FromSeconds(1));
                if (_disposed || _sdk is null || SelectedSession is null) return;
                await LoadChildSessionsAsync();
                var child = ChildSessions.FirstOrDefault(item => item.SessionId == childSessionId);
                if (child is null || !child.IsRunning) return;
            }
        }
        catch (Exception exception)
        {
            DiagnosticLog.Warn("subagent", $"monitor_failed session={childSessionId} error={exception.Message}");
        }
    }

    private async Task RestoreSavedChildRecentContentsAsync(UiProjectState saved)
    {
        if (_sdk is null) return;
        foreach (var parentId in saved.RecentContent
                     .Where(item => item.Kind == "child-session" && !string.IsNullOrWhiteSpace(item.ParentSessionId))
                     .Select(item => item.ParentSessionId!)
                     .Distinct(StringComparer.Ordinal))
        {
            if (Sessions.All(item => item.SessionId != parentId)) continue;
            var result = await _sdk.ListChildSessionsAsync(parentId);
            foreach (var child in ProjectChildSessions(result))
            {
                if (saved.RecentContent.Any(item => item.Kind == "child-session" && item.SessionId == child.SessionId)
                    && RecentContents.All(item => item.ContentId != $"child-session:{child.SessionId}"))
                    RecentContents.Add(RecentContentItem.FromChildSession(child));
            }
        }
        var ordered = saved.RecentContent
            .Select(item => item.Kind switch
            {
                "session" => RecentContents.FirstOrDefault(recent => recent.ContentId == $"session:{item.SessionId}"),
                "child-session" => RecentContents.FirstOrDefault(recent => recent.ContentId == $"child-session:{item.SessionId}"),
                "file" => RecentContents.FirstOrDefault(recent => recent.ContentId == $"file:{item.DependencyId ?? "project"}:{item.Path}"),
                _ => null
            })
            .Where(item => item is not null)
            .Cast<RecentContentItem>()
            .DistinctBy(item => item.ContentId)
            .Take(RecentContentLimit)
            .ToArray();
        RecentContents.Clear();
        foreach (var item in ordered) RecentContents.Add(item);
        NotifyRecentContentChanged();
    }

    private IReadOnlyList<ChildSessionItem> ProjectChildSessions(ChildSessionsResult result)
    {
        var agents = Agents.ToDictionary(agent => agent.Id, StringComparer.Ordinal);
        var invocations = result.Invocations.ToDictionary(value => value.ChildSessionId, StringComparer.Ordinal);
        return result.Sessions.Select(session =>
        {
            invocations.TryGetValue(session.SessionId, out var invocation);
            agents.TryGetValue(session.AgentId ?? string.Empty, out var agent);
            var task = invocation is null ? string.Empty : JsonText(invocation.Task, "text");
            var resultText = invocation?.Result is { } value ? JsonText(value, "result") : string.Empty;
            var approvalId = invocation?.Result is { } invocationResult ? JsonText(invocationResult, "approvalId") : string.Empty;
            var state = invocation?.State
                ?? (result.SessionStates.TryGetValue(session.SessionId, out var sessionState) ? sessionState : "idle");
            return new ChildSessionItem(
                session.SessionId,
                session.ParentSessionId ?? result.ParentSessionId,
                session.Title ?? "Delegated task",
                session.AgentId ?? string.Empty,
                agent?.DisplayName ?? session.AgentId ?? "Agent",
                state,
                invocation?.CreatedAt ?? session.CreatedAt,
                session.ModelId ?? string.Empty,
                task,
                resultText,
                approvalId);
        }).ToArray();
    }

    public void ClearSelectedChildSession()
    {
        SelectedChildSession = null;
        ChildPendingApproval = null;
        ChildSessionTimeline.Clear();
    }

    private static string JsonText(JsonElement value, string property) =>
        value.ValueKind == JsonValueKind.Object
        && value.TryGetProperty(property, out var text)
        && text.ValueKind == JsonValueKind.String
            ? text.GetString() ?? string.Empty
            : string.Empty;
}
