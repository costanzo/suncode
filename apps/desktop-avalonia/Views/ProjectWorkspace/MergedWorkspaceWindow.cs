using Avalonia;
using Avalonia.Controls;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.ProjectWorkspace;

internal sealed class MergedWorkspaceWindow : WorkspaceWindow
{
    internal const double TearOffDistance = 28;
    private readonly MergedWorkspaceTabs _tabs = new();
    private readonly ContentControl _content = new();
    private readonly Dictionary<string, WorkspaceWindow> _projectWindows = [];
    private string? _activeProjectId;
    private bool _suppressCloseNotification;

    internal override bool IsMergedHost => true;
    internal event Action<string, PixelPoint>? ProjectTornOff;
    internal event Action<MergedWorkspaceWindow>? HostClosed;
    internal IReadOnlyList<string> ProjectIds => _tabs.Tabs.Select(item => item.ProjectId).ToArray();

    internal MergedWorkspaceWindow(DesktopViewModel initialViewModel) : base()
    {
        DataContext = initialViewModel;
        _tabs.ProjectSelected += SelectProject;
        _tabs.ProjectTornOff += (projectId, pointer) => ProjectTornOff?.Invoke(projectId, pointer);
        Content = _content;
        Closed += (_, _) =>
        {
            if (!_suppressCloseNotification) HostClosed?.Invoke(this);
        };
    }

    internal bool ContainsProject(string projectId) => _projectWindows.ContainsKey(projectId);

    internal void AddProject(string projectId, WorkspaceWindow source)
    {
        source.Workspace.DataContext = source.DataContext;
        source.Content = null;
        var title = source.DataContext is DesktopViewModel viewModel && !string.IsNullOrWhiteSpace(viewModel.ProjectTitle)
            ? viewModel.ProjectTitle
            : projectId;
        _projectWindows[projectId] = source;
        _tabs.Tabs.Add(new MergedProjectTab(projectId, title));
        UpdateMergedTitle();
        if (_tabs.Tabs.Count == 1) SelectProject(projectId);
    }

    internal void SelectProject(string projectId)
    {
        var selected = _tabs.Tabs.FirstOrDefault(item => item.ProjectId == projectId);
        if (selected is null || !_projectWindows.TryGetValue(projectId, out var source)) return;
        if (!ReferenceEquals(_content.Content, source.Workspace))
        {
            if (_content.Content is ProjectWorkspace previousWorkspace)
                previousWorkspace.SetMergedTabs(null);
            _content.Content = source.Workspace;
            source.Workspace.SetMergedTabs(_tabs);
        }
        _activeProjectId = selected.ProjectId;
        DataContext = source.DataContext;
        _tabs.Select(projectId);
        UpdateMergedTitle();
    }

    internal WorkspaceWindow? RemoveProject(string projectId)
    {
        var tab = _tabs.Tabs.FirstOrDefault(item => item.ProjectId == projectId);
        if (tab is null) return null;
        if (!_projectWindows.TryGetValue(projectId, out var source)) return null;
        if (_content.Content is ProjectWorkspace activeWorkspace && ReferenceEquals(activeWorkspace, source.Workspace))
        {
            activeWorkspace.SetMergedTabs(null);
            _content.Content = null;
        }
        _projectWindows.Remove(projectId);
        _tabs.Remove(projectId);
        UpdateMergedTitle();
        if (_tabs.Tabs.Count > 0) SelectProject(_tabs.Tabs[0].ProjectId);
        return source;
    }

    internal void CloseWithoutNotification()
    {
        _suppressCloseNotification = true;
        Close();
    }

    internal static bool ShouldTearOff(double deltaX, double deltaY) =>
        Math.Sqrt(deltaX * deltaX + deltaY * deltaY) >= TearOffDistance;

    private void UpdateMergedTitle()
    {
        var selected = _tabs.Tabs.FirstOrDefault(item => item.ProjectId == _activeProjectId)
            ?? _tabs.Tabs.FirstOrDefault();
        Title = selected is null ? "SunCode" : $"{selected.Title} ({_tabs.Tabs.Count})";
    }
}
