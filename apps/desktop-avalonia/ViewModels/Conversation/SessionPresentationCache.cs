using System.Globalization;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;

namespace SunCode.Desktop.ViewModels;

internal sealed class SessionPresentationState : IDisposable
{
    internal required BulkObservableCollection<MessageItem> Messages { get; init; }
    internal required IReadOnlyList<ToolActivityTurnItem> ToolActivityTurns { get; init; }
    internal required string? SelectedToolTurnId { get; init; }
    internal required string? SelectedToolCallId { get; init; }
    internal required IReadOnlyList<ActivityItem> Activities { get; init; }
    internal required IReadOnlyList<string> ChangedPaths { get; init; }
    internal required IReadOnlyList<TodoItem> CurrentTodos { get; init; }
    internal required string ActiveTurnId { get; init; }
    internal required string ActiveTurnState { get; init; }
    internal required string LastTurnId { get; init; }
    internal bool FullControlEnabled { get; init; }

    public void Dispose()
    {
        foreach (var message in Messages) message.Dispose();
    }
}

internal sealed class SessionPresentationCache : IDisposable
{
    internal const int RecentSessionLimit = 10;
    internal static readonly TimeSpan RecentActivityWindow = TimeSpan.FromDays(1);

    private readonly Dictionary<string, SessionPresentationState> _entries = new(StringComparer.Ordinal);
    private readonly Dictionary<string, (double Offset, bool Following)> _viewports = new(StringComparer.Ordinal);

    internal void StoreViewport(string sessionId, double offset, bool following) =>
        _viewports[sessionId] = (Math.Max(0, offset), following);

    internal bool TryGetViewport(string sessionId, out double offset, out bool following)
    {
        if (_viewports.TryGetValue(sessionId, out var viewport))
        {
            offset = viewport.Offset;
            following = viewport.Following;
            return true;
        }

        offset = 0;
        following = true;
        return false;
    }

    internal bool TryTake(string sessionId, out SessionPresentationState state)
    {
        if (_entries.Remove(sessionId, out var found))
        {
            state = found;
            return true;
        }

        state = null!;
        return false;
    }

    internal void Store(string sessionId, SessionPresentationState state)
    {
        if (_entries.Remove(sessionId, out var previous)) previous.Dispose();
        _entries[sessionId] = state;
    }

    internal void Clear()
    {
        foreach (var entry in _entries.Values) entry.Dispose();
        _entries.Clear();
        _viewports.Clear();
    }

    internal void Prune(
        IReadOnlyCollection<SessionItem> sessions,
        IEnumerable<string> recentlyViewedSessionIds,
        DateTimeOffset now)
    {
        var retainedIds = RetainedSessionIds(sessions, recentlyViewedSessionIds, now);
        foreach (var sessionId in _entries.Keys.Where(id => !retainedIds.Contains(id)).ToArray())
        {
            _entries[sessionId].Dispose();
            _entries.Remove(sessionId);
        }
        foreach (var sessionId in _viewports.Keys.Where(id => !retainedIds.Contains(id)).ToArray())
            _viewports.Remove(sessionId);
    }

    internal static HashSet<string> RetainedSessionIds(
        IReadOnlyCollection<SessionItem> sessions,
        IEnumerable<string> recentlyViewedSessionIds,
        DateTimeOffset now)
    {
        var sessionsById = sessions.ToDictionary(session => session.SessionId, StringComparer.Ordinal);
        var recentIds = recentlyViewedSessionIds
            .Where(sessionsById.ContainsKey)
            .Distinct(StringComparer.Ordinal)
            .Take(RecentSessionLimit)
            .ToHashSet(StringComparer.Ordinal);

        return sessions
            .Select(ProjectActivity)
            .Where(item => recentIds.Contains(item.Session.SessionId))
            .Where(item => item.HasActivity
                && item.Activity <= now
                && now - item.Activity <= RecentActivityWindow)
            .Select(item => item.Session.SessionId)
            .ToHashSet(StringComparer.Ordinal);
    }

    private static (SessionItem Session, bool HasActivity, DateTimeOffset Activity) ProjectActivity(SessionItem session)
    {
        var hasActivity = DateTimeOffset.TryParse(
            session.LastActivityAt,
            CultureInfo.InvariantCulture,
            out var activity);
        return (session, hasActivity, hasActivity ? activity : DateTimeOffset.MinValue);
    }

    public void Dispose()
    {
        Clear();
    }
}
