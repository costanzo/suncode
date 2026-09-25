using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Markup.Xaml;
using Avalonia.Styling;
using SunCode.Desktop.Models;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Views.About;
using SunCode.Desktop.Views.ProjectHub;
using SunCode.Desktop.Views.ProjectWorkspace;
using SunCode.Desktop.Views.Settings;
using SunCode.Desktop.Views.DialogWindow;

namespace SunCode.Desktop;

public sealed partial class App : Application
{
    private DesktopViewModel? _viewModel;
    private UiStateStore? _uiStateStore;
    private ProjectHubWindow? _hubWindow;
    private SettingsWindow? _settingsWindow;
    private AboutWindow? _aboutWindow;
    private SessionAttentionCoordinator? _attentionCoordinator;
    private MergedWorkspaceWindow? _mergedWindow;
    private readonly Dictionary<string, WorkspaceWindow> _projectWindows = [];
    private readonly HashSet<string> _openingProjects = [];
    private readonly Dictionary<string, TaskCompletionSource<bool>> _openingProjectSignals = [];
    private readonly SemaphoreSlim _activationGate = new(1, 1);
    internal bool CanMergeWindows => _projectWindows.Count >= 2;

    public override void Initialize()
    {
        DiagnosticLog.Info("app.initialize", "xaml_load begin");
        AvaloniaXamlLoader.Load(this);
        ConfigureNativeApplicationMenu();
        DiagnosticLog.Info("app.initialize", "xaml_load end");
#if DEBUG
        this.AttachDeveloperTools();
#endif
    }

    public override void OnFrameworkInitializationCompleted()
    {
        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
        {
            DiagnosticLog.Info("app.lifecycle", "framework_initialization begin");
            _uiStateStore = new UiStateStore();
            _viewModel = new DesktopViewModel(_uiStateStore);
            MacOSDockIcon.Apply();
            _viewModel.ThemeChanged += ApplyTheme;
            _hubWindow = new ProjectHubWindow { DataContext = _viewModel };
            if (Program.InstanceCoordinator is { } instance)
            {
                instance.ActivationReceived += OnActivationReceived;
                foreach (var request in instance.DrainPending()) OnActivationReceived(request);
            }
            desktop.ShutdownMode = ShutdownMode.OnLastWindowClose;
            _hubWindow.Show();
            _attentionCoordinator = new SessionAttentionCoordinator(
                IsApplicationForeground,
                ActivateAsync,
                SystemNotificationBackend.Create(),
                AppDataPaths.DataDirectory);
            _ = _attentionCoordinator.StartAsync();
            desktop.Exit += (_, _) =>
            {
                DiagnosticLog.Info("app.lifecycle", "exit begin");
                if (Program.InstanceCoordinator is { } instance) instance.ActivationReceived -= OnActivationReceived;
                _attentionCoordinator?.Dispose();
                _attentionCoordinator = null;
                _settingsWindow?.Close();
                _aboutWindow?.Close();
                foreach (var window in _projectWindows.Values.ToArray()) window.Close();
                _viewModel.Dispose();
                _uiStateStore?.Dispose();
                DiagnosticLog.Info("app.lifecycle", "exit end");
            };
        }

        base.OnFrameworkInitializationCompleted();
    }

    private void ApplyTheme(string mode)
    {
        var variant = mode == "light" ? ThemeVariant.Light : ThemeVariant.Dark;
        RequestedThemeVariant = variant;
        if (_hubWindow is not null) _hubWindow.RequestedThemeVariant = variant;
        if (_settingsWindow is not null) _settingsWindow.RequestedThemeVariant = variant;
        if (_aboutWindow is not null) _aboutWindow.RequestedThemeVariant = variant;
        foreach (var window in _projectWindows.Values)
        {
            window.RequestedThemeVariant = variant;
        }
        if (_mergedWindow is not null) _mergedWindow.RequestedThemeVariant = variant;
    }

    internal async Task OpenProjectPathAsync(string path)
    {
        if (_viewModel is null) return;
        var project = await _viewModel.RegisterProjectAsync(path);
        if (project is not null)
        {
            await OpenProjectWindowAsync(project);
        }
        else if (_hubWindow is not null)
        {
            _hubWindow.Show();
            _hubWindow.Activate();
        }
    }

    internal async Task OpenProjectWindowAsync(ProjectItem project)
    {
        var disposition = ResolveProjectWindowDisposition(
            _projectWindows.ContainsKey(project.ProjectId),
            _openingProjects.Contains(project.ProjectId));
        if (disposition == ProjectWindowDisposition.ActivateExisting &&
            _projectWindows.TryGetValue(project.ProjectId, out var existing))
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
            if (_openingProjectSignals.TryGetValue(project.ProjectId, out var signal)) await signal.Task;
            return;
        }
        if (!_openingProjects.Add(project.ProjectId)) return;
        var openingSignal = new TaskCompletionSource<bool>(TaskCreationOptions.RunContinuationsAsynchronously);
        _openingProjectSignals[project.ProjectId] = openingSignal;

        var viewModel = new DesktopViewModel(_uiStateStore);
        viewModel.ThemeChanged += ApplyTheme;
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

            var window = new WorkspaceWindow { DataContext = viewModel };
            _projectWindows[project.ProjectId] = window;
            window.Closed += (_, _) => ProjectWindowClosed(project.ProjectId, viewModel);
            _hubWindow?.Hide();
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
            _openingProjects.Remove(project.ProjectId);
            _openingProjectSignals.Remove(project.ProjectId);
            openingSignal.TrySetResult(_projectWindows.ContainsKey(project.ProjectId));
        }
    }

    internal Task MergeAllProjectWindowsAsync()
    {
        if (_mergedWindow is { } existingMerged)
        {
            foreach (var (projectId, source) in _projectWindows.ToArray())
            {
                if (existingMerged.ContainsProject(projectId)) continue;
                existingMerged.AddProject(projectId, source);
                source.Hide();
            }
            existingMerged.Activate();
            return Task.CompletedTask;
        }
        if (_projectWindows.Count < 2) return Task.CompletedTask;
        var sources = _projectWindows.ToArray();
        var first = sources[0].Value;
        if (first.DataContext is not DesktopViewModel firstViewModel) return Task.CompletedTask;

        var merged = new MergedWorkspaceWindow(firstViewModel);
        merged.ProjectTornOff += TearOffProject;
        merged.HostClosed += MergedHostClosed;
        _mergedWindow = merged;
        foreach (var (projectId, source) in sources)
        {
            merged.AddProject(projectId, source);
            source.Hide();
        }
        _hubWindow?.Hide();
        merged.Show();
        merged.Activate();
        return Task.CompletedTask;
    }

    private void TearOffProject(string projectId, PixelPoint pointer)
    {
        if (_mergedWindow is not { } merged) return;
        var source = merged.RemoveProject(projectId);
        if (source is null) return;
        source.Content = source.Workspace;
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
            if (_projectWindows.TryGetValue(projectId, out var source)) source.Close();
        }
    }

    private bool IsApplicationForeground()
    {
        if (ApplicationLifetime is not IClassicDesktopStyleApplicationLifetime desktop) return false;
        return NotificationForegroundEvaluator.IsForeground(desktop.Windows.Select(window => (
            window.IsActive,
            window.IsVisible,
            window.WindowState == WindowState.Minimized)));
    }

    private void OnActivationReceived(DesktopActivationRequest request) =>
        Avalonia.Threading.Dispatcher.UIThread.Post(async () =>
        {
            await _activationGate.WaitAsync();
            try { await ActivateAsync(request); }
            finally { _activationGate.Release(); }
        });

    private async Task ActivateAsync(DesktopActivationRequest request)
    {
        try
        {
            request.Validate();
            if (request.Kind == "activate.application")
            {
                ActivateFallbackWindow();
                return;
            }
            if (_viewModel is null || request.ProjectId is null || request.SessionId is null)
            {
                ActivateFallbackWindow();
                return;
            }
            await _viewModel.InitializeAsync();
            var project = _viewModel.Projects.FirstOrDefault(item => item.ProjectId == request.ProjectId);
            if (project is null)
            {
                ActivateFallbackWindow();
                return;
            }
            await OpenProjectWindowAsync(project);
            if (!_projectWindows.TryGetValue(project.ProjectId, out var window)
                || window.DataContext is not DesktopViewModel viewModel)
            {
                ActivateFallbackWindow();
                return;
            }
            var navigated = await viewModel.NavigateToSessionAsync(
                request.SessionId,
                request.ParentSessionId,
                request.ChildSessionId);
            if (!navigated) DiagnosticLog.Warn("notification.activation", "route_rejected=true reason=ownership_or_missing");
            if (_mergedWindow?.ContainsProject(project.ProjectId) == true)
            {
                _mergedWindow.SelectProject(project.ProjectId);
                _mergedWindow.Show();
                if (_mergedWindow.WindowState == WindowState.Minimized) _mergedWindow.WindowState = WindowState.Normal;
                _mergedWindow.Activate();
            }
            else
            {
                window.Show();
                if (window.WindowState == WindowState.Minimized) window.WindowState = WindowState.Normal;
                window.Activate();
            }
        }
        catch (Exception exception)
        {
            DiagnosticLog.Warn("notification.activation", $"route_failed=true error={exception.Message}");
            ActivateFallbackWindow();
        }
    }

    private void ActivateFallbackWindow()
    {
        var window = (Window?)(_mergedWindow?.IsVisible == true ? _mergedWindow : null)
            ?? (Window?)_projectWindows.Values.FirstOrDefault(value => value.IsVisible)
            ?? _hubWindow;
        if (window is null) return;
        window.Show();
        if (window.WindowState == WindowState.Minimized) window.WindowState = WindowState.Normal;
        window.Activate();
    }

    internal void ShowSettings(Window owner)
    {
        var viewModel = owner.DataContext as DesktopViewModel ?? _viewModel;
        if (viewModel is null) return;
        if (_settingsWindow is not null)
        {
            _settingsWindow.Activate();
            return;
        }

        _settingsWindow = new SettingsWindow { DataContext = viewModel };
        SetOtherWindowsEnabled(owner, false);
        _settingsWindow.Closed += (_, _) =>
        {
            SetOtherWindowsEnabled(owner, true);
            _settingsWindow = null;
        };
        _ = _settingsWindow.ShowDialog(owner);
    }

    internal void ShowAbout(Window owner)
    {
        if (_aboutWindow is not null)
        {
            _aboutWindow.Activate();
            return;
        }

        _aboutWindow = new AboutWindow();
        SetOtherWindowsEnabled(owner, false);
        _aboutWindow.Closed += (_, _) =>
        {
            SetOtherWindowsEnabled(owner, true);
            _aboutWindow = null;
        };
        _ = _aboutWindow.ShowDialog(owner);
    }

    internal void ShowArchiveConfirmation(Window owner, SessionItem session, Action confirm)
    {
        var dialog = new DialogWindow(
            "Archive this session?",
            "It will leave the active session list, but can be reopened later.",
            session.DisplayTitle,
            confirm);
        SetOtherWindowsEnabled(owner, false);
        dialog.Closed += (_, _) => SetOtherWindowsEnabled(owner, true);
        _ = dialog.ShowDialog(owner);
    }

    private void ConfigureNativeApplicationMenu()
    {
        var menu = new NativeMenu();
        var about = new NativeMenuItem { Header = $"About {AppInfo.ProductName}" };
        about.Click += (_, _) =>
        {
            if (ApplicationLifetime is not IClassicDesktopStyleApplicationLifetime desktop) return;
            var owner = desktop.Windows.FirstOrDefault(window => window.IsActive)
                ?? desktop.Windows.FirstOrDefault(window => window.IsVisible)
                ?? _hubWindow;
            if (owner is not null) ShowAbout(owner);
        };
        menu.Items.Add(about);
        var windowActions = new NativeMenu();
        var mergeWindows = new NativeMenuItem { Header = "Merge All Windows", IsEnabled = CanMergeWindows };
        mergeWindows.Click += (_, _) => _ = MergeAllProjectWindowsAsync();
        windowActions.Items.Add(mergeWindows);
        var windowMenu = new NativeMenuItem { Header = "Window", Menu = windowActions };
        windowMenu.Menu!.NeedsUpdate += (_, _) => mergeWindows.IsEnabled = CanMergeWindows;
        menu.Items.Add(windowMenu);
        NativeMenu.SetMenu(this, menu);
    }

    internal static ProjectWindowDisposition ResolveProjectWindowDisposition(bool isOpen, bool isOpening) =>
        isOpen
            ? ProjectWindowDisposition.ActivateExisting
            : isOpening ? ProjectWindowDisposition.AwaitOpening : ProjectWindowDisposition.OpenNew;

    private void SetOtherWindowsEnabled(Window owner, bool enabled)
    {
        if (_hubWindow is not null && _hubWindow != owner) _hubWindow.IsEnabled = enabled;
        foreach (var window in _projectWindows.Values)
        {
            if (window != owner) window.IsEnabled = enabled;
        }
    }

    private void ProjectWindowClosed(string projectId, DesktopViewModel viewModel)
    {
        viewModel.ThemeChanged -= ApplyTheme;
        viewModel.Dispose();
        _projectWindows.Remove(projectId);
        if (_projectWindows.Count == 0 && _hubWindow is not null)
        {
            _hubWindow.Show();
            _hubWindow.Activate();
        }
    }
}

internal enum ProjectWindowDisposition
{
    OpenNew,
    ActivateExisting,
    AwaitOpening
}
