using Avalonia.Controls;
using Avalonia.Input.Platform;
using Avalonia.Interactivity;
using Avalonia.Threading;
using Avalonia.VisualTree;
using SvgControl = Avalonia.Svg.Skia.Svg;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.Chat;

public sealed partial class ChatArea : UserControl
{
    private bool _scrollPending;
    private bool _forceScrollPending;
    private bool _scrollingToEnd;
    private bool _followTail = true;
    private object? _messageSource;

    public ChatArea()
    {
        InitializeComponent();
    }

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;

    internal void ScrollConversationToEnd()
    {
        var sourceChanged = !ReferenceEquals(_messageSource, ViewModel.Messages);
        if (sourceChanged)
        {
            _messageSource = ViewModel.Messages;
            _followTail = true;
            _forceScrollPending = true;
        }
        if (!_followTail && !_forceScrollPending) return;
        QueueScrollToEnd();
    }

    private void QueueScrollToEnd()
    {
        if (_scrollPending) return;
        _scrollPending = true;
        Dispatcher.UIThread.Post(() =>
        {
            _scrollPending = false;
            var force = _forceScrollPending;
            _forceScrollPending = false;
            if ((!force && !_followTail) || ViewModel.Messages.Count == 0) return;

            _scrollingToEnd = true;
            ConversationScroller.ScrollToEnd();
            _followTail = true;
            Dispatcher.UIThread.Post(
                () => _scrollingToEnd = false,
                DispatcherPriority.Background);
        }, DispatcherPriority.Background);
    }

    private void ConversationScrollChanged(object? sender, ScrollChangedEventArgs e)
    {
        if (!_scrollingToEnd && Math.Abs(e.OffsetDelta.Y) > 0.1)
            _followTail = IsNearConversationEnd();

        // Markdown content can gain height over several layout passes. Continue following only
        // when the user was already at the end; never steal their position while reading history.
        if (_followTail && e.ExtentDelta.Y > 0.1)
            QueueScrollToEnd();
    }

    private bool IsNearConversationEnd() =>
        ConversationScroller.Extent.Height
        - ConversationScroller.Viewport.Height
        - ConversationScroller.Offset.Y <= 32;

    private async void RetrySession(object? sender, RoutedEventArgs e)
    {
        if (ViewModel.SelectedSession is { } session) await ViewModel.SelectSessionAsync(session);
    }

    private void ToggleTurnProcess(object? sender, RoutedEventArgs e)
    {
        if ((sender as Control)?.DataContext is MessageItem message)
            ViewModel.ToggleTurnProcess(message);
    }

    private async void CopyMessage(object? sender, RoutedEventArgs e)
    {
        if ((sender as Control)?.DataContext is not MessageItem message ||
            TopLevel.GetTopLevel(this)?.Clipboard is not { } clipboard) return;

        await clipboard.SetTextAsync(message.Text);
        if (sender is not Button button) return;

        ToolTip.SetTip(button, "Copied");
        if (button.GetVisualDescendants().OfType<SvgControl>().FirstOrDefault() is { } icon)
            SvgControl.SetCss(icon, this.FindResource("SuccessSvgCss") as string);
        await Task.Delay(1400);
        ToolTip.SetTip(button, "Copy response");
        if (button.GetVisualDescendants().OfType<SvgControl>().FirstOrDefault() is { } resetIcon)
            SvgControl.SetCss(resetIcon, this.FindResource("IconSvgCss") as string);
    }
}
