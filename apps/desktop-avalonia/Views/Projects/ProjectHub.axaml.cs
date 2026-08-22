using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using SunCode.Desktop.Models;
using SunCode.Desktop.Views.Shell;

namespace SunCode.Desktop.Views.Projects;

public sealed partial class ProjectHub : UserControl
{
    public ProjectHub()
    {
        InitializeComponent();
    }

    private MainWindow? Owner => TopLevel.GetTopLevel(this) as MainWindow;

    private async void OpenProject(object? sender, RoutedEventArgs e)
    {
        if (Owner is { } owner) await owner.OpenProjectPickerAsync();
    }

    private async void ProjectClicked(object? sender, RoutedEventArgs e)
    {
        if ((sender as Control)?.DataContext is ProjectItem project && Owner is { } owner)
            await owner.OpenProjectAsync(project);
    }

    private void OpenSettings(object? sender, RoutedEventArgs e) => Owner?.ShowSettings();
    private void CloseApplication(object? sender, RoutedEventArgs e) => Owner?.Close();
    private void MinimizeWindow(object? sender, RoutedEventArgs e) => Owner?.MinimizeWindow();
    private void ToggleHubMaximized(object? sender, RoutedEventArgs e) => Owner?.ToggleHubMaximized();
    private void TitleBarPressed(object? sender, PointerPressedEventArgs e) => Owner?.TitleBarPressed(sender, e);
    private void TitleBarMoved(object? sender, PointerEventArgs e) => Owner?.TitleBarMoved(sender, e);
    private void TitleBarReleased(object? sender, PointerReleasedEventArgs e) => Owner?.TitleBarReleased(sender, e);
    private void TitleBarDoubleTapped(object? sender, TappedEventArgs e) => Owner?.TitleBarDoubleTapped(sender, e);
    private void TrafficLightEntered(object? sender, PointerEventArgs e) => MainWindow.SetTrafficLightState(sender, "hover");
    private void TrafficLightExited(object? sender, PointerEventArgs e) => MainWindow.SetTrafficLightState(sender, "normal");
    private void TrafficLightPressed(object? sender, PointerPressedEventArgs e) => MainWindow.SetTrafficLightState(sender, "press");
    private void TrafficLightReleased(object? sender, PointerReleasedEventArgs e) => MainWindow.SetTrafficLightState(sender, "hover");
}
