using Avalonia;
using Avalonia.Controls;
using Avalonia.Styling;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Views.ProjectWorkspace;

namespace SunCode.Desktop.Infrastructure;

/// <summary>
/// Owns project windows: opening (with de-duplication of concurrent opens), merging
/// into one tabbed host, tear-off, and close. UI-thread only.
/// </summary>
internal sealed class ProjectWindowManager
{
    private readonly ProjectWindowRegistry<WorkspaceWindow> _registry = new();
    private readonly Window _hubWindow;
    private readonly Func<IEnumerable<Window>> _allWindows;
    private readonly UiStateStore? _uiStateStore;
    private readonly AppSettingsViewModel? _appSettings;
    private MergedWorkspaceWindow? _mergedWindow;

    public ProjectWindowManager(
        Window hubWindow,
        Func<IEnumerable<Window>> allWindows,
        UiStateStore? uiStateStore,
        AppSettingsViewModel? appSettings)
    {
        _hubWindow = hubWindow;
        _allWindows = allWindows;
        _uiStateStore = uiStateStore;
        _appSettings = appSettings;
    }

    public bool CanMergeWindows => _registry.Count >= 2;

    public bool TryGetWindow(string projectId, out WorkspaceWindow window) => _registry.TryGet(projectId, out window);

    /// <summary>Completes when the project's window is registered. UI-thread only.</summary>
    public Task<WorkspaceWindow> WaitForWindowAsync(string projectId) => _registry.WaitForWindowAsync(projectId);

    public async Task OpenProjectWindowAsync(ProjectItem project)
    {
        var disposition = _registry.Resolve(project.ProjectId);
        if (disposition == ProjectWindowDisposition.ActivateExisting &&
            _registry.TryGet(project.ProjectId, out var existing))
        {
            if (_mergedWindow?.ContainsProject(project.ProjectId) == true)
            {
                _mergedWindow.SelectProject(project.ProjectId);
                _mergedWindow.Show();
                _mergedWindow.Activate();
                return;
            }
            existing.Show();
            existing.Activate();
            return;
        }
        if (disposition == ProjectWindowDisposition.AwaitOpening)
        {
            await _registry.WhenOpened(project.ProjectId);
            return;
        }
        if (!_registry.TryBeginOpening(project.ProjectId)) return;

        var viewModel = new DesktopViewModel(_uiStateStore, _appSettings);
        try
        {
            DiagnosticLog.Info("project.window", $"open begin project={project.ProjectId}");
            await viewModel.InitializeAsync();
            await viewModel.SelectProjectAsync(project);
            if (!viewModel.IsProjectOpen)
            {
                viewModel.Dispose();
                return;
            }

            var window = new WorkspaceWindow { DataContext = viewModel, OriginWindow = ResolveOriginWindow() };
            _registry.Register(project.ProjectId, window);
            window.Closed += (_, _) => ProjectWindowClosed(project.ProjectId, viewModel);
            _hubWindow.Hide();
            window.Show();
            window.Activate();
            DiagnosticLog.Info("project.window", $"open end project={project.ProjectId}");
        }
        catch (Exception exception)
        {
            DiagnosticLog.Error("project.window", exception, $"project={project.ProjectId}");
            viewModel.Dispose();
            throw;
        }
        finally
        {
            _registry.EndOpening(project.ProjectId);
        }
    }

    public void MergeAllProjectWindows()
    {
        if (_mergedWindow is { } existingMerged)
        {
            foreach (var (projectId, source) in _registry.Snapshot())
            {
                if (existingMerged.ContainsProject(projectId)) continue;
                existingMerged.AddProject(projectId, source);
                source.Hide();
            }
            existingMerged.Activate();
            return;
        }
        if (_registry.Count < 2) return;
        var sources = _registry.Snapshot();
        var first = sources[0].Value;
        if (first.DataContext is not DesktopViewModel firstViewModel) return;

        var merged = new MergedWorkspaceWindow(firstViewModel);
        merged.ProjectTornOff += TearOffProject;
        merged.ProjectCloseRequested += CloseMergedProject;
        merged.HostClosed += MergedHostClosed;
        _mergedWindow = merged;
        foreach (var (projectId, source) in sources)
        {
            merged.AddProject(projectId, source);
            source.Hide();
        }
        _hubWindow.Hide();
        merged.Show();
        merged.Activate();
    }

    /// <summary>
    /// Shows and activates the window hosting the project (the merged host if the
    /// project is a tab there), restoring it if minimized.
    /// </summary>
    public void RevealProject(string projectId, WorkspaceWindow window)
    {
        Window target = window;
        if (_mergedWindow?.ContainsProject(projectId) == true)
        {
            _mergedWindow.SelectProject(projectId);
            target = _mergedWindow;
        }
        target.Show();
        if (target.WindowState == WindowState.Minimized) target.WindowState = WindowState.Normal;
        target.Activate();
    }

    /// <summary>The visible merged host, else any visible project window, else null.</summary>
    public Window? FirstVisibleWindow() =>
        (Window?)(_mergedWindow?.IsVisible == true ? _mergedWindow : null)
        ?? _registry.Windows.FirstOrDefault(value => value.IsVisible);

    public void SetEnabledExcept(Window owner, bool enabled)
    {
        foreach (var window in _registry.Windows)
        {
            if (window != owner) window.IsEnabled = enabled;
        }
    }

    public void ApplyTheme(ThemeVariant variant)
    {
        foreach (var window in _registry.Windows) window.RequestedThemeVariant = variant;
        if (_mergedWindow is not null) _mergedWindow.RequestedThemeVariant = variant;
    }

    public void CloseAll()
    {
        foreach (var (_, window) in _registry.Snapshot()) window.Close();
    }

    private void CloseMergedProject(string projectId)
    {
        if (_mergedWindow is not { } merged) return;
        var source = merged.RemoveProject(projectId);
        if (source is null) return;
        source.Close();

        if (merged.ProjectIds.Count == 0)
        {
            _mergedWindow = null;
            merged.CloseWithoutNotification();
        }
        else if (merged.ProjectIds.Count == 1)
        {
            UnmergeRemainingProject(merged);
        }
    }

    private void TearOffProject(string projectId, PixelPoint pointer)
    {
        if (_mergedWindow is not { } merged) return;
        var source = merged.RemoveProject(projectId);
        if (source is null) return;
        source.Content = source.Workspace;
        source.OriginWindow = merged;
        source.Position = new PixelPoint(pointer.X - 240, pointer.Y - 18);
        source.RestoreAsProjectWindow();

        if (merged.ProjectIds.Count == 0)
        {
            _mergedWindow = null;
            merged.CloseWithoutNotification();
            return;
        }
        if (merged.ProjectIds.Count == 1) UnmergeRemainingProject(merged);
    }

    private void UnmergeRemainingProject(MergedWorkspaceWindow merged)
    {
        var remaining = merged.ProjectIds.ToArray();
        _mergedWindow = null;
        foreach (var projectId in remaining)
        {
            var source = merged.RemoveProject(projectId);
            if (source is null) continue;
            source.Content = source.Workspace;
            source.OriginWindow = merged;
            source.RestoreAsProjectWindow();
        }
        merged.CloseWithoutNotification();
    }

    private void MergedHostClosed(MergedWorkspaceWindow merged)
    {
        if (!ReferenceEquals(_mergedWindow, merged)) return;
        _mergedWindow = null;
        foreach (var projectId in merged.ProjectIds.ToArray())
        {
            merged.RemoveProject(projectId);
            if (_registry.TryGet(projectId, out var source)) source.Close();
        }
    }

    private void ProjectWindowClosed(string projectId, DesktopViewModel viewModel)
    {
        viewModel.Dispose();
        _registry.Remove(projectId);
        if (_registry.Count == 0)
        {
            _hubWindow.Show();
            _hubWindow.Activate();
        }
    }

    private Window ResolveOriginWindow()
    {
        var windows = _allWindows().ToArray();
        return windows.FirstOrDefault(window => window is WorkspaceWindow && window.IsActive && window.IsVisible)
            ?? windows.FirstOrDefault(window => window is WorkspaceWindow && window.IsVisible)
            ?? _hubWindow;
    }
}
