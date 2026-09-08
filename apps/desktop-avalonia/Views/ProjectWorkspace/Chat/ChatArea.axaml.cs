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
    private ScrollViewer? _conversationScroller;
    public event EventHandler? ExpandedComposerRequested;
    public event Action<MessageItem>? LongUserMessageRequested;
    public event Action<MessageItem>? ToolDetailRequested;

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

    internal void ScrollConversationToEndForSessionEntry()
    {
        // A newly loaded session can realize virtualized rows over multiple
        // layout passes. Both corrections belong to this one entry action;
        // later message changes never request another automatic scroll.
        Dispatcher.UIThread.Post(() =>
        {
            AttachConversationScroller();
            SetConversationOffsetToBottom();
            Dispatcher.UIThread.Post(SetConversationOffsetToBottom, DispatcherPriority.Background);
        }, DispatcherPriority.Loaded);
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
        ScrollToBottomButton.IsVisible = !atBottom && distanceFromBottom > 1;
    }

    private void ScrollToBottom(object? sender, RoutedEventArgs e)
    {
        ScrollToBottomButton.IsVisible = false;
        AttachConversationScroller();
        SetConversationOffsetToBottom();
        Dispatcher.UIThread.Post(SetConversationOffsetToBottom, DispatcherPriority.Loaded);
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
