using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.VisualTree;
using SvgControl = Avalonia.Svg.Skia.Svg;
using SunCode.Desktop.Models;

namespace SunCode.Desktop.Views.About;

public sealed partial class AboutWindow : Window
{
    public AboutWindow()
    {
        InitializeComponent();
        WindowDecorations = OperatingSystem.IsMacOS()
            ? Avalonia.Controls.WindowDecorations.BorderOnly
            : Avalonia.Controls.WindowDecorations.None;
        Icon = new WindowIcon(Avalonia.Platform.AssetLoader.Open(new Uri("avares://SunCode/Assets/logo/suncode-logo-128.png")));
        VersionText.Text = AppInfo.DisplayVersion;
    }

    private void CloseAbout(object? sender, RoutedEventArgs e) => Close();
    private void MinimizeAbout(object? sender, RoutedEventArgs e) => WindowState = WindowState.Minimized;
    private void ToggleAboutMaximized(object? sender, RoutedEventArgs e) =>
        WindowState = WindowState == WindowState.Maximized ? WindowState.Normal : WindowState.Maximized;

    private void TitleBarPressed(object? sender, PointerPressedEventArgs e)
    {
        if (e.GetCurrentPoint(this).Properties.IsLeftButtonPressed && !OriginatesFromButton(e.Source)) BeginMoveDrag(e);
    }

    private void TitleBarDoubleTapped(object? sender, TappedEventArgs e)
    {
        if (OriginatesFromButton(e.Source)) return;
        ToggleAboutMaximized(sender, new RoutedEventArgs());
        e.Handled = true;
    }

    private static bool OriginatesFromButton(object? source) =>
        source is Button || source is Avalonia.Visual visual && visual.FindAncestorOfType<Button>() is not null;

    private void TrafficLightEntered(object? sender, PointerEventArgs e) => SetTrafficLightState(sender, "hover");
    private void TrafficLightExited(object? sender, PointerEventArgs e) => SetTrafficLightState(sender, "normal");
    private void TrafficLightPressed(object? sender, PointerPressedEventArgs e) => SetTrafficLightState(sender, "press");
    private void TrafficLightReleased(object? sender, PointerReleasedEventArgs e) => SetTrafficLightState(sender, "hover");

    private static void SetTrafficLightState(object? sender, string state)
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
}
