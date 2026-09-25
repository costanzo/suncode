using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.VisualTree;
using SunCode.Desktop.Infrastructure;
using System.Collections.ObjectModel;

namespace SunCode.Desktop.Views.ProjectWorkspace;

internal sealed partial class MergedWorkspaceTabs : UserControl
{
    private Button? _dragButton;
    private string? _dragProjectId;
    private PixelPoint _dragStart;

    internal ObservableCollection<MergedProjectTab> Tabs { get; } = [];
    internal event Action<string>? ProjectSelected;
    internal event Action<string, PixelPoint>? ProjectTornOff;

    public MergedWorkspaceTabs()
    {
        InitializeComponent();
        TabsList.ItemsSource = Tabs;
        AddHandler(PointerPressedEvent, TabPointerPressed, RoutingStrategies.Tunnel, handledEventsToo: true);
        AddHandler(PointerMovedEvent, TabPointerMoved, RoutingStrategies.Tunnel, handledEventsToo: true);
        AddHandler(PointerReleasedEvent, TabPointerReleased, RoutingStrategies.Tunnel, handledEventsToo: true);
    }

    internal void Select(string projectId)
    {
        foreach (var tab in Tabs) tab.IsSelected = tab.ProjectId == projectId;
    }

    internal void Remove(string projectId)
    {
        var tab = Tabs.FirstOrDefault(item => item.ProjectId == projectId);
        if (tab is not null) Tabs.Remove(tab);
    }

    private void SelectTab(object? sender, RoutedEventArgs e)
    {
        if ((sender as Control)?.DataContext is MergedProjectTab tab)
            ProjectSelected?.Invoke(tab.ProjectId);
    }

    private void TabPointerPressed(object? sender, PointerPressedEventArgs e)
    {
        var button = FindTabButton(e.Source);
        if (button?.DataContext is not MergedProjectTab tab ||
            !e.GetCurrentPoint(button).Properties.IsLeftButtonPressed) return;
        _dragButton = button;
        _dragProjectId = tab.ProjectId;
        _dragStart = Avalonia.VisualExtensions.PointToScreen(button, e.GetPosition(button));
        e.Pointer.Capture(button);
    }

    private void TabPointerMoved(object? sender, PointerEventArgs e)
    {
        var button = _dragButton;
        if (button is null || _dragProjectId is null) return;
        var current = Avalonia.VisualExtensions.PointToScreen(button, e.GetPosition(button));
        var deltaX = current.X - _dragStart.X;
        var deltaY = current.Y - _dragStart.Y;
        if (!MergedWorkspaceWindow.ShouldTearOff(deltaX, deltaY)) return;
        var projectId = _dragProjectId;
        _dragButton = null;
        _dragProjectId = null;
        e.Pointer.Capture(null);
        ProjectTornOff?.Invoke(projectId, current);
        e.Handled = true;
    }

    private void TabPointerReleased(object? sender, PointerReleasedEventArgs e)
    {
        if (_dragButton is null) return;
        _dragButton = null;
        _dragProjectId = null;
    }

    private static Button? FindTabButton(object? source)
    {
        var button = source as Button ?? (source as Visual)?.FindAncestorOfType<Button>();
        return button?.Classes.Contains("merged-project-tab") == true ? button : null;
    }
}

internal sealed class MergedProjectTab : ObservableObject
{
    private bool _isSelected;

    internal string ProjectId { get; }
    public string Title { get; }
    public bool IsSelected
    {
        get => _isSelected;
        set => SetProperty(ref _isSelected, value);
    }

    internal MergedProjectTab(string projectId, string title)
    {
        ProjectId = projectId;
        Title = title;
    }
}
