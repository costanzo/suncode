using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Platform.Storage;
using Avalonia.VisualTree;
using SvgControl = Avalonia.Svg.Skia.Svg;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.ProjectWorkspace;

public sealed partial class WorkspaceWindow : Window
{
    private bool _initialized;
    private bool _windowDragActive;
    private bool _windowDragStarted;
    private PixelPoint _windowDragStartPointer;
    private PixelPoint _windowDragStartPosition;
    private bool _isFullScreen;
    private bool _isFullScreenTransition;
    private NativeMenuItem? _toggleNavigationMenuItem;
    private NativeMenu? _recentProjectsMenu;

    public WorkspaceWindow()
    {
        InitializeComponent();
        WindowDecorations = Avalonia.Controls.WindowDecorations.BorderOnly;
        AddHandler(KeyDownEvent, WindowKeyDown, RoutingStrategies.Tunnel);
        Icon = new WindowIcon(Avalonia.Platform.AssetLoader.Open(new Uri("avares://SunCode/Assets/logo/suncode-logo-128.png")));
        Opened += OnOpened;
        Closing += OnClosing;
        SizeChanged += (_, _) =>
        {
            ViewModel.UpdateLayoutWidth(Bounds.Width);
            ProjectWorkspaceView.ClampGitViewerHeight();
        };
        ConfigureNativeProjectMenu();
    }

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;

    private async void OnOpened(object? sender, EventArgs e)
    {
        if (_initialized) return;
        _initialized = true;
        ViewModel.ConversationChanged += ConversationChanged;
        await ViewModel.InitializeAsync();
        ConfigureProjectWindow();
        UpdateNativeProjectMenu();
        ProjectWorkspaceView.ScrollConversationToEnd();
    }

    private void OnClosing(object? sender, WindowClosingEventArgs e)
    {
        ViewModel.ConversationChanged -= ConversationChanged;
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

    internal void ShowSettings()
    {
        if (Application.Current is App app) app.ShowSettings(this);
    }

    internal void ShowArchiveConfirmation(SessionItem session)
    {
        if (Application.Current is App app)
            app.ShowArchiveConfirmation(this, session, () => _ = ViewModel.ArchiveSessionAsync(session));
    }

    private void ConfigureNativeProjectMenu()
    {
        var projectActions = new NativeMenu();

        var openProject = new NativeMenuItem { Header = "Open Project…" };
        openProject.Click += (_, _) => _ = OpenProjectPickerAsync();
        projectActions.Items.Add(openProject);

        var backToProjects = new NativeMenuItem { Header = "Back to Projects" };
        backToProjects.Click += (_, _) => Close();
        projectActions.Items.Add(backToProjects);

        _recentProjectsMenu = new NativeMenu();
        projectActions.Items.Add(new NativeMenuItem
        {
            Header = "Open Recent Project",
            Menu = _recentProjectsMenu
        });

        _toggleNavigationMenuItem = new NativeMenuItem { Header = "Hide Project Navigation" };
        _toggleNavigationMenuItem.Click += (_, _) => ViewModel.NavigationVisible = !ViewModel.NavigationVisible;
        projectActions.Items.Add(_toggleNavigationMenuItem);
        projectActions.Items.Add(new NativeMenuItemSeparator());

        var settings = new NativeMenuItem
        {
            Header = "Settings…",
            Gesture = new KeyGesture(Key.OemComma, OperatingSystem.IsMacOS() ? KeyModifiers.Meta : KeyModifiers.Control)
        };
        settings.Click += (_, _) => ShowSettings();
        projectActions.Items.Add(settings);

        var closeWindow = new NativeMenuItem { Header = "Close Window" };
        closeWindow.Click += (_, _) => Close();
        projectActions.Items.Add(closeWindow);

        var menu = new NativeMenu();
        menu.Items.Add(new NativeMenuItem { Header = "Project actions", Menu = projectActions });
        menu.NeedsUpdate += (_, _) => UpdateNativeProjectMenu();
        NativeMenu.SetMenu(this, menu);
    }

    private void UpdateNativeProjectMenu()
    {
        if (_toggleNavigationMenuItem is not null)
            _toggleNavigationMenuItem.Header = ViewModel.NavigationVisible ? "Hide Project Navigation" : "Show Project Navigation";
        if (_recentProjectsMenu is null) return;

        _recentProjectsMenu.Items.Clear();
        foreach (var project in ViewModel.Projects)
        {
            var recent = new NativeMenuItem
            {
                Header = string.IsNullOrWhiteSpace(project.DisplayName) ? project.CanonicalRoot : project.DisplayName,
                IsEnabled = Application.Current is not App app || !app.IsProjectOpen(project.ProjectId)
            };
            recent.Click += (_, _) =>
            {
                if (Application.Current is App current) _ = current.OpenProjectWindowAsync(project);
            };
            _recentProjectsMenu.Items.Add(recent);
        }
    }

    private void ConversationChanged() =>
        ProjectWorkspaceView.ScrollConversationToEnd();

    private void WindowKeyDown(object? sender, KeyEventArgs e)
    {
        if (e.Key == Key.Escape && ProjectWorkspaceView.HandleEscape())
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
            ViewModel.NavigationVisible = !ViewModel.NavigationVisible;
            return;
        }

        if (IsToggleGitViewerShortcut(e.Key, e.KeyModifiers))
        {
            e.Handled = true;
            ProjectWorkspaceView.ToggleGitViewer();
        }
    }

    internal static bool IsToggleNavigationShortcut(Key key, KeyModifiers modifiers) =>
        key == Key.D1 &&
        (modifiers.HasFlag(KeyModifiers.Meta) || modifiers.HasFlag(KeyModifiers.Control));

    internal static bool IsToggleGitViewerShortcut(Key key, KeyModifiers modifiers) =>
        key == Key.D9 &&
        (modifiers.HasFlag(KeyModifiers.Meta) || modifiers.HasFlag(KeyModifiers.Control));

    internal void TitleBarPressed(object? sender, PointerPressedEventArgs e)
    {
        if (sender is not Control region || !e.GetCurrentPoint(region).Properties.IsLeftButtonPressed ||
            OriginatesFromButton(e.Source) || _isFullScreen) return;
        _windowDragActive = true;
        _windowDragStarted = false;
        _windowDragStartPointer = Avalonia.VisualExtensions.PointToScreen(this, e.GetPosition(this));
        _windowDragStartPosition = Position;
        e.Pointer.Capture(region);
        e.Handled = true;
    }

    internal void TitleBarMoved(object? sender, PointerEventArgs e)
    {
        if (!_windowDragActive || sender is not Control region) return;
        var pointer = Avalonia.VisualExtensions.PointToScreen(this, e.GetPosition(this));
        var deltaX = pointer.X - _windowDragStartPointer.X;
        var deltaY = pointer.Y - _windowDragStartPointer.Y;
        if (!_windowDragStarted && Math.Sqrt(deltaX * deltaX + deltaY * deltaY) < 8) return;
        _windowDragStarted = true;
        Position = new PixelPoint(_windowDragStartPosition.X + deltaX, _windowDragStartPosition.Y + deltaY);
        e.Handled = true;
    }

    internal void TitleBarReleased(object? sender, PointerReleasedEventArgs e)
    {
        if (sender is not Control region) return;
        _windowDragActive = false;
        _windowDragStarted = false;
        e.Pointer.Capture(null);
        e.Handled = true;
    }

    internal void TitleBarDoubleTapped(object? sender, TappedEventArgs e)
    {
        if (OriginatesFromButton(e.Source)) return;
        if (UsesMaximizedStateForTitleBarDoubleTap(OperatingSystem.IsMacOS())) ToggleWindowMaximized();
        else ToggleFullScreen();
        e.Handled = true;
    }

    internal static bool UsesMaximizedStateForTitleBarDoubleTap(bool isMacOS) => isMacOS;

    private static bool OriginatesFromButton(object? source) =>
        source is Button || source is Visual visual && visual.FindAncestorOfType<Button>() is not null;

    internal void MinimizeWindow() => WindowState = WindowState.Minimized;

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

    internal static void SetTrafficLightState(object? sender, string state)
    {
        if (sender is not Button button || button.GetVisualDescendants().OfType<SvgControl>().FirstOrDefault() is not { } icon) return;
        var kind = button.Name?.Contains("Close", StringComparison.Ordinal) == true
            ? "close"
            : button.Name?.Contains("Minimize", StringComparison.Ordinal) == true ? "minimize" : "maximize";
        var file = (kind, state) switch
        {
            ("close", "hover") => "2-close-2-hover.svg",
            ("close", "press") => "2-close-3-press.svg",
            ("close", _) => "1-close-1-normal.svg",
            ("minimize", "hover") => "2-minimize-2-hover.svg",
            ("minimize", "press") => "2-minimize-3-press.svg",
            ("minimize", _) => "2-minimize-1-normal.svg",
            ("maximize", "hover") => "3-maximize-2-hover.svg",
            ("maximize", "press") => "3-maximize-3-press.svg",
            _ => "3-maximize-1-normal.svg"
        };
        icon.Path = $"/Assets/traffic-lights/{file}";
    }

    private void ConfigureProjectWindow()
    {
        ExtendClientAreaToDecorationsHint = true;
        Title = ViewModel.ProjectTitle;
        MinWidth = 620;
        MinHeight = 620;
        ResizeAndCenter(1440, 900);
        ViewModel.UpdateLayoutWidth(1440);
    }

    private void ResizeAndCenter(double width, double height)
    {
        Width = width;
        Height = height;
        if (Screens.Primary is not { } screen) return;
        var area = screen.WorkingArea;
        Position = new PixelPoint(
            area.X + Math.Max(0, (area.Width - (int)width) / 2),
            area.Y + Math.Max(0, (area.Height - (int)height) / 2));
    }
}
