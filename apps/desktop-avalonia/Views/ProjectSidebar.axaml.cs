using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.VisualTree;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views;

public sealed partial class ProjectSidebar : UserControl
{
    public ProjectSidebar()
    {
        InitializeComponent();
    }

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;
    private ProjectWorkspace? Workspace => this.FindAncestorOfType<ProjectWorkspace>();

    private async void SessionClicked(object? sender, RoutedEventArgs e)
    {
        if ((sender as Control)?.DataContext is SessionItem session)
            await ViewModel.SelectSessionAsync(session);
    }

    private void CreateSession(object? sender, RoutedEventArgs e) =>
        Workspace?.ShowSessionDialog("New session", string.Empty, "Create", null);

    private void RenameSessionItem(object? sender, RoutedEventArgs e)
    {
        if (sender is MenuItem { CommandParameter: SessionItem session })
            Workspace?.ShowSessionDialog("Rename session", session.DisplayTitle, "Save", session);
    }

    private async void ArchiveSessionItem(object? sender, RoutedEventArgs e)
    {
        if (sender is MenuItem { CommandParameter: SessionItem session })
            await ViewModel.ArchiveSessionAsync(session);
    }

    private async void NavigationPointerExited(object? sender, PointerEventArgs e)
    {
        if (ViewModel.NavigationPinned) return;
        await Task.Delay(420);
        if (!ViewModel.NavigationPinned) ViewModel.NavigationVisible = false;
    }
}
