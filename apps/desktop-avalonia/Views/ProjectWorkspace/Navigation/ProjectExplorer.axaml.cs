using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Platform.Storage;
using Avalonia.VisualTree;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.ProjectWorkspace.Navigation;

public sealed partial class ProjectExplorer : UserControl
{
    public ProjectExplorer()
    {
        InitializeComponent();
        ExplorerTree.AddHandler(TreeViewItem.ExpandedEvent, ExplorerItemExpanded, RoutingStrategies.Bubble);
    }

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;

    private async void ExplorerItemExpanded(object? sender, RoutedEventArgs e)
    {
        if (e.Source is TreeViewItem { DataContext: ExplorerNode node })
            await ViewModel.LoadExplorerChildrenAsync(node);
    }

    private async void AddDependency(object? sender, RoutedEventArgs e)
    {
        var topLevel = TopLevel.GetTopLevel(this);
        if (topLevel?.StorageProvider is null) return;
        var folders = await topLevel.StorageProvider.OpenFolderPickerAsync(new FolderPickerOpenOptions
        {
            Title = "Add a read-only source dependency",
            AllowMultiple = false
        });
        var path = folders.FirstOrDefault()?.TryGetLocalPath();
        if (!string.IsNullOrWhiteSpace(path)) await ViewModel.AddProjectDependencyAsync(path);
    }

    private void DeleteDependency(object? sender, RoutedEventArgs e)
    {
        if (sender is MenuItem { CommandParameter: ExplorerNode node })
            this.FindAncestorOfType<ProjectWorkspace>()?.ShowDependencyDeleteDialog(node);
    }

    private async void RefreshExplorer(object? sender, RoutedEventArgs e)
        => await ViewModel.RefreshExplorerAsync();
}
