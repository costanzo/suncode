using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;

namespace SunCode.Desktop.ViewModels;

public sealed partial class DesktopViewModel
{
    private readonly UiStateStore? _uiStateStore;
    private bool _restoringUiState;
    internal bool RestoringSavedFile { get; set; }

    internal DesktopViewModel(UiStateStore? uiStateStore = null, AppSettingsViewModel? appSettings = null)
    {
        _uiStateStore = uiStateStore;
        AppSettings = appSettings ?? new AppSettingsViewModel();
        AppSettings.LanguageChanged += OnAppLanguageChanged;
        Mcp = new McpServersViewModel(this);
        LanguageServers = new LanguageServersViewModel(this);
        Network = new NetworkSettingsViewModel(this);
        Remote = new RemoteServerViewModel(this);
        Git = new GitReviewViewModel(this);
        ProviderTrace = new ProviderTraceViewModel(this);
        Editor = new EditorViewModel(this);
        Explorer = new ExplorerViewModel(this);
        Browser = new BrowserRuntimeViewModel(this);
        Computer = new ComputerRuntimeViewModel(this);
        Layout = new WorkspaceLayoutViewModel(this);
        Layout.RegionStateChanged += SaveRegionState;
        Layout.PanelGeometryChanged += SavePanelGeometry;
        Layout.PropertyChanged += OnLayoutPropertyChanged;
        ContextUsage = new ContextUsageViewModel(() => SelectedModel);
        _conversationDurationTimer.Tick += ConversationDurationTick;
        ComposerAttachments.CollectionChanged += (_, _) => OnPropertyChanged(nameof(ComposerBottomClearance));
        AttachSessionCollectionListeners();
    }

    internal UiProjectState SavedUiProjectState => SelectedProject is { } project
        ? _uiStateStore?.Project(project.ProjectId) ?? new UiProjectState()
        : new UiProjectState();

    internal void SaveSettingsNavigation(string page, string? providerId, bool providersExpanded, string? agentId, bool agentsExpanded)
    {
        _uiStateStore?.UpdateSettings(state =>
        {
            state.Page = page;
            state.ProviderId = providerId;
            state.ProvidersExpanded = providersExpanded;
            state.AgentId = agentId;
            state.AgentsExpanded = agentsExpanded;
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
        else if (SelectedChildSession is { } child)
        {
            state.CurrentContentKind = "child-session";
            state.CurrentSessionId = child.SessionId;
            state.CurrentDependencyId = null;
            state.CurrentFilePath = null;
            state.LastSessionId = child.ParentSessionId;
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
            Layout.NavigationVisible = saved.LeftRegion != "closed";
            Layout.ExplorerVisible = saved.LeftRegion == "explorer";
            Layout.ReviewVisible = saved.RightRegion == "review";
            Layout.ChildSessionsVisible = saved.RightRegion == "children";
            Layout.GitVisible = saved.BottomDrawer == "git";
            Layout.ProviderTraceVisible = saved.BottomDrawer == "providerTrace";
            Layout.ToolActivityVisible = saved.BottomDrawer == "toolActivity";
            if (saved.NavigationWidth is { } navigationWidth) Layout.NavigationPaneWidth = Math.Max(0, navigationWidth);
            if (saved.ReviewWidth is { } reviewWidth) Layout.ReviewPaneWidth = Math.Max(0, reviewWidth);
            if (saved.BottomDrawerHeight is { } drawerHeight) Layout.BottomDrawerHeight = Math.Max(0, drawerHeight);
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
            var restored = new List<RecentContentItem>();
            foreach (var item in saved.RecentContent.Take(RecentContentViewModel.Limit))
            {
                if (item.Kind == "session" && item.SessionId is { Length: > 0 } sessionId)
                {
                    var session = Sessions.FirstOrDefault(value => value.SessionId == sessionId);
                    if (session is not null) restored.Add(RecentContentItem.FromSession(session));
                }
                else if (item.Kind == "file" && item.Path is { Length: > 0 } path && IsSafeRelativePath(path))
                {
                    restored.Add(RecentContentItem.FromFile(new ExplorerNode(Path.GetFileName(path), path, "file", item.DependencyId)));
                }
            }
            RecentContent.ReplaceAll(restored);
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
            saved.NavigationWidth = Layout.NavigationPaneWidth;
            saved.ReviewWidth = Layout.ReviewPaneWidth;
            saved.BottomDrawerHeight = Layout.BottomDrawerHeight;
        });
    }

    internal void SaveRegionState()
    {
        SaveProjectUiState(saved =>
        {
            saved.LeftRegion = !Layout.NavigationVisible ? "closed" : Layout.ExplorerVisible ? "explorer" : "sessions";
            saved.RightRegion = Layout.ReviewVisible ? "review" : Layout.ChildSessionsVisible ? "children" : "closed";
            saved.BottomDrawer = Layout.GitVisible ? "git" : Layout.ProviderTraceVisible ? "providerTrace" : Layout.ToolActivityVisible ? "toolActivity" : "closed";
        });
    }

    // Gutter attention combines session state with the user's right-region
    // choice, so it follows the layout's review and child-session toggles.
    private void OnLayoutPropertyChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs e)
    {
        if (e.PropertyName is not (nameof(WorkspaceLayoutViewModel.ReviewVisible) or nameof(WorkspaceLayoutViewModel.ChildSessionsVisible))) return;
        OnPropertyChanged(nameof(ReviewGutterAttention));
        OnPropertyChanged(nameof(ChildSessionsGutterAttention));
    }
}
