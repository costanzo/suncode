using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Media;
using Avalonia.Platform.Storage;
using Avalonia.VisualTree;
using SvgControl = Avalonia.Svg.Skia.Svg;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.Shell;

public sealed partial class MainWindow : Window
{
    private readonly bool _isHubWindow;
    private bool _initialized;
    private bool _windowResizeActive;
    private WindowEdge _windowResizeEdge;
    private PixelPoint _windowResizeStartPointer;
    private PixelPoint _windowResizeStartPosition;
    private Size _windowResizeStartSize;
    private bool _windowDragActive;
    private bool _windowDragStarted;
    private PixelPoint _windowDragStartPointer;
    private PixelPoint _windowDragStartPosition;
    private bool _isFullScreen;
    private bool _isFullScreenTransition;
    private NativeMenuItem? _toggleNavigationMenuItem;
    private NativeMenu? _recentProjectsMenu;

    public MainWindow() : this(true)
    {
    }

    internal MainWindow(bool isHubWindow)
    {
        _isHubWindow = isHubWindow;
        InitializeComponent();
        WindowDecorations = OperatingSystem.IsMacOS()
            ? Avalonia.Controls.WindowDecorations.BorderOnly
            : Avalonia.Controls.WindowDecorations.None;
        AddHandler(KeyDownEvent, WindowKeyDown, RoutingStrategies.Tunnel);
        Icon = new WindowIcon(Avalonia.Platform.AssetLoader.Open(new Uri("avares://SunCode/Assets/logo/suncode-logo-128.png")));
        Opened += OnOpened;
        Closing += OnClosing;
        SizeChanged += (_, _) =>
        {
            ViewModel.UpdateLayoutWidth(Bounds.Width);
            ProjectWorkspaceView.ClampGitViewerHeight();
        };
        if (!_isHubWindow) ConfigureNativeProjectMenu();
    }

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;

    private async void OnOpened(object? sender, EventArgs e)
    {
        if (_initialized) return;
        _initialized = true;
        ViewModel.ConversationChanged += ConversationChanged;
        await ViewModel.InitializeAsync();
        if (_isHubWindow) ConfigureHubWindow();
        else
        {
            ConfigureProjectWindow();
            UpdateNativeProjectMenu();
            ProjectWorkspaceView.ScrollConversationToEnd();
        }
    }

    private void OnClosing(object? sender, WindowClosingEventArgs e)
    {
        ViewModel.ConversationChanged -= ConversationChanged;
        if (_isHubWindow) ViewModel.Dispose();
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

        if (IsToggleNavigationShortcut(e.Key, e.KeyModifiers, ViewModel.IsProjectOpen))
        {
            e.Handled = true;
            ViewModel.NavigationVisible = !ViewModel.NavigationVisible;
            return;
        }

        if (IsToggleGitViewerShortcut(e.Key, e.KeyModifiers, ViewModel.IsProjectOpen))
        {
            e.Handled = true;
            ProjectWorkspaceView.ToggleGitViewer();
        }
    }

    internal static bool IsToggleNavigationShortcut(Key key, KeyModifiers modifiers, bool isProjectOpen) =>
        isProjectOpen && key == Key.D1 &&
        (modifiers.HasFlag(KeyModifiers.Meta) || modifiers.HasFlag(KeyModifiers.Control));

    internal static bool IsToggleGitViewerShortcut(Key key, KeyModifiers modifiers, bool isProjectOpen) =>
        isProjectOpen && key == Key.D9 &&
        (modifiers.HasFlag(KeyModifiers.Meta) || modifiers.HasFlag(KeyModifiers.Control));

    private void WindowResizePressed(object? sender, PointerPressedEventArgs e)
    {
        if (sender is not Control { Tag: string edgeName } control ||
            !e.GetCurrentPoint(control).Properties.IsLeftButtonPressed || _isFullScreen) return;
        _windowResizeEdge = edgeName switch
        {
            "North" => WindowEdge.North,
            "South" => WindowEdge.South,
            "West" => WindowEdge.West,
            "East" => WindowEdge.East,
            "NorthWest" => WindowEdge.NorthWest,
            "NorthEast" => WindowEdge.NorthEast,
            "SouthWest" => WindowEdge.SouthWest,
            "SouthEast" => WindowEdge.SouthEast,
            _ => throw new InvalidOperationException($"Unknown window edge {edgeName}")
        };
        _windowResizeActive = true;
        _windowResizeStartPointer = Avalonia.VisualExtensions.PointToScreen(this, e.GetPosition(this));
        _windowResizeStartPosition = Position;
        _windowResizeStartSize = Bounds.Size;
        e.Pointer.Capture(control);
        e.Handled = true;
    }

    private void WindowResizeMoved(object? sender, PointerEventArgs e)
    {
        if (!_windowResizeActive) return;
        var pointer = Avalonia.VisualExtensions.PointToScreen(this, e.GetPosition(this));
        const double coordinateScale = 1;
        var deltaX = pointer.X - _windowResizeStartPointer.X;
        var deltaY = pointer.Y - _windowResizeStartPointer.Y;
        var west = _windowResizeEdge is WindowEdge.West or WindowEdge.NorthWest or WindowEdge.SouthWest;
        var east = _windowResizeEdge is WindowEdge.East or WindowEdge.NorthEast or WindowEdge.SouthEast;
        var north = _windowResizeEdge is WindowEdge.North or WindowEdge.NorthWest or WindowEdge.NorthEast;
        var south = _windowResizeEdge is WindowEdge.South or WindowEdge.SouthWest or WindowEdge.SouthEast;
        var width = Math.Max(MinWidth, _windowResizeStartSize.Width + (east ? deltaX : west ? -deltaX : 0));
        var height = Math.Max(MinHeight, _windowResizeStartSize.Height + (south ? deltaY : north ? -deltaY : 0));
        Width = width;
        Height = height;
        Position = new PixelPoint(
            west ? _windowResizeStartPosition.X + (int)Math.Round((_windowResizeStartSize.Width - width) * coordinateScale) : _windowResizeStartPosition.X,
            north ? _windowResizeStartPosition.Y + (int)Math.Round((_windowResizeStartSize.Height - height) * coordinateScale) : _windowResizeStartPosition.Y);
        e.Handled = true;
    }

    private void WindowResizeReleased(object? sender, PointerReleasedEventArgs e)
    {
        if (!_windowResizeActive) return;
        _windowResizeActive = false;
        e.Pointer.Capture(null);
        e.Handled = true;
    }

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
        ToggleMaximized();
        e.Handled = true;
    }

    private static bool OriginatesFromButton(object? source) =>
        source is Button || source is Visual visual && visual.FindAncestorOfType<Button>() is not null;

    internal void MinimizeWindow() => WindowState = WindowState.Minimized;

    internal void ToggleHubMaximized() =>
        WindowState = WindowState == WindowState.Maximized ? WindowState.Normal : WindowState.Maximized;

    internal void ToggleMaximized()
    {
        if (_isFullScreenTransition) return;
        if (_isFullScreen) _ = ExitFullScreenAsync();
        else _ = EnterFullScreenAsync();
    }

    private async Task EnterFullScreenAsync()
    {
        _isFullScreenTransition = true;
        _isFullScreen = true;
        Background = this.FindResource("CanvasBrush") as IBrush;
        ProjectWorkspaceView.SetFullScreenChrome(true);
        SetResizeHandlesVisible(false);
        WindowState = WindowState.FullScreen;
        await Task.Delay(900);
        _isFullScreenTransition = false;
    }

    private async Task ExitFullScreenAsync()
    {
        _isFullScreenTransition = true;
        _isFullScreen = false;
        ProjectWorkspaceView.SetFullScreenChrome(false);
        SetResizeHandlesVisible(true);
        WindowState = WindowState.Normal;
        await Task.Delay(900);
        Background = Brushes.Transparent;
        _isFullScreenTransition = false;
    }

    private void SetResizeHandlesVisible(bool visible)
    {
        ResizeNorth.IsVisible = visible;
        ResizeSouth.IsVisible = visible;
        ResizeWest.IsVisible = visible;
        ResizeEast.IsVisible = visible;
        ResizeNorthWest.IsVisible = visible;
        ResizeNorthEast.IsVisible = visible;
        ResizeSouthWest.IsVisible = visible;
        ResizeSouthEast.IsVisible = visible;
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

    private void ConfigureHubWindow()
    {
        WindowState = WindowState.Normal;
        ExtendClientAreaToDecorationsHint = true;
        Title = "Welcome to SunCode";
        MinWidth = 760;
        MinHeight = 552;
        ResizeAndCenter(980, 712);
        ViewModel.UpdateLayoutWidth(980);
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
