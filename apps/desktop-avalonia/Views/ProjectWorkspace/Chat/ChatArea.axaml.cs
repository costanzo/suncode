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
    private const double ConversationBottomThreshold = 32;
    private ScrollViewer? _conversationScroller;
    private DesktopViewModel? _observedViewModel;
    private bool _followingConversation = true;
    private bool _scrollingConversation;
    private string _lastTurnState = string.Empty;
    private DispatcherTimer? _conversationScrollTimer;
    private double _conversationScrollStart;
    private double _conversationScrollTarget;
    private DateTimeOffset _conversationScrollStartedAt;
    private double? _programmaticConversationOffset;
    public event EventHandler? ExpandedComposerRequested;
    public event Action<MessageItem>? LongUserMessageRequested;
    public event Action<MessageItem>? ToolDetailRequested;

    public ChatArea()
    {
        InitializeComponent();
        AttachedToVisualTree += (_, _) => QueueAttachConversationScroller();
        DetachedFromVisualTree += (_, _) => StopConversationScrollAnimation();
        Loaded += (_, _) => QueueAttachConversationScroller();
        DataContextChanged += (_, _) => ObserveViewModel();
        ChatInput.ExpandedComposerRequested += ForwardExpandedComposerRequested;
    }

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;

    internal string ExpandedComposerDraft => ChatInput.ExpandedComposerDraft;

    internal void SetComposerText(string text) => ChatInput.SetComposerText(text);

    internal void ClearComposerText() => ChatInput.ClearComposerText();

    private void ForwardExpandedComposerRequested(object? sender, EventArgs e) =>
        ExpandedComposerRequested?.Invoke(this, EventArgs.Empty);

    internal void ScrollConversationToEndForSessionEntry()
    {
        // A session entry is an explicit navigation action. Start that
        // conversation in follow mode even if the previous session was scrolled
        // away from its bottom.
        _followingConversation = true;

        // A newly loaded session can realize rows over multiple layout passes.
        // Both corrections belong to this one entry action; turn completion
        // handles any later follow-to-bottom behavior.
        Dispatcher.UIThread.Post(() =>
        {
            AttachConversationScroller();
            // Establish the correct position for the first layout pass, then
            // retry after rows have measured. Without this, the initial extent
            // can be zero and there is nothing for the animation to target.
            SetConversationOffsetToBottom();
            Dispatcher.UIThread.Post(() =>
            {
                AttachConversationScroller();
                AnimateConversationToBottom();
                Dispatcher.UIThread.Post(AnimateConversationToBottom, DispatcherPriority.Background);
            }, DispatcherPriority.Background);
        }, DispatcherPriority.Loaded);
    }

    private void SetConversationOffsetToBottom()
    {
        if (_conversationScroller is null) return;
        var bottomOffset = Math.Max(
            0,
            _conversationScroller.Extent.Height - _conversationScroller.Viewport.Height);
        _scrollingConversation = true;
        try
        {
            var offset = new Vector(
                _conversationScroller.Offset.X,
                bottomOffset);
            _programmaticConversationOffset = offset.Y;
            _conversationScroller.Offset = offset;
        }
        finally
        {
            _scrollingConversation = false;
        }
    }

    private void AnimateConversationToBottom()
    {
        if (_conversationScroller is null) return;

        // Streaming deltas can arrive faster than the animation duration. The
        // active timer already tracks the changing extent, so keep it running
        // instead of restarting the easing curve for every delta.
        if (_conversationScrollTimer is not null) return;

        var currentOffset = _conversationScroller.Offset.Y;
        var targetOffset = Math.Max(
            0,
            _conversationScroller.Extent.Height - _conversationScroller.Viewport.Height);
        if (Math.Abs(targetOffset - currentOffset) <= 1)
        {
            StopConversationScrollAnimation();
            SetConversationOffsetToBottom();
            return;
        }

        StopConversationScrollAnimation();
        _conversationScrollStart = currentOffset;
        _conversationScrollTarget = targetOffset;
        _conversationScrollStartedAt = DateTimeOffset.UtcNow;
        _conversationScrollTimer = new DispatcherTimer
        {
            Interval = TimeSpan.FromMilliseconds(16)
        };
        _conversationScrollTimer.Tick += ConversationScrollAnimationTick;
        _conversationScrollTimer.Start();
    }

    private void ConversationScrollAnimationTick(object? sender, EventArgs e)
    {
        if (_conversationScroller is null)
        {
            StopConversationScrollAnimation();
            return;
        }

        const double durationMilliseconds = 260;
        // The final response can still be measuring while the animation is in
        // progress. Follow the moving extent so the last frame lands on the
        // actual bottom instead of jumping when the timer completes.
        _conversationScrollTarget = Math.Max(
            0,
            _conversationScroller.Extent.Height - _conversationScroller.Viewport.Height);
        var progress = Math.Clamp(
            (DateTimeOffset.UtcNow - _conversationScrollStartedAt).TotalMilliseconds / durationMilliseconds,
            0,
            1);
        // Cubic ease-out: responsive at the start, then settles gently at the
        // latest message instead of stopping abruptly.
        var eased = 1 - Math.Pow(1 - progress, 3);
        var offset = _conversationScrollStart
            + ((_conversationScrollTarget - _conversationScrollStart) * eased);

        _scrollingConversation = true;
        try
        {
            _programmaticConversationOffset = offset;
            _conversationScroller.Offset = new Vector(_conversationScroller.Offset.X, offset);
        }
        finally
        {
            _scrollingConversation = false;
        }

        if (progress >= 1)
        {
            StopConversationScrollAnimation();
            SetConversationOffsetToBottom();
        }
    }

    private void StopConversationScrollAnimation()
    {
        if (_conversationScrollTimer is null) return;
        _conversationScrollTimer.Stop();
        _conversationScrollTimer.Tick -= ConversationScrollAnimationTick;
        _conversationScrollTimer = null;
    }

    private void AttachConversationScroller()
    {
        var scroller = ConversationScroller;
        if (ReferenceEquals(scroller, _conversationScroller) || scroller is null) return;
        if (_conversationScroller is not null) _conversationScroller.ScrollChanged -= ConversationScrollChanged;
        _conversationScroller = scroller;
        _conversationScroller.ScrollChanged += ConversationScrollChanged;
    }

    private void QueueAttachConversationScroller()
    {
        // Retry after layout so the named ScrollViewer is available even when
        // the view is initially loaded before session content is realized.
        Dispatcher.UIThread.Post(AttachConversationScroller, DispatcherPriority.Loaded);
    }

    private void ConversationScrollChanged(object? sender, ScrollChangedEventArgs e)
    {
        if (_conversationScroller is null) return;
        var distanceFromBottom = _conversationScroller.Extent.Height
            - _conversationScroller.Viewport.Height
            - _conversationScroller.Offset.Y;
        var atBottom = distanceFromBottom <= ConversationBottomThreshold;

        // Offset changes caused by content/layout updates must not make us
        // forget that the user was following the stream. Only an unguarded
        // offset change (wheel, scrollbar drag, touch, keyboard) represents
        // user navigation.
        var isProgrammaticOffset = _programmaticConversationOffset is { } expectedOffset
            && Math.Abs(expectedOffset - _conversationScroller.Offset.Y) <= 0.5;
        if (!_scrollingConversation && !isProgrammaticOffset && Math.Abs(e.OffsetDelta.Y) > 0.1)
        {
            StopConversationScrollAnimation();
            _programmaticConversationOffset = null;
            _followingConversation = atBottom;
        }
        else if (isProgrammaticOffset && !_scrollingConversation)
        {
            _programmaticConversationOffset = null;
        }

        ScrollToBottomButton.IsVisible = !atBottom && distanceFromBottom > 1;
    }

    private void ScrollToBottom(object? sender, RoutedEventArgs e)
    {
        _followingConversation = true;
        ScrollToBottomButton.IsVisible = false;
        AttachConversationScroller();
        AnimateConversationToBottom();
        Dispatcher.UIThread.Post(AnimateConversationToBottom, DispatcherPriority.Loaded);
    }

    private void ObserveViewModel()
    {
        if (_observedViewModel is not null)
            _observedViewModel.PropertyChanged -= ViewModelPropertyChanged;

        _observedViewModel = DataContext as DesktopViewModel;
        _lastTurnState = _observedViewModel?.ActiveTurnState ?? string.Empty;
        if (_observedViewModel is not null)
            _observedViewModel.PropertyChanged += ViewModelPropertyChanged;
    }

    private void ViewModelPropertyChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs e)
    {
        if (_observedViewModel is null) return;

        if (e.PropertyName is nameof(DesktopViewModel.Messages) or nameof(DesktopViewModel.HasMessages))
        {
            QueueScrollToConversationEnd();
            return;
        }

        if (e.PropertyName != nameof(DesktopViewModel.ActiveTurnState)) return;

        var state = _observedViewModel.ActiveTurnState;
        if (state == "admitted")
        {
            // Sending a new turn is an explicit request to follow its output,
            // even when the previous turn was being reviewed higher up.
            _followingConversation = true;
            QueueScrollToConversationEnd();
        }
        var enteredTerminalState = IsTerminalTurnState(state) && !IsTerminalTurnState(_lastTurnState);
        _lastTurnState = state;
        if (enteredTerminalState) QueueScrollToConversationEnd();
    }

    private void QueueScrollToConversationEnd()
    {
        if (!_followingConversation) return;

        Dispatcher.UIThread.Post(() =>
        {
            // The user may have scrolled while the layout/event was queued.
            if (!_followingConversation) return;
            AttachConversationScroller();
            AnimateConversationToBottom();
        }, DispatcherPriority.Background);
    }

    private static bool IsTerminalTurnState(string state) =>
        state is "completed" or "failed" or "cancelled" or "interrupted";

    private async void RetrySession(object? sender, RoutedEventArgs e)
    {
        if (ViewModel.SelectedSession is { } session) await ViewModel.SelectSessionAsync(session);
    }

    private void ViewLongUserMessage(object? sender, RoutedEventArgs e)
    {
        if ((sender as Control)?.DataContext is MessageItem message)
            LongUserMessageRequested?.Invoke(message);
    }

    private void OpenToolDetail(object? sender, RoutedEventArgs e)
    {
        if ((sender as Control)?.DataContext is not MessageItem message) return;
        ToolDetailRequested?.Invoke(message);
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
