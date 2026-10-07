using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Markup.Xaml;
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

    internal Task OpenProjectWindowAsync(ProjectItem project) =>
        _windows?.OpenProjectWindowAsync(project) ?? Task.CompletedTask;

    // Merging is synchronous. The Task shape is kept only for existing view callers.
    internal Task MergeAllProjectWindowsAsync()
    {
        _windows?.MergeAllProjectWindows();
        return Task.CompletedTask;
    }

    internal void ShowSettings(Window owner, Control? source = null) => _dialogs?.ShowSettings(owner, source);

    internal void ShowAbout(Window owner) => _dialogs?.ShowAbout(owner);

    internal void ShowArchiveConfirmation(Window owner, SessionItem session, Action confirm) =>
        _dialogs?.ShowArchiveConfirmation(owner, session, confirm);

    internal void ShowSessionConfirmation(
        Window owner,
        string title,
        string description,
        string target,
        Action confirm,
        string confirmLabel) =>
        _dialogs?.ShowSessionConfirmation(owner, title, description, target, confirm, confirmLabel);

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
            if (_viewModel is null || _windows is null || request.ProjectId is null || request.SessionId is null)
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
            if (ApplicationLifetime is not IClassicDesktopStyleApplicationLifetime desktop) return;
            var owner = desktop.Windows.FirstOrDefault(window => window.IsActive)
                ?? desktop.Windows.FirstOrDefault(window => window.IsVisible)
                ?? _hubWindow;
            if (owner is not null) ShowAbout(owner);
        };
        menu.Items.Add(about);
        var windowActions = new NativeMenu();
        var mergeWindows = new NativeMenuItem { Header = "Merge All Windows", IsEnabled = CanMergeWindows };
        mergeWindows.Click += (_, _) => _windows?.MergeAllProjectWindows();
        windowActions.Items.Add(mergeWindows);
        var windowMenu = new NativeMenuItem { Header = "Window", Menu = windowActions };
        windowMenu.Menu!.NeedsUpdate += (_, _) => mergeWindows.IsEnabled = CanMergeWindows;
        menu.Items.Add(windowMenu);
        NativeMenu.SetMenu(this, menu);
    }

    private void SetOtherWindowsEnabled(Window owner, bool enabled)
    {
        if (_hubWindow is not null && _hubWindow != owner) _hubWindow.IsEnabled = enabled;
        _windows?.SetEnabledExcept(owner, enabled);
    }
}
