using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Media;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.ProjectWorkspace;

public sealed partial class ProjectSwitcher : UserControl
{
    public event EventHandler? OpenProjectRequested;
    public event Action<ProjectItem>? ProjectRequested;

    public ProjectSwitcher()
    {
        InitializeComponent();
    }

    private async void ProjectFlyoutOpened(object? sender, EventArgs e)
    {
        SetChevronAngle(-90);
        if (DataContext is DesktopViewModel viewModel)
        {
            await viewModel.RefreshProjectsAsync();
        }
    }

    private void ProjectFlyoutClosed(object? sender, EventArgs e) => SetChevronAngle(90);

    private void OpenProject(object? sender, RoutedEventArgs e)
    {
        ProjectSwitcherButton.Flyout?.Hide();
        OpenProjectRequested?.Invoke(this, EventArgs.Empty);
    }

    private void OpenRecentProject(object? sender, RoutedEventArgs e)
    {
        if (sender is not Button { CommandParameter: ProjectItem project }) return;
        ProjectSwitcherButton.Flyout?.Hide();
        ProjectRequested?.Invoke(project);
    }

    private void SetChevronAngle(double angle)
    {
        if (ProjectSwitcherChevron.RenderTransform is RotateTransform rotation)
        {
            rotation.Angle = angle;
        }
        else
        {
            ProjectSwitcherChevron.RenderTransform = new RotateTransform(angle);
        }
    }
}
