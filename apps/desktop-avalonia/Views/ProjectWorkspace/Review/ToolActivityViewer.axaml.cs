using System.ComponentModel;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Input.Platform;
using Avalonia.Interactivity;
using Avalonia.Threading;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.ProjectWorkspace.Review;

public sealed partial class ToolActivityViewer : UserControl
{
    private bool _followingOutput = true;
    private bool _scrollingOutput;
    private ToolActivityItem? _observedTool;
    private DesktopViewModel? _observedViewModel;

    public ToolActivityViewer()
    {
        InitializeComponent();
        OutputScroller.ScrollChanged += OutputScrollChanged;
        DataContextChanged += (_, _) => ObserveViewModel();
    }

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;

    private void ToolSelected(object? sender, SelectionChangedEventArgs e)
    {
        if (e.AddedItems.OfType<ToolActivityItem>().FirstOrDefault() is { } tool)
        {
            var turn = ViewModel.ToolActivityTurns.First(item => item.TurnId == tool.TurnId);
            ViewModel.SelectToolActivity(turn, tool);
        }
        else if (e.AddedItems.OfType<ToolActivityTurnItem>().FirstOrDefault() is { } turn)
        {
            ViewModel.SelectToolActivityTurn(turn);
        }
        ObserveSelectedTool();
    }

    private void ObserveViewModel()
    {
        if (_observedViewModel is not null) _observedViewModel.PropertyChanged -= ViewModelPropertyChanged;
        _observedViewModel = DataContext as DesktopViewModel;
        if (_observedViewModel is not null) _observedViewModel.PropertyChanged += ViewModelPropertyChanged;
        ObserveSelectedTool();
    }

    private void ViewModelPropertyChanged(object? sender, PropertyChangedEventArgs e)
    {
        if (e.PropertyName == nameof(DesktopViewModel.SelectedToolActivity)) ObserveSelectedTool();
    }

    private void ObserveSelectedTool()
    {
        if (_observedTool is not null) _observedTool.PropertyChanged -= ToolPropertyChanged;
        _observedTool = _observedViewModel?.SelectedToolActivity;
        if (_observedTool is not null) _observedTool.PropertyChanged += ToolPropertyChanged;
        _followingOutput = true;
        QueueOutputScroll();
    }

    private void ToolPropertyChanged(object? sender, PropertyChangedEventArgs e)
    {
        if (e.PropertyName is nameof(ToolActivityItem.Output) or nameof(ToolActivityItem.HasOutput)) QueueOutputScroll();
    }

    private void OutputScrollChanged(object? sender, ScrollChangedEventArgs e)
    {
        if (_scrollingOutput) return;
        var distance = OutputScroller.Extent.Height - OutputScroller.Viewport.Height - OutputScroller.Offset.Y;
        if (Math.Abs(e.OffsetDelta.Y) > 0.1) _followingOutput = distance <= 20;
    }

    private void QueueOutputScroll()
    {
        if (!_followingOutput) return;
        Dispatcher.UIThread.Post(() =>
        {
            if (!_followingOutput) return;
            _scrollingOutput = true;
            OutputScroller.Offset = new Vector(OutputScroller.Offset.X, Math.Max(0, OutputScroller.Extent.Height - OutputScroller.Viewport.Height));
            _scrollingOutput = false;
        }, DispatcherPriority.Background);
    }

    private async void CopyToolActivity(object? sender, RoutedEventArgs e)
    {
        if (TopLevel.GetTopLevel(this)?.Clipboard is not { } clipboard || ViewModel.SelectedToolActivity is not { } tool) return;
        await clipboard.SetTextAsync(string.Join(Environment.NewLine, new[]
        {
            ViewModel.SelectedToolActivityTitle,
            tool.DisplayName,
            $"State: {tool.StateText}",
            $"Tool call: {tool.ToolCallId}",
            "",
            "Request",
            tool.Request,
            "",
            "Live output",
            tool.Output,
            "",
            "Result",
            tool.Result,
            "",
            "Error",
            tool.ErrorText,
        }));
    }

    private void CloseToolActivity(object? sender, RoutedEventArgs e) => ViewModel.ToolActivityVisible = false;
}
