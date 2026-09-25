using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Platform.Storage;
using Avalonia.VisualTree;
using Avalonia.Input;
using Avalonia;

using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.ProjectWorkspace.Navigation;

public sealed partial class ProjectExplorer : UserControl
{
    private const double DragThresholdSquared = 16;
    private Point? _dragStart;
    private ExplorerNode? _dragNode;
    private PointerPressedEventArgs? _dragPress;
    
    public ProjectExplorer()
    {
        InitializeComponent();
        ExplorerTree.AddHandler(TreeViewItem.ExpandedEvent, ExplorerItemExpanded, RoutingStrategies.Bubble);
        ExplorerTree.AddHandler(TreeViewItem.PointerPressedEvent, ExplorerPointerPressed, RoutingStrategies.Tunnel);
        ExplorerTree.AddHandler(TreeViewItem.PointerMovedEvent, ExplorerPointerMoved, RoutingStrategies.Tunnel);
        ExplorerTree.AddHandler(TreeViewItem.PointerReleasedEvent, ExplorerPointerReleased, RoutingStrategies.Tunnel);
    }

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;

    private async void ExplorerItemExpanded(object? sender, RoutedEventArgs e)
    {
        if (e.Source is TreeViewItem { DataContext: ExplorerNode node })
            await ViewModel.LoadExplorerChildrenAsync(node);
    }

    private async void ExplorerSelectionChanged(object? sender, SelectionChangedEventArgs e)
    {
        if (e.AddedItems.OfType<ExplorerNode>().FirstOrDefault() is not {IsFile: true} node) return;
        if (_dragNode is not null) return;
        await ViewModel.SelectExplorerFileAsync(node);
    }

    private void ExplorerDoubleTapped(object? sender, TappedEventArgs e)
    {
        if (OriginatesFromButton(e.Source)) return;
        if (e.Source is not Visual visual) return;
        if (visual.FindAncestorOfType<TreeViewItem>()?.DataContext is not ExplorerNode
            {
                ToggleOnDoubleClick: true
            } node) return;
        node.IsExpanded = !node.IsExpanded;
        e.Handled = true;
    }

    private void ExplorerPointerPressed(object? sender, PointerPressedEventArgs e)
    {
        _dragStart = null;
        _dragNode = null;
        _dragPress = null;
        if (e.Source is not Visual visual) return;
        if (visual.FindAncestorOfType<TreeViewItem>()?.DataContext is not ExplorerNode { IsFile: true } node) return;
        if (OriginatesFromButton(e.Source)) return;
        
        _dragStart = e.GetPosition(ExplorerTree);
        _dragNode = node;
        _dragPress = e;
    }

    private async void ExplorerPointerMoved(object? sender, PointerEventArgs e)
    {
        if (_dragStart is not { } start || _dragNode is not { } node || _dragPress is not { } press) return;
        
        var position = e.GetPosition(ExplorerTree);
        if (PointDistanceSquared(start, position) < DragThresholdSquared) return;

        var payload = ExplorerDragPayload.FromNode(node);
        _dragStart = null;
        _dragNode = null;
        _dragPress = null;

        var data = new DataTransfer();
        data.Add(DataTransferItem.CreateText(payload.Serialize()));
        e.Handled = true;
        await DragDrop.DoDragDropAsync(press, data, DragDropEffects.Copy);
    }

    private async void ExplorerPointerReleased(object? sender, PointerReleasedEventArgs e)
    {
        var node = _dragNode;
        _dragStart = null;
        _dragNode = null;
        _dragPress = null;
        if (node is { IsFile: true })
            await ViewModel.SelectExplorerFileAsync(node);
    }

    private static double PointDistanceSquared(Point a, Point b)
    {
        var dx = a.X - b.X;
        var dy = a.Y - b.Y;
        return dx * dx + dy * dy;
    }
    
    private static bool OriginatesFromButton(object? source) =>
        source is Button || source is Visual visual && visual.FindAncestorOfType<Button>() is not null;
    

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
