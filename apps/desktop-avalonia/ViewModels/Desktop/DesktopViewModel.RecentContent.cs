using SunCode.Desktop.Models;

namespace SunCode.Desktop.ViewModels;

public sealed partial class DesktopViewModel
{
    public RecentContentViewModel RecentContent { get; } = new();

    public bool HasCurrentContent => SelectedEditorFile is not null || SelectedSession is not null || SelectedChildSession is not null;
    public bool CurrentContentIsFile => SelectedEditorFile is not null;
    public bool CurrentContentIsSession => SelectedEditorFile is null && SelectedChildSession is null && SelectedSession is not null;
    public bool CurrentContentIsChildSession => SelectedChildSession is not null;
    public string CurrentContentTitle => SelectedChildSession?.Title ?? CurrentContentTitleFor(SelectedSession, SelectedEditorFile);
    public string CurrentContentIconPath => SelectedEditorFile?.IconPath ?? (SelectedChildSession is not null ? "/Assets/icons/agent.svg" : "/Assets/icons/message.svg");

    internal void RememberRecentSession(SessionItem session) =>
        RememberRecentContent(RecentContentItem.FromSession(session));

    internal void RememberRecentFile(ExplorerNode file) =>
        RememberRecentContent(RecentContentItem.FromFile(file));

    internal void RememberRecentChildSession(ChildSessionItem child) =>
        RememberRecentContent(RecentContentItem.FromChildSession(child));

    internal void RefreshRecentSessionReferences() => RecentContent.RefreshSessionReferences(Sessions);

    internal void RefreshRecentChildSessionReferences() =>
        RecentContent.RefreshChildSessionReferences(SelectedSession?.SessionId, ChildSessions);

    internal static string CurrentContentTitleFor(SessionItem? session, ExplorerNode? file) =>
        file?.Name ?? session?.DisplayTitle ?? "No content selected";

    private void RememberRecentContent(RecentContentItem candidate)
    {
        RecentContent.Remember(candidate);
        SaveProjectUiState(saved =>
        {
            saved.RecentContent = RecentContent.ToUiState();
            SaveCurrentContentState(saved);
        });
    }

    private void ClearRecentContents() => RecentContent.Clear();

    private void NotifyCurrentContentChanged()
    {
        OnPropertyChanged(nameof(HasCurrentContent));
        OnPropertyChanged(nameof(CurrentContentIsFile));
        OnPropertyChanged(nameof(CurrentContentIsSession));
        OnPropertyChanged(nameof(CurrentContentIsChildSession));
        OnPropertyChanged(nameof(CurrentContentTitle));
        OnPropertyChanged(nameof(CurrentContentIconPath));
    }
}
