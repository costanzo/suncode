using System.Collections.ObjectModel;
using SunCode.Desktop.Models;

namespace SunCode.Desktop.ViewModels;

public sealed partial class DesktopViewModel
{
    internal const int RecentContentLimit = 20;

    public ObservableCollection<RecentContentItem> RecentContents { get; } = [];
    public ObservableCollection<RecentContentItem> RecentContentOptions { get; } = [];

    public bool HasCurrentContent => SelectedEditorFile is not null || SelectedSession is not null;
    public bool CurrentContentIsFile => SelectedEditorFile is not null;
    public bool CurrentContentIsSession => SelectedEditorFile is null && SelectedSession is not null;
    public string CurrentContentTitle => CurrentContentTitleFor(SelectedSession, SelectedEditorFile);
    public string CurrentContentIconPath => SelectedEditorFile?.IconPath ?? "/Assets/icons/message.svg";
    public bool HasRecentContentOptions => RecentContentOptions.Count > 0;
    public string RecentContentCountText => $"{RecentContentOptions.Count} / {RecentContentLimit}";

    internal void RememberRecentSession(SessionItem session) =>
        RememberRecentContent(RecentContentItem.FromSession(session));

    internal void RememberRecentFile(ExplorerNode file) =>
        RememberRecentContent(RecentContentItem.FromFile(file));

    internal void RefreshRecentSessionReferences()
    {
        for (var index = RecentContents.Count - 1; index >= 0; index--)
        {
            var recent = RecentContents[index];
            if (!recent.IsSession) continue;
            var session = Sessions.FirstOrDefault(item => $"session:{item.SessionId}" == recent.ContentId);
            if (session is null)
            {
                RecentContents.RemoveAt(index);
                continue;
            }
            recent.UpdateFrom(RecentContentItem.FromSession(session));
        }
        NotifyRecentContentChanged();
    }

    internal static string CurrentContentTitleFor(SessionItem? session, ExplorerNode? file) =>
        file?.Name ?? session?.DisplayTitle ?? "No content selected";

    private void RememberRecentContent(RecentContentItem candidate)
    {
        var existing = RecentContents.FirstOrDefault(item => item.ContentId == candidate.ContentId);
        RecentContentItem current;
        if (existing is null)
        {
            current = candidate;
            RecentContents.Insert(0, current);
        }
        else
        {
            current = existing;
            current.UpdateFrom(candidate);
            var index = RecentContents.IndexOf(current);
            if (index > 0) RecentContents.Move(index, 0);
        }

        foreach (var item in RecentContents) item.IsCurrent = ReferenceEquals(item, current);
        while (RecentContents.Count > RecentContentLimit)
            RecentContents.RemoveAt(RecentContents.Count - 1);
        NotifyRecentContentChanged();
    }

    private void ClearRecentContents()
    {
        RecentContents.Clear();
        NotifyRecentContentChanged();
    }

    private void NotifyCurrentContentChanged()
    {
        OnPropertyChanged(nameof(HasCurrentContent));
        OnPropertyChanged(nameof(CurrentContentIsFile));
        OnPropertyChanged(nameof(CurrentContentIsSession));
        OnPropertyChanged(nameof(CurrentContentTitle));
        OnPropertyChanged(nameof(CurrentContentIconPath));
    }

    private void NotifyRecentContentChanged()
    {
        RecentContentOptions.Clear();
        foreach (var item in RecentContents.Where(item => !item.IsCurrent))
            RecentContentOptions.Add(item);
        OnPropertyChanged(nameof(HasRecentContentOptions));
        OnPropertyChanged(nameof(RecentContentCountText));
    }
}
