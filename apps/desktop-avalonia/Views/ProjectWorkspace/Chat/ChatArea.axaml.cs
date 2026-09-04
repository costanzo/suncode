using Avalonia.Controls;
using Avalonia;
using Avalonia.Media.Imaging;
using Avalonia.Input.Platform;
using Avalonia.Interactivity;
using Avalonia.Threading;
using Avalonia.VisualTree;
using SvgControl = Avalonia.Svg.Skia.Svg;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.ProjectWorkspace.Chat;

public sealed partial class ChatArea : UserControl
{
    private bool _scrollPending;
    private bool _forceScrollPending;
    private bool _scrollingToEnd;
    private bool _followTail = true;
    private object? _messageSource;
    private ScrollViewer? _conversationScroller;
    private Control? _dialogReturnFocus;

    public event EventHandler? ExpandedComposerRequested;
    public event Action<MessageItem>? LongUserMessageRequested;

    public ChatArea()
    {
        InitializeComponent();
        AttachedToVisualTree += (_, _) => QueueAttachConversationScroller();
        Loaded += (_, _) => QueueAttachConversationScroller();
        ConversationList.TemplateApplied += (_, _) => QueueAttachConversationScroller();
        ChatInput.ExpandedComposerRequested += ForwardExpandedComposerRequested;
    }

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;

    internal string ExpandedComposerDraft => ChatInput.ExpandedComposerDraft;

    internal void SetComposerText(string text) => ChatInput.SetComposerText(text);

    internal void ClearComposerText() => ChatInput.ClearComposerText();

    private void ForwardExpandedComposerRequested(object? sender, EventArgs e) =>
        ExpandedComposerRequested?.Invoke(this, EventArgs.Empty);

    internal void ScrollConversationToEnd()
    {
        // Session content can be realized after the view's initial template pass.
        // Re-check here so scrolling remains observable for the current session.
        AttachConversationScroller();
        var sourceChanged = !ReferenceEquals(_messageSource, ViewModel.Messages);
        if (sourceChanged)
        {
            _messageSource = ViewModel.Messages;
            _followTail = true;
            _forceScrollPending = true;
        }
        if (!_followTail && !_forceScrollPending) return;
        ScrollToBottomButton.IsVisible = false;
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
            if (_conversationScroller is null)
            {
                AttachConversationScroller();
            }

            // ScrollIntoView(last item) can stop short with a virtualized
            // ListBox and the conversation's bottom clearance. Set the
            // ScrollViewer to its actual maximum offset instead.
            SetConversationOffsetToBottom();
            _followTail = true;
            // A second pass handles the extent update produced by virtualization
            // after the first offset assignment.
            Dispatcher.UIThread.Post(() =>
            {
                SetConversationOffsetToBottom();
                _scrollingToEnd = false;
            }, DispatcherPriority.Background);
        }, DispatcherPriority.Background);
    }

    private void SetConversationOffsetToBottom()
    {
        if (_conversationScroller is null) return;
        var bottomOffset = Math.Max(
            0,
            _conversationScroller.Extent.Height - _conversationScroller.Viewport.Height);
        _conversationScroller.Offset = new Vector(
            _conversationScroller.Offset.X,
            bottomOffset);
    }

    private void AttachConversationScroller()
    {
        var scroller = ConversationList.GetVisualDescendants().OfType<ScrollViewer>().FirstOrDefault();
        if (ReferenceEquals(scroller, _conversationScroller) || scroller is null) return;
        if (_conversationScroller is not null) _conversationScroller.ScrollChanged -= ConversationScrollChanged;
        _conversationScroller = scroller;
        _conversationScroller.ScrollChanged += ConversationScrollChanged;
    }

    private void QueueAttachConversationScroller()
    {
        // The ListBox's ScrollViewer is created by its template. Retry after
        // layout so the control is available even when the view is initially
        // loaded before the template or session content is realized.
        Dispatcher.UIThread.Post(AttachConversationScroller, DispatcherPriority.Loaded);
    }

    private void ConversationScrollChanged(object? sender, ScrollChangedEventArgs e)
    {
        if (_conversationScroller is null) return;
        var distanceFromBottom = _conversationScroller.Extent.Height
            - _conversationScroller.Viewport.Height
            - _conversationScroller.Offset.Y;
        var atBottom = distanceFromBottom <= 32;
        if (!_scrollingToEnd && Math.Abs(e.OffsetDelta.Y) > 0.1 && _conversationScroller is not null)
        {
            _followTail = atBottom;
        }
        ScrollToBottomButton.IsVisible = !atBottom && distanceFromBottom > 1;
        if (_followTail && e.ExtentDelta.Y > 0.1) QueueScrollToEnd();
    }

    private void ScrollToBottom(object? sender, RoutedEventArgs e)
    {
        _followTail = true;
        _forceScrollPending = true;
        ScrollToBottomButton.IsVisible = false;
        AttachConversationScroller();
        SetConversationOffsetToBottom();
        Dispatcher.UIThread.Post(SetConversationOffsetToBottom, DispatcherPriority.Loaded);
        QueueScrollToEnd();
    }

    private void ConversationSelectionChanged(object? sender, SelectionChangedEventArgs e)
    {
        // Conversation rows are not selectable actions. Clear incidental pointer/keyboard
        // selection while retaining ListBox's recycling panel for long histories.
        if (ConversationList.SelectedIndex >= 0) ConversationList.SelectedIndex = -1;
    }

    private async void RetrySession(object? sender, RoutedEventArgs e)
    {
        if (ViewModel.SelectedSession is { } session) await ViewModel.SelectSessionAsync(session);
    }

    private void ToggleTurnProcess(object? sender, RoutedEventArgs e)
    {
        if ((sender as Control)?.DataContext is MessageItem message)
            ViewModel.ToggleTurnProcess(message);
    }

    private void ViewLongUserMessage(object? sender, RoutedEventArgs e)
    {
        if ((sender as Control)?.DataContext is MessageItem message)
            LongUserMessageRequested?.Invoke(message);
    }

    private void OpenToolDetail(object? sender, RoutedEventArgs e)
    {
        if ((sender as Control)?.DataContext is not MessageItem message) return;

        ToolDetailTitle.Text = message.ToolSummaryText;
        ToolDetailState.Text = message.ToolStateText;
        ToolDetailRequest.Text = message.ToolRequest;
        ToolDetailResult.Text = message.ToolResult;
        ToolDetailOutput.Text = message.ToolOutput;
        ToolDetailResultLabel.Text = message.ToolName == "bash" ? "Command output" : "Result";
        ToolDetailError.Text = message.ToolErrorText;
        ToolDetailRequestPanel.IsVisible = message.HasToolRequest;
        ToolDetailResultPanel.IsVisible = message.HasToolResult;
        ToolDetailOutputPanel.IsVisible = message.HasToolOutput;
        ToolDetailErrorPanel.IsVisible = message.HasToolError;
        _dialogReturnFocus = TopLevel.GetTopLevel(this)?.FocusManager?.GetFocusedElement() as Control;
        ToolDetailOverlay.IsVisible = true;
        Dispatcher.UIThread.Post(() => ToolDetailCloseButton.Focus(), DispatcherPriority.Input);
    }

    private void CloseToolDetail(object? sender, RoutedEventArgs e)
    {
        ToolDetailOverlay.IsVisible = false;
        var target = _dialogReturnFocus;
        _dialogReturnFocus = null;
        if (target is not null) Dispatcher.UIThread.Post(() => target.Focus(), DispatcherPriority.Input);
    }

    private async void PreviewMessageAttachment(object? sender, RoutedEventArgs e)
    {
        if ((sender as Control)?.DataContext is not ComposerAttachment attachment ||
            TopLevel.GetTopLevel(this) is not Window owner) return;
        Bitmap? bitmap = null;
        try
        {
            bitmap = File.Exists(attachment.StoragePath) ? new Bitmap(attachment.StoragePath) : attachment.Preview;
            var preview = new Window
            {
                Title = attachment.Name,
                Width = 720,
                Height = 560,
                MinWidth = 360,
                MinHeight = 280,
                WindowStartupLocation = WindowStartupLocation.CenterOwner,
                Background = this.FindResource("SurfaceRaisedBrush") as Avalonia.Media.IBrush,
                Content = new Image { Source = bitmap, Stretch = Avalonia.Media.Stretch.Uniform }
            };
            await preview.ShowDialog(owner);
        }
        finally
        {
            if (bitmap is not null && !ReferenceEquals(bitmap, attachment.Preview)) bitmap.Dispose();
        }
    }

    private async void CopyMessage(object? sender, RoutedEventArgs e)
    {
        if ((sender as Control)?.DataContext is not MessageItem message ||
            TopLevel.GetTopLevel(this)?.Clipboard is not { } clipboard) return;

        await clipboard.SetTextAsync(message.Text);
        ConversationAnnouncement.Text = "Response copied to clipboard";
        if (sender is not Button button) return;

        ToolTip.SetTip(button, "Copied");
        if (button.GetVisualDescendants().OfType<SvgControl>().FirstOrDefault() is { } icon)
        {
            icon.Path = "/Assets/icons/check.svg";
            SvgControl.SetCss(icon, this.FindResource("CopySuccessSvgCss") as string);
        }
        await Task.Delay(1400);
        ConversationAnnouncement.Text = string.Empty;
        ToolTip.SetTip(button, "Copy response");
        if (button.GetVisualDescendants().OfType<SvgControl>().FirstOrDefault() is { } resetIcon)
        {
            resetIcon.Path = "/Assets/icons/copy.svg";
            SvgControl.SetCss(resetIcon, this.FindResource("IconSvgCss") as string);
        }
    }
}
