using Avalonia.Controls;
using Avalonia.Interactivity;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.ProjectWorkspace.Review;

public sealed partial class TurnDiffViewer : UserControl
{
    public TurnDiffViewer()
    {
        InitializeComponent();
    }

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;

    private void CloseTurnDiff(object? sender, RoutedEventArgs e) => ViewModel.Layout.TurnDiffVisible = false;
    
}