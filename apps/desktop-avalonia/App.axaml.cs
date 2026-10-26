using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Markup.Xaml;
using Avalonia.Input;
using Avalonia.Styling;
using SunCode.Desktop.Models;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Views.ProjectHub;
using SunCode.Sdk;

namespace SunCode.Desktop;

public sealed partial class App : Application
{
    private DesktopViewModel? _viewModel;
    private ProjectHubViewModel? _hubViewModel;
    private AppSettingsViewModel? _appSettings;
    private UiStateStore? _uiStateStore;
    private ProjectHubWindow? _hubWindow;
    private ProjectWindowManager? _windows;
    private DesktopDockMenuController? _dockMenuController;
    private WindowsTaskbarJumpList? _windowsJumpList;
    private DialogService? _dialogs;
    private BrowserHostBridge? _browserHost;
    private SessionAttentionCoordinator? _attentionCoordinator;
    private readonly SemaphoreSlim _activationGate = new(1, 1);
    private LocalizationService? _localization;
    internal bool CanMergeWindows => _windows?.CanMergeWindows == true;

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
            _appSettings = new AppSettingsViewModel();
            _viewModel = new DesktopViewModel(_uiStateStore, _appSettings);
            _hubViewModel = new ProjectHubViewModel(_viewModel);
            _hubWindow = new ProjectHubWindow { DataContext = _hubViewModel };
            _windows = new ProjectWindowManager(_hubWindow, () => desktop.Windows, _uiStateStore, _appSettings);
            _dockMenuController = new DesktopDockMenuController(
                () => _viewModel.Projects.ToArray(),
                () => _windows.OpenProjects,
                OpenProjectWindowAsync,
                async () =>
                {
                    ShowProjectHub();
                    await _hubWindow.OpenProjectPickerAsync();
                },
                ShowProjectHub,
                () => _hubWindow.IsEnabled && desktop.Windows.All(window => window.IsEnabled));
            if (OperatingSystem.IsMacOS()) NativeDock.SetMenu(this, _dockMenuController.Menu);
            if (OperatingSystem.IsWindows() && WindowsTaskbarJumpList.IsEnabled)
            {
                _windowsJumpList = new WindowsTaskbarJumpList(() => _viewModel.Projects.ToArray());
                _ = RefreshWindowsJumpListAsync();
            }
            _dialogs = new DialogService(() => _viewModel, SetOtherWindowsEnabled);
            _browserHost = new BrowserHostBridge(new ProjectWindowBrowserHost(
                _windows,
                projectId => _viewModel?.Projects.FirstOrDefault(item => item.ProjectId == projectId)));
            AgentSdk.BrowserHostRequested += _browserHost.EnsureBrowserHost;
            _localization = new LocalizationService(this);
            _appSettings.LanguageChanged += ApplyLanguage;
            MacOSDockIcon.Apply();
            _appSettings.ThemeChanged += ApplyTheme;
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
                if (_browserHost is not null) AgentSdk.BrowserHostRequested -= _browserHost.EnsureBrowserHost;
                _dialogs?.CloseAll();
                _windows?.CloseAll();
                _hubViewModel.Dispose();
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
        _dialogs?.ApplyTheme(variant);
        _windows?.ApplyTheme(variant);
    }

    private void ApplyLanguage(string locale)
    {
        _localization?.SetLocale(locale);
        _ = _windowsJumpList?.RefreshAsync();
    }

    internal async Task OpenProjectPathAsync(string path)
    {
        if (_viewModel is null) return;
        var project = await _viewModel.RegisterProjectAsync(path);
        if (project is not null)
        {
            await OpenProjectWindowAsync(project);
            _ = _windowsJumpList?.RefreshAsync();
        }
        else if (_hubWindow is not null)
        {
            _hubWindow.Show();
            _hubWindow.Activate();
        }
    }

    internal async Task OpenProjectWindowAsync(ProjectItem project)
    {
        if (_windows is null) return;
        await _windows.OpenProjectWindowAsync(project);
        if (_viewModel is not null) await _viewModel.RefreshProjectsAsync();
        _ = _windowsJumpList?.RefreshAsync();
    }

    private async Task RefreshWindowsJumpListAsync()
    {
        if (_viewModel is null || _windowsJumpList is null) return;
        await _viewModel.InitializeAsync();
        await _windowsJumpList.RefreshAsync();
    }

    // Merging is synchronous. The Task shape is kept only for existing view callers.
    internal Task MergeAllProjectWindowsAsync()
    {
        _windows?.MergeAllProjectWindows();
        return Task.CompletedTask;
    }

    internal void ShowSettings(Window owner, Control? source = null, SettingsPage? page = null) => _dialogs?.ShowSettings(owner, source, page);

    internal void ShowProjectHub()
    {
        if (_hubWindow is null) return;
        _hubWindow.Show();
        if (_hubWindow.WindowState == WindowState.Minimized) _hubWindow.WindowState = WindowState.Normal;
        _hubWindow.Activate();
    }

    internal void ShowAbout(Window owner) => _dialogs?.ShowAbout(owner);

    internal void ShowArchiveConfirmation(Window owner, SessionItem session, Action confirm) =>
        _dialogs?.ShowArchiveConfirmation(owner, session, confirm);

    internal void ShowSessionConfirmation(
        Window owner,
        string title,
        string description,
        string target,
        Action confirm,
        string confirmLabel,
        Size? size = null) =>
        _dialogs?.ShowSessionConfirmation(owner, title, description, target, confirm, confirmLabel, size);
    
    internal void ShowConfirmation(
        Window owner,
        string title,
        string description,
        string targetLabel,
        string target,
        string confirmLabel,
        Action confirm,
        string? note = null,
        Size? size = null) =>
        _dialogs?.ShowConfirmation(owner, title, description, targetLabel, target, confirmLabel, confirm, note, size);
    
    internal void ShowSessionNameDialog(
        Window owner,
        string title,
        string description,
        string value,
        string confirmLabel,
        Action<string> submit,
        Size? size = null) =>
        _dialogs?.ShowSessionNameDialog(owner, title, description, value, confirmLabel, submit, size);
            

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
            if (_viewModel is null || _windows is null)
            {
                ActivateFallbackWindow();
                return;
            }
            await _viewModel.InitializeAsync();
            if (request.Kind == "activate.project")
            {
                var jumpProject = _viewModel.Projects.FirstOrDefault(item =>
                    string.Equals(
                        PathNormalization.FullPath(item.CanonicalRoot),
                        PathNormalization.FullPath(request.ProjectPath!),
                        OperatingSystem.IsWindows() ? StringComparison.OrdinalIgnoreCase : StringComparison.Ordinal));
                if (jumpProject is null && Directory.Exists(request.ProjectPath))
                    jumpProject = await _viewModel.RegisterProjectAsync(request.ProjectPath);
                if (jumpProject is null)
                {
                    ActivateFallbackWindow();
                    return;
                }
                await OpenProjectWindowAsync(jumpProject);
                _ = _windowsJumpList?.RefreshAsync();
                return;
            }
            if (request.ProjectId is null || request.SessionId is null)
            {
                ActivateFallbackWindow();
                return;
            }
            var project = _viewModel.Projects.FirstOrDefault(item => item.ProjectId == request.ProjectId);
            if (project is null)
            {
                ActivateFallbackWindow();
                return;
            }
            await OpenProjectWindowAsync(project);
            if (!_windows.TryGetWindow(project.ProjectId, out var window)
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
            _windows.RevealProject(project.ProjectId, window);
        }
        catch (Exception exception)
        {
            DiagnosticLog.Warn("notification.activation", $"route_failed=true error={exception.Message}");
            ActivateFallbackWindow();
        }
    }

    private void ActivateFallbackWindow()
    {
        var window = _windows?.FirstVisibleWindow() ?? _hubWindow;
        if (window is null) return;
        window.Show();
        if (window.WindowState == WindowState.Minimized) window.WindowState = WindowState.Normal;
        window.Activate();
    }

    private void ConfigureNativeApplicationMenu()
    {
        var menu = new NativeMenu();
        var about = new NativeMenuItem { Header = $"About {AppInfo.ProductName}" };
        about.Click += (_, _) =>
        {
            if (ResolveMenuOwner() is { } owner) ShowAbout(owner);
        };
        menu.Items.Add(about);
        menu.Items.Add(new NativeMenuItemSeparator());
        var settings = new NativeMenuItem { Header = "Settings…", Gesture = new KeyGesture(Key.OemComma, KeyModifiers.Meta) };
        settings.Click += (_, _) =>
        {
            if (ResolveMenuOwner() is { } owner) ShowSettings(owner);
        };
        menu.Items.Add(settings);
        menu.NeedsUpdate += (_, _) =>
        {
            about.Header = LocalizationService.GetString("LocAboutSunCode", $"About {AppInfo.ProductName}");
            settings.Header = LocalizationService.GetString("LocMenuSettings", "Settings…");
            settings.IsEnabled = ResolveMenuOwner() is not null;
            about.IsEnabled = ResolveMenuOwner() is not null;
        };
        NativeMenu.SetMenu(this, menu);
    }

    private Window? ResolveMenuOwner()
    {
        if (ApplicationLifetime is not IClassicDesktopStyleApplicationLifetime desktop) return null;
        var active = desktop.Windows.FirstOrDefault(window => window.IsActive && window.IsVisible);
        return active is { IsEnabled: true } &&
               (active is ProjectHubWindow or SunCode.Desktop.Views.ProjectWorkspace.WorkspaceWindow)
            ? active : null;
    }

    private void SetOtherWindowsEnabled(Window owner, bool enabled)
    {
        if (_hubWindow is not null && _hubWindow != owner) _hubWindow.IsEnabled = enabled;
        _windows?.SetEnabledExcept(owner, enabled);
    }
}
