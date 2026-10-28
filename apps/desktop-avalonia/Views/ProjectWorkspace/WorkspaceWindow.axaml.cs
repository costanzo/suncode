using Avalonia;
using Avalonia.Controls;
using Avalonia.Data.Converters;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Platform.Storage;
using Avalonia.Threading;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.ProjectWorkspace;

public partial class WorkspaceWindow : Window
{
    private readonly DispatcherTimer _mcpLoadTimer = new() { Interval = TimeSpan.FromMilliseconds(500) };
    private bool _initialized;
    private bool _isFullScreen;
    private bool _isFullScreenTransition;
    private DesktopMenuController? _desktopMenus;
    private bool _restoringWindowGeometry;
    private readonly ProjectWorkspace? _workspace;
    private readonly bool _adoptedWorkspace;
    
    internal Window? OriginWindow { get; set; }

    
    internal bool IsReleasedForMerge { get; set; }
    
    public WorkspaceWindow() : this(new ProjectWorkspace(), adoptedWorkspace: false)
    {
    }
    
    
    
    internal WorkspaceWindow(ProjectWorkspace? workspace, bool adoptedWorkspace)
    {
        InitializeComponent();
        _workspace = workspace;
        _adoptedWorkspace = adoptedWorkspace;
        if (workspace is not null) Content = workspace;
        WindowDecorations = Avalonia.Controls.WindowDecorations.BorderOnly;
        AddHandler(KeyDownEvent, WindowKeyDown, RoutingStrategies.Tunnel);
        Icon = AppIcon.Window;
        Opened += OnOpened;
        PropertyChanged += (_, change) =>
        {
            if (_initialized && change.Property == Window.WindowStateProperty)
                ActiveWorkspace?.UpdateWindowsMaximizeButton(WindowState);
        };
        Closing += OnClosing;
        Activated += async (_, _) =>
        {
            UpdateTrafficLightFocus(true);
            if (DataContext is DesktopViewModel model)
            {
                await model.Git.CheckAvailabilityAsync();
                await model.Git.RefreshOnActivationAsync();
            }
        };
        Deactivated += (_, _) =>
        {
            UpdateTrafficLightFocus(false);
            ActiveWorkspace?.CloseWindowsMenu();
        };
        _mcpLoadTimer.Tick += McpLoadTick;
        SizeChanged += (_, _) =>
        {
            ViewModel.Layout.UpdateLayoutSize(Bounds.Width, Bounds.Height);
            ActiveWorkspace?.ClampGitViewerHeight();
            SaveWindowGeometry();
        };
        PositionChanged += (_, _) => SaveWindowGeometry();
        ConfigureNativeProjectMenu();
    }

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;
    internal ProjectWorkspace Workspace =>
        _workspace ?? throw new InvalidOperationException("A merged host has no workspace of its own.");
    internal virtual ProjectWorkspace? ActiveWorkspace => _workspace;
    internal virtual bool IsMergedHost => false;

    
    internal void ReleaseForMerge()
    {
        IsReleasedForMerge = true;
        Close();
    }
    
    private async void OnOpened(object? sender, EventArgs e)
    {
        if (_initialized) return;
        _initialized = true;
        ActiveWorkspace?.UpdateWindowsMaximizeButton(WindowState);
        if (!IsMergedHost) ViewModel.SessionEntered += SessionEntered;
        if (ViewModel.IsProjectOpening)
        {
            RestoreWindowGeometry();
            ConfigureProjectWindow();
            UpdateTrafficLightFocus(IsActive);
            return;
        }
        await ViewModel.InitializeAsync();
        if (IsMergedHost || _adoptedWorkspace)
        {
            if (IsMergedHost) RestoreWindowGeometry();
            StartMcpLoadPollingIfNeeded();
            UpdateNativeProjectMenu();
            UpdateTrafficLightFocus(IsActive);
            return;
        }
        RestoreWindowGeometry();
        ConfigureProjectWindow();
        await CompleteProjectOpenAsync();
    }
    
    internal async Task CompleteProjectOpenAsync()
    {
        Title = ViewModel.ProjectTitle;
        UpdateNativeProjectMenu();
        if (ViewModel.SelectedSession != null) SessionEntered();
        UpdateTrafficLightFocus(IsActive);
        await ViewModel.Mcp.StartProjectAsync();
        StartMcpLoadPollingIfNeeded();
    }

    private void OnClosing(object? sender, WindowClosingEventArgs e)
    {
        
        if (!IsReleasedForMerge) _workspace?.ClosePreview();
        _mcpLoadTimer.Stop();
        if (!IsMergedHost) ViewModel.SessionEntered -= SessionEntered;
        // Panels have not been restored yet, so saving now would overwrite the project's layout.
        if (ViewModel.IsProjectOpening) return;
        SaveWindowGeometry();
        ViewModel.SavePanelGeometry();
        ViewModel.SaveRegionState();
    }

    internal void StartMcpLoadPollingIfNeeded()
    {
        if (ViewModel.Mcp.IsLoading) _mcpLoadTimer.Start();
    }
    
    private async void McpLoadTick(object? sender, EventArgs e)
    {
        await ViewModel.Mcp.RefreshLoadProgressAsync();
        if (!ViewModel.Mcp.IsLoading) _mcpLoadTimer.Stop();
    }

    private void RestoreWindowGeometry()
    {
        if (!ViewModel.IsProjectOpen && !ViewModel.IsProjectOpening) return;
        var saved = ViewModel.SavedUiProjectState;
        _restoringWindowGeometry = true;
        try
        {
            var width = saved.WindowWidth is { } savedWidth ? Math.Clamp(savedWidth, MinWidth, 2400) : Width;
            var height = saved.WindowHeight is { } savedHeight ? Math.Clamp(savedHeight, MinHeight, 1600) : Height;
            Width = width;
            Height = height;
            if (saved.WindowX is { } x && saved.WindowY is { } y)
            {
                var px = (int)Math.Round(x);
                var py = (int)Math.Round(y);
                PixelRect? area = Screens.ScreenFromPoint(new PixelPoint(px, py))?.WorkingArea;
                area ??= OriginWindow is { } origin ? Screens.ScreenFromWindow(origin)?.WorkingArea : null;
                area ??= Screens.Primary?.WorkingArea;
                if (area is { } workArea)
                {
                    px = Math.Clamp(px, workArea.X, Math.Max(workArea.X, workArea.Right - (int)Math.Round(width)));
                    py = Math.Clamp(py, workArea.Y, Math.Max(workArea.Y, workArea.Bottom - (int)Math.Round(height)));
                }
                Position = new PixelPoint(px, py);
            }
            WindowState = saved.WindowState switch
            {
                "maximized" => WindowState.Maximized,
                "fullscreen" => WindowState.FullScreen,
                _ => WindowState.Normal
            };
        }
        finally { _restoringWindowGeometry = false; }
    }

    private void SaveWindowGeometry()
    {
        if (_restoringWindowGeometry || !ViewModel.IsProjectOpen || WindowState != WindowState.Normal) return;
        ViewModel.SaveWindowGeometry(Position.X, Position.Y, Bounds.Width, Bounds.Height, "normal");
    }
    
    protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
    {
        base.OnPropertyChanged(change);
        if (change.Property != WindowStateProperty) return;
        var state = WindowState;
        _isFullScreen = state == WindowState.FullScreen;
        if (_restoringWindowGeometry || DataContext is not DesktopViewModel { IsProjectOpen: true } model) return;
        var saved = state switch
        {
            WindowState.Normal => "normal",
            WindowState.Maximized => "maximized",
            WindowState.FullScreen => "fullscreen",
            _ => null
        };
        if (saved is not null) model.SaveWindowState(saved);
    }

    internal async Task OpenProjectPickerAsync()
    {
        var folders = await StorageProvider.OpenFolderPickerAsync(new FolderPickerOpenOptions
        {
            Title = "Open a local project",
            AllowMultiple = false
        });
        var folder = folders.FirstOrDefault();
        var path = folder?.TryGetLocalPath();
        if (string.IsNullOrWhiteSpace(path) && folder?.Path is { IsFile: true } uri)
        {
            path = uri.LocalPath;
        }
        if (string.IsNullOrWhiteSpace(path)) return;
        if (Application.Current is App app) await app.OpenProjectPathAsync(path);
        else
        {
            await ViewModel.OpenProjectAsync(path);
            if (ViewModel.IsProjectOpen) ConfigureProjectWindow();
        }
    }

    internal async Task OpenProjectAsync(ProjectItem project)
    {
        if (Application.Current is App app) await app.OpenProjectWindowAsync(project);
        else
        {
            await ViewModel.SelectProjectAsync(project);
            ConfigureProjectWindow();
        }
    }

    internal void ShowSettings(Control? source = null)
    {
        if (Application.Current is App app) app.ShowSettings(this, source);
    }

    internal void ShowArchiveConfirmation(SessionItem session)
    {
        if (Application.Current is App app)
            app.ShowArchiveConfirmation(this, session, () => _ = ViewModel.ArchiveSessionAsync(session));
    }

    internal void ShowSessionConfirmation(SessionItem session, string title, string description, string confirmLabel, Action confirm, Size? size = null)
    {
        if (Application.Current is App app)
            app.ShowSessionConfirmation(this, title, description, session.DisplayTitle, confirm, confirmLabel, size);
    }

    internal void ShowConfirmation(
        string title,
        string description,
        string targetLabel,
        string target,
        string confirmLabel,
        Action confirm,
        string? note = null,
        Size? size = null)
    {
        if (Application.Current is App app)
            app.ShowConfirmation(this, title, description, targetLabel, target, confirmLabel, confirm, note, size);
    }
    
    internal void ShowSessionNameDialog(string title, string description, string value, string confirmLabel, Action<string> submit, Size? size = null)
    {
        if (Application.Current is App app)
            app.ShowSessionNameDialog(this, title, description, value, confirmLabel, submit, size);
    }
            

    private void ConfigureNativeProjectMenu()
    {
        _desktopMenus = new DesktopMenuController(
            () => this,
            () => DataContext as DesktopViewModel,
            () => ActiveWorkspace,
            OpenProjectPickerAsync);
        ActiveWorkspace?.SetWindowsMenuItems(_desktopMenus.BarGroups);
        NativeMenu.SetMenu(this, _desktopMenus.Menu);
    }

    private void UpdateNativeProjectMenu()
    {
        _desktopMenus?.Refresh();
        ActiveWorkspace?.SetWindowsMenuItems(_desktopMenus?.BarGroups ?? []);
    }

    internal void RefreshDesktopMenus() => UpdateNativeProjectMenu();

    private void SessionEntered() =>
        ActiveWorkspace?.ScrollConversationToEndForSessionEntry();

    private void WindowKeyDown(object? sender, KeyEventArgs e)
    {
        if (OperatingSystem.IsWindows() && e.Key == Key.F10)
        {
            e.Handled = ActiveWorkspace?.IsWindowsMenuExpanded == true
                ? ActiveWorkspace.CloseWindowsMenu(restoreFocus: true)
                : ToggleWindowsMenu();
            return;
        }

        if (e.Key == Key.Escape && ActiveWorkspace?.CloseWindowsMenu(restoreFocus: true) == true)
        {
            e.Handled = true;
            return;
        }

        if (e.Key == Key.Escape && ActiveWorkspace?.HandleEscape() == true)
        {
            e.Handled = true;
            return;
        }

        var commandModifier = e.KeyModifiers.HasFlag(KeyModifiers.Meta) || e.KeyModifiers.HasFlag(KeyModifiers.Control);
        if (commandModifier && e.Key == Key.OemComma)
        {
            e.Handled = true;
            ShowSettings();
            return;
        }

        if (IsToggleNavigationShortcut(e.Key, e.KeyModifiers))
        {
            e.Handled = true;
            ViewModel.Layout.NavigationVisible = !ViewModel.Layout.NavigationVisible;
            return;
        }

        if (ViewModel.Git.IsAvailable && IsToggleGitViewerShortcut(e.Key, e.KeyModifiers))
        {
            e.Handled = true;
            ActiveWorkspace?.ToggleGitViewer();
        }
    }

    private bool ToggleWindowsMenu()
    {
        if (ActiveWorkspace is not { } workspace) return false;
        workspace.ToggleWindowsMenu(this, new RoutedEventArgs());
        return true;
    }

    internal static bool IsToggleNavigationShortcut(Key key, KeyModifiers modifiers) =>
        key == Key.D1 &&
        (modifiers.HasFlag(KeyModifiers.Meta) || modifiers.HasFlag(KeyModifiers.Control));

    internal static bool IsToggleGitViewerShortcut(Key key, KeyModifiers modifiers) =>
        key == Key.D9 &&
        (modifiers.HasFlag(KeyModifiers.Meta) || modifiers.HasFlag(KeyModifiers.Control));

    public static readonly IValueConverter TitleBarRoleConverter = 
        new FuncValueConverter<WindowState, WindowDecorationsElementRole>(GetTitleBarRole);
    
    internal static WindowDecorationsElementRole GetTitleBarRole(WindowState state) =>
        state == WindowState.FullScreen ? WindowDecorationsElementRole.None : WindowDecorationsElementRole.TitleBar;
    
    protected virtual void UpdateTrafficLightFocus(bool isActive)
    {
        _workspace?.SetTrafficLightFocus(isActive);
    }

    internal void MinimizeWindow()
    {
        SaveWindowGeometry();
        WindowState = WindowState.Minimized;
    }

    internal void ToggleWindowMaximized()
    {
        if (_isFullScreen || _isFullScreenTransition) return;
        WindowState = GetTitleBarDoubleTapTargetState(WindowState);
    }

    internal static WindowState GetTitleBarDoubleTapTargetState(WindowState currentState) =>
        currentState == WindowState.Maximized ? WindowState.Normal : WindowState.Maximized;

    internal void ToggleFullScreen()
    {
        if (_isFullScreenTransition) return;
        if (_isFullScreen) _ = ExitFullScreenAsync();
        else _ = EnterFullScreenAsync();
    }

    private async Task EnterFullScreenAsync()
    {
        _isFullScreenTransition = true;
        _isFullScreen = true;
        WindowState = WindowState.FullScreen;
        await Task.Delay(900);
        _isFullScreenTransition = false;
    }

    private async Task ExitFullScreenAsync()
    {
        _isFullScreenTransition = true;
        _isFullScreen = false;
        WindowState = WindowState.Normal;
        await Task.Delay(900);
        _isFullScreenTransition = false;
    }

    private void ConfigureProjectWindow()
    {
        ExtendClientAreaToDecorationsHint = true;
        Title = ViewModel.ProjectTitle;
        MinWidth = 620;
        MinHeight = 620;
        ResizeAndCenter(1440, 900);
        ViewModel.Layout.UpdateLayoutWidth(1440);
    }

    internal void RestoreAsProjectWindow()
    {
        ConfigureProjectWindow();
        Show();
        Activate();
    }

    private void ResizeAndCenter(double width, double height)
    {
        Width = width;
        Height = height;
        var area = ResolveOriginWorkingArea();
        Position = new PixelPoint(
            area.X + Math.Max(0, (area.Width - (int)width) / 2),
            area.Y + Math.Max(0, (area.Height - (int)height) / 2));
    }
    
    private PixelRect ResolveOriginWorkingArea()
    {
        if (OriginWindow is { } origin&& Screens.ScreenFromWindow(origin) is { } originScreen)
            return originScreen.WorkingArea;
        if (Screens.Primary is { } primary) return primary.WorkingArea;
        return new PixelRect(Position, PixelSize.FromSize(Bounds.Size, 1));
    }
}
