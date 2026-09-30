using System.Collections.ObjectModel;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;

namespace SunCode.Desktop.ViewModels;

// Most-recent-first files, sessions, and child sessions viewed in one project
// window. The current item stays in Items for history but is left out of the
// switcher Options. The window decides what is current and persists the list.
public sealed class RecentContentViewModel : ObservableObject
{
    internal const int Limit = 20;

    public ObservableCollection<RecentContentItem> Items { get; } = [];
    public ObservableCollection<RecentContentItem> Options { get; } = [];
    public bool HasOptions => Options.Count > 0;
    public string CountText => $"{Options.Count} / {Limit}";

    internal void Remember(RecentContentItem candidate)
    {
        var existing = Items.FirstOrDefault(item => item.ContentId == candidate.ContentId);
        RecentContentItem current;
        if (existing is null)
        {
            current = candidate;
            Items.Insert(0, current);
        }
        else
        {
            current = existing;
            current.UpdateFrom(candidate);
            var index = Items.IndexOf(current);
            if (index > 0) Items.Move(index, 0);
        }

        foreach (var item in Items) item.IsCurrent = ReferenceEquals(item, current);
        while (Items.Count > Limit)
            Items.RemoveAt(Items.Count - 1);
        RefreshOptions();
    }

    internal bool Contains(string contentId) => Items.Any(item => item.ContentId == contentId);

    internal void AddIfMissing(RecentContentItem item)
    {
        if (!Contains(item.ContentId)) Items.Add(item);
    }

    internal void ReplaceAll(IEnumerable<RecentContentItem> items)
    {
        Items.Clear();
        foreach (var item in items) Items.Add(item);
        RefreshOptions();
    }

    internal void Clear()
    {
        Items.Clear();
        RefreshOptions();
    }

    // Drops sessions that no longer exist and refreshes titles of the rest.
    internal void RefreshSessionReferences(IReadOnlyCollection<SessionItem> sessions)
    {
        for (var index = Items.Count - 1; index >= 0; index--)
        {
            var recent = Items[index];
            if (!recent.IsSession) continue;
            var session = sessions.FirstOrDefault(item => $"session:{item.SessionId}" == recent.ContentId);
            if (session is null)
            {
                Items.RemoveAt(index);
                continue;
            }
            recent.UpdateFrom(RecentContentItem.FromSession(session));
        }
        RefreshOptions();
    }

    // Only child sessions of the loaded parent can be checked; others are kept.
    internal void RefreshChildSessionReferences(string? parentSessionId, IReadOnlyCollection<ChildSessionItem> children)
    {
        for (var index = Items.Count - 1; index >= 0; index--)
        {
            var recent = Items[index];
            if (!recent.IsChildSession) continue;
            if (parentSessionId is not null && recent.ChildSession?.ParentSessionId != parentSessionId) continue;
            var child = children.FirstOrDefault(item => $"child-session:{item.SessionId}" == recent.ContentId);
            if (child is null) { Items.RemoveAt(index); continue; }
            recent.UpdateFrom(RecentContentItem.FromChildSession(child));
        }
        RefreshOptions();
    }

    internal List<UiRecentContentState> ToUiState() => Items.Select(item => item.IsSession
        ? new UiRecentContentState { Kind = "session", SessionId = item.Session?.SessionId, LastViewedAt = DateTimeOffset.UtcNow }
        : item.IsChildSession
            ? new UiRecentContentState { Kind = "child-session", SessionId = item.ChildSession?.SessionId, ParentSessionId = item.ChildSession?.ParentSessionId, LastViewedAt = DateTimeOffset.UtcNow }
            : new UiRecentContentState { Kind = "file", DependencyId = item.File?.DependencyId, Path = item.File?.Path, LastViewedAt = DateTimeOffset.UtcNow }).ToList();

    internal void RefreshOptions()
    {
        Options.Clear();
        foreach (var item in Items.Where(item => !item.IsCurrent))
            Options.Add(item);
        OnPropertyChanged(nameof(HasOptions));
        OnPropertyChanged(nameof(CountText));
    }
}
