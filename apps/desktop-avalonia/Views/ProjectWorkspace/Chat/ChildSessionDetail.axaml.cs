using Avalonia.Controls;
using Avalonia.Interactivity;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.ProjectWorkspace.Chat;

public sealed partial class ChildSessionDetail : UserControl
{
    public ChildSessionDetail() => InitializeComponent();

    private async void ApproveOnce(object? sender, RoutedEventArgs e)
    {
        if (DataContext is DesktopViewModel viewModel)
            await viewModel.ResolveChildApprovalAsync("allow_once");
    }

    private async void DenyApproval(object? sender, RoutedEventArgs e)
    {
        if (DataContext is DesktopViewModel viewModel)
            await viewModel.ResolveChildApprovalAsync("deny");
    }

    private void BackToParent(object? sender, RoutedEventArgs e)
    {
        if (DataContext is DesktopViewModel viewModel) viewModel.ClearSelectedChildSession();
    }
}
