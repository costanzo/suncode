using System.Runtime.InteropServices;
using Avalonia;
using Avalonia.Automation;
using Avalonia.Controls;
using Avalonia.Controls.Chrome;
using Avalonia.Controls.Presenters;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Markup.Xaml.Styling;
using Avalonia.Media;
using Avalonia.Styling;
using Avalonia.Themes.Fluent;
using Avalonia.Threading;
using Avalonia.VisualTree;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Views.ProjectWorkspace;

namespace SunCode.Desktop.Tests;

[Collection("Avalonia UI")]
public sealed class WorkspaceTitleBarTests(AvaloniaUiFixture ui)
{
    [Theory]
    [InlineData("LINUX", "UbuntuTitleBarControls")]
    [InlineData("WINDOWS", "WindowsTitleBarControls")]
    [InlineData("OSX", "MacTitleBarControls")]
    public Task PlatformSelectsItsOwnWindowControls(string platform, string expected) => ui.Run(() => WithResources(() =>
    {
        var workspace = new ProjectWorkspace(OSPlatform.Create(platform));
        foreach (var name in new[] { "UbuntuTitleBarControls", "WindowsTitleBarControls", "MacTitleBarControls" })
            Assert.Equal(name == expected, workspace.FindControl<StackPanel>(name)!.IsVisible);
        Assert.Equal(platform == "OSX", workspace.FindControl<StackPanel>("MacTitleBarActions")!.IsVisible);
    }));

    [Theory]
    [InlineData(620, false)]
    [InlineData(620, true)]
    [InlineData(1440, false)]
    [InlineData(1440, true)]
    public Task UbuntuControlsKeepTheirTargetsAndDoNotOverlapTheTitle(int width, bool dark) => ui.Run(() => WithResources(() =>
    {
        var workspace = new ProjectWorkspace(OSPlatform.Linux);
        var titleBar = workspace.FindControl<Grid>("WorkspaceTitleBar")!;
        ((Grid)titleBar.Parent!).Children.Remove(titleBar);
        var content = workspace.FindControl<ContentSwitcher>("ContentSwitcherControl")!;
        var trigger = content.FindControl<Button>("ContentSwitcherButton")!;
        trigger.IsVisible = true;
        trigger.Content = new TextBlock { Text = new string('W', 200), TextTrimming = TextTrimming.CharacterEllipsis };
        var window = new Window { Content = titleBar, Width = width, Height = 100,
            RequestedThemeVariant = dark ? ThemeVariant.Dark : ThemeVariant.Light };
        try
        {
            window.Show(); Dispatcher.UIThread.RunJobs(); window.UpdateLayout();
            workspace.SetTrafficLightFocus(true);
            var buttons = titleBar.GetVisualDescendants().OfType<Button>()
                .Where(button => button.Classes.Contains("ubuntu-titlebar-control")).ToArray();
            Assert.Equal(3, buttons.Length);
            var titleBounds = content.TranslatePoint(default, titleBar)!.Value;
            foreach (var button in buttons)
            {
                Assert.Equal(new Size(32, 36), button.Bounds.Size);
                Assert.True(button.IsEffectivelyVisible);
                Assert.True(button.IsEffectivelyEnabled);
                var position = button.TranslatePoint(default, titleBar)!.Value;
                Assert.True(position.X >= titleBounds.X + content.Bounds.Width);
                Assert.True(position.X + button.Bounds.Width <= titleBar.Bounds.Width);
                var circle = button.GetVisualDescendants().OfType<Border>().Single(border => border.Name == "CaptionCircle");
                Assert.Equal(new Size(20, 20), circle.Bounds.Size);
                Assert.Equal(26, ((ISolidColorBrush)circle.Background!).Color.A);
            }
            var minimize = buttons[0];
            var origin = minimize.TranslatePoint(default, window)!.Value;
            var pointer = new Pointer(1, PointerType.Mouse, true);
            minimize.RaiseEvent(new PointerEventArgs(InputElement.PointerEnteredEvent, minimize, pointer, window, origin,
                0, new PointerPointProperties(), KeyModifiers.None));
            var hovered = minimize.GetVisualDescendants().OfType<Border>().Single(border => border.Name == "CaptionCircle");
            Assert.Equal(38, ((ISolidColorBrush)hovered.Background!).Color.A);
            var presenter = minimize.GetVisualDescendants().OfType<ContentPresenter>().Single();
            Assert.Equal(Colors.Transparent, ((ISolidColorBrush)presenter.Background!).Color);
            minimize.RaiseEvent(new PointerPressedEventArgs(minimize, pointer, window, origin, 0,
                new PointerPointProperties(RawInputModifiers.LeftMouseButton, PointerUpdateKind.LeftButtonPressed), KeyModifiers.None));
            Assert.Equal(64, ((ISolidColorBrush)hovered.Background!).Color.A);
            minimize.RaiseEvent(new PointerReleasedEventArgs(minimize, pointer, window, origin, 0,
                new PointerPointProperties(RawInputModifiers.None, PointerUpdateKind.LeftButtonReleased), KeyModifiers.None, MouseButton.Left));
            minimize.RaiseEvent(new PointerEventArgs(InputElement.PointerExitedEvent, minimize, pointer, window, origin,
                0, new PointerPointProperties(), KeyModifiers.None));
            buttons[1].Focus();
            Assert.True(minimize.Focus(NavigationMethod.Tab));
            var ring = minimize.GetVisualDescendants().OfType<Border>().Single(border => border.Name == "CaptionFocus");
            Assert.Equal((IBrush)window.FindResource(window.ActualThemeVariant, "AccentBrush")!, ring.BorderBrush);
            Assert.Equal(new Thickness(0), presenter.BorderThickness);
            workspace.SetTrafficLightFocus(false);
            Assert.Equal(Colors.Transparent, ((ISolidColorBrush)hovered.Background!).Color);
            Assert.Equal((IBrush)window.FindResource(window.ActualThemeVariant, "TextMutedBrush")!,
                minimize.GetVisualDescendants().OfType<Avalonia.Controls.Shapes.Path>().Single().Stroke);
            Assert.True(minimize.IsEffectivelyEnabled);
        }
        finally { window.Close(); }
    }));

    [Fact]
    public Task UbuntuActionsTargetTheOwningWindowAndRestoreLabelTracksLocale() => ui.Run(() => WithResources(() =>
    {
        using var model = new DesktopViewModel();
        model.BeginProjectOpening(new ProjectItem("chrome-test", "Chrome test", ""));
        var workspace = new ProjectWorkspace(OSPlatform.Linux) { DataContext = model };
        // The opening state mounts real window chrome without initializing the SDK.
        var owner = new WorkspaceWindow(workspace, adoptedWorkspace: true) { DataContext = model };
        var maximize = workspace.FindControl<Button>("UbuntuMaximizeButton")!;
        var minimize = workspace.FindControl<Button>("UbuntuMinimizeButton")!;
        var close = workspace.FindControl<Button>("UbuntuCloseButton")!;
        var closed = false;
        EventHandler<WindowClosingEventArgs> onClosing = (_, args) => { closed = true; args.Cancel = true; };
        try
        {
            owner.Show(); Dispatcher.UIThread.RunJobs();
            maximize.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
            Assert.Equal(WindowState.Maximized, owner.WindowState);
            Assert.Equal("Restore window", AutomationProperties.GetName(maximize));
            var localization = new LocalizationService(Application.Current!);
            localization.SetLocale("zh-CN");
            Assert.Equal("还原窗口", AutomationProperties.GetName(maximize));
            maximize.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
            Assert.Equal(WindowState.Normal, owner.WindowState);
            Assert.Equal("最大化窗口", AutomationProperties.GetName(maximize));
            minimize.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
            Assert.Equal(WindowState.Minimized, owner.WindowState);
            owner.Closing += onClosing;
            close.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
            Assert.True(closed);
            Assert.Equal(WindowDecorationsElementRole.User, WindowDecorationProperties.GetElementRole(workspace.FindControl<StackPanel>("UbuntuTitleBarControls")!));
        }
        finally { owner.Closing -= onClosing; owner.Close(); }
    }));

    [Fact]
    public Task ReparentedAndMergedProjectTabsFollowTheirHostMaximizeState() => ui.Run(() => WithResources(() =>
    {
        using var firstModel = new DesktopViewModel();
        using var secondModel = new DesktopViewModel();
        firstModel.BeginProjectOpening(new ProjectItem("first", "First", ""));
        secondModel.BeginProjectOpening(new ProjectItem("second", "Second", ""));
        var first = new ProjectWorkspace(OSPlatform.Linux) { DataContext = firstModel };
        var second = new ProjectWorkspace(OSPlatform.Linux) { DataContext = secondModel };
        var firstWindow = new WorkspaceWindow(first, true) { DataContext = firstModel };
        var secondWindow = new WorkspaceWindow(second, true) { DataContext = secondModel };
        var merged = new MergedWorkspaceWindow(firstModel);
        try
        {
            merged.AddProject("first", firstWindow);
            merged.AddProject("second", secondWindow);
            merged.Show(); Dispatcher.UIThread.RunJobs();
            merged.WindowState = WindowState.Maximized;
            merged.SelectProject("second");
            Assert.Equal("Restore window", AutomationProperties.GetName(second.FindControl<Button>("UbuntuMaximizeButton")!));
            merged.SelectProject("first");
            var maximize = first.FindControl<Button>("UbuntuMaximizeButton")!;
            Assert.Equal("Restore window", AutomationProperties.GetName(maximize));
            maximize.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
            Assert.Equal(WindowState.Normal, merged.WindowState);
            Assert.Equal(WindowState.Normal, firstWindow.WindowState);
            Assert.Equal("Maximize window", AutomationProperties.GetName(maximize));
        }
        finally
        {
            merged.RemoveProject("first"); merged.RemoveProject("second");
            merged.Close(); firstWindow.Close(); secondWindow.Close();
        }
    }));

    private static void WithResources(Action test)
    {
        var app = Application.Current!;
        var originalResources = app.Resources.MergedDictionaries.ToArray();
        var resources = new ResourceInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/ThemeResources.axaml") };
        var language = LocalizationService.CreateDictionary("en-US");
        var theme = new FluentTheme();
        var foundation = new StyleInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/FoundationStyles.axaml") };
        var styles = new StyleInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/ControlStyles.axaml") };
        app.Resources.MergedDictionaries.Add(resources); app.Resources.MergedDictionaries.Add(language);
        app.Styles.Add(theme); app.Styles.Add(foundation); app.Styles.Add(styles);
        try { test(); }
        finally
        {
            app.Styles.Remove(styles); app.Styles.Remove(foundation); app.Styles.Remove(theme);
            // LocalizationService can replace the language dictionary while a window is open.
            foreach (var item in app.Resources.MergedDictionaries.Except(originalResources).ToArray())
                app.Resources.MergedDictionaries.Remove(item);
        }
    }
}
