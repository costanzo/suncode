using Avalonia.Controls;
using Avalonia.Interactivity;
using SunCode.Desktop.Models;

namespace SunCode.Desktop.Views.Projects;

public sealed partial class ProjectHub : UserControl
{
    public ProjectHub()
    {
        InitializeComponent();
    }

    private ProjectHubWindow? Owner => TopLevel.GetTopLevel(this) as ProjectHubWindow;

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
}
