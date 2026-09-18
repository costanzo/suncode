using Avalonia.Controls;
using Avalonia.Interactivity;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.ProjectWorkspace.Review;

public sealed partial class ChildSessionsPanel : UserControl
{
    public ChildSessionsPanel() => InitializeComponent();
    private async void SelectChildSession(object? sender, RoutedEventArgs e)
    {
        if (DataContext is DesktopViewModel viewModel && sender is Button { CommandParameter: ChildSessionItem child })
            await viewModel.SelectChildSessionAsync(child);
    }
}
