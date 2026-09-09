using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;

namespace SunCode.Desktop.ViewModels;

public sealed partial class DesktopViewModel
{
    private readonly UiStateStore? _uiStateStore;
    private bool _restoringUiState;
    internal bool RestoringSavedFile { get; set; }

    internal DesktopViewModel(UiStateStore? uiStateStore = null) => _uiStateStore = uiStateStore;

    internal UiProjectState SavedUiProjectState => SelectedProject is { } project
        ? _uiStateStore?.Project(project.ProjectId) ?? new UiProjectState()
        : new UiProjectState();

    internal void SaveSettingsNavigation(string page, string? providerId, bool providersExpanded)
    {
        _uiStateStore?.UpdateSettings(state =>
        {
            state.Page = page;
            state.ProviderId = providerId;
            state.ProvidersExpanded = providersExpanded;
        });
    }

    internal UiSettingsState SavedSettingsNavigation => _uiStateStore?.Settings ?? new UiSettingsState();

    private void SaveProjectUiState(Action<UiProjectState> update)
    {
        if (_restoringUiState || SelectedProject is not { } project) return;
        _uiStateStore?.UpdateProject(project.ProjectId, update);
    }

    private void SaveCurrentContentState(UiProjectState state)
    {
        if (SelectedEditorFile is { } file)
        {
            state.CurrentContentKind = "file";
            state.CurrentSessionId = null;
            state.CurrentDependencyId = file.DependencyId;
            state.CurrentFilePath = file.Path;
        }
        else if (SelectedSession is { } session)
        {
            state.CurrentContentKind = "session";
            state.CurrentSessionId = session.SessionId;
            state.CurrentDependencyId = null;
            state.CurrentFilePath = null;
            state.LastSessionId = session.SessionId;
        }
        else
        {
            state.CurrentContentKind = null;
            state.CurrentSessionId = null;
            state.CurrentDependencyId = null;
            state.CurrentFilePath = null;
        }
    }

    internal void RestoreProjectPresentationState()
    {
        if (SelectedProject is not { } project || _uiStateStore is null) return;
        var saved = _uiStateStore.Project(project.ProjectId);
        _restoringUiState = true;
        try
        {
            NavigationVisible = saved.LeftRegion != "closed";
            ExplorerVisible = saved.LeftRegion == "explorer";
            ReviewVisible = saved.RightRegion != "closed";
            GitVisible = saved.BottomDrawer == "git";
            ProviderTraceVisible = saved.BottomDrawer == "providerTrace";
            ToolActivityVisible = saved.BottomDrawer == "toolActivity";
            if (saved.NavigationWidth is { } navigationWidth) NavigationPaneWidth = Math.Clamp(navigationWidth, 236, 300);
            if (saved.ReviewWidth is { } reviewWidth) ReviewPaneWidth = Math.Clamp(reviewWidth, 276, 352);
            if (saved.BottomDrawerHeight is { } drawerHeight) BottomDrawerHeight = Math.Clamp(drawerHeight, 240, 720);
        }
        finally { _restoringUiState = false; }
    }

    internal UiProjectState RestoreRecentContentState()
    {
        if (SelectedProject is not { } project || _uiStateStore is null) return new UiProjectState();
        var saved = _uiStateStore.Project(project.ProjectId);
        _restoringUiState = true;
        try
        {
            RecentContents.Clear();
            foreach (var item in saved.RecentContent.Take(RecentContentLimit))
            {
                if (item.Kind == "session" && item.SessionId is { Length: > 0 } sessionId)
                {
                    var session = Sessions.FirstOrDefault(value => value.SessionId == sessionId);
                    if (session is not null) RecentContents.Add(RecentContentItem.FromSession(session));
                }
                else if (item.Kind == "file" && item.Path is { Length: > 0 } path && IsSafeRelativePath(path))
                {
                    RecentContents.Add(RecentContentItem.FromFile(new ExplorerNode(Path.GetFileName(path), path, "file", item.DependencyId)));
                }
            }
            NotifyRecentContentChanged();
        }
        finally { _restoringUiState = false; }
        return saved;
    }

    private static bool IsSafeRelativePath(string path) =>
        !Path.IsPathRooted(path) && !path.Contains('\0') && !path.Split('/', '\\').Any(part => part is "" or "." or "..");

    internal void SaveWindowGeometry(double x, double y, double width, double height, string state)
    {
        SaveProjectUiState(saved =>
        {
            saved.WindowX = x;
            saved.WindowY = y;
            saved.WindowWidth = width;
            saved.WindowHeight = height;
            saved.WindowState = state;
        });
    }

    internal void SavePanelGeometry()
    {
        SaveProjectUiState(saved =>
        {
            saved.NavigationWidth = NavigationPaneWidth;
            saved.ReviewWidth = ReviewPaneWidth;
            saved.BottomDrawerHeight = BottomDrawerHeight;
        });
    }

    internal void SaveRegionState()
    {
        SaveProjectUiState(saved =>
        {
            saved.LeftRegion = !NavigationVisible ? "closed" : ExplorerVisible ? "explorer" : "sessions";
            saved.RightRegion = ReviewVisible ? "review" : "closed";
            saved.BottomDrawer = GitVisible ? "git" : ProviderTraceVisible ? "providerTrace" : ToolActivityVisible ? "toolActivity" : "closed";
        });
    }
}
