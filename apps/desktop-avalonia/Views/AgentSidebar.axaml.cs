using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.VisualTree;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views;

public sealed partial class AgentSidebar : UserControl
{
    public AgentSidebar()
    {
        InitializeComponent();
    }

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;

    private async void ApproveOnce(object? sender, RoutedEventArgs e) =>
        await ViewModel.ResolveApprovalAsync("allow_once");

    private async void DenyApproval(object? sender, RoutedEventArgs e) =>
        await ViewModel.ResolveApprovalAsync("deny");

    private void RestoreCheckpoint(object? sender, RoutedEventArgs e)
    {
        if ((sender as Control)?.DataContext is CheckpointItem checkpoint)
            this.FindAncestorOfType<ProjectWorkspace>()?.ShowUndoDialog(checkpoint);
    }
}
