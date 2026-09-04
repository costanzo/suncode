using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Input.Platform;
using Avalonia.Interactivity;
using Avalonia.Threading;
using Avalonia.VisualTree;
using SvgControl = Avalonia.Svg.Skia.Svg;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.ProjectWorkspace;

public sealed partial class ProjectWorkspace : UserControl
{
    private SessionItem? _sessionDialogTarget;
    private CheckpointItem? _pendingCheckpoint;
    private ExplorerNode? _pendingDependencyDeletion;
    private string _layoutResizeTarget = string.Empty;
    private Point _layoutResizeStart;
    private double _layoutResizeStartNavigationWidth;
    private double _layoutResizeStartReviewWidth;
    private double _layoutResizeStartBottomHeight;
    private string _expandedComposerDraft = string.Empty;
    private MessageItem? _longUserMessage;

    public ProjectWorkspace()
    {
        InitializeComponent();
        ChatArea.ExpandedComposerRequested += ShowExpandedComposer;
        ChatArea.LongUserMessageRequested += ShowLongUserMessage;
    }

    private WorkspaceWindow? Owner => TopLevel.GetTopLevel(this) as WorkspaceWindow;
    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;

    internal void ScrollConversationToEnd() =>
        ChatArea.ScrollConversationToEnd();

    internal void ClampGitViewerHeight()
    {
        if (TopLevel.GetTopLevel(this) is not Window window) return;
        ViewModel.BottomDrawerHeight = Math.Clamp(ViewModel.BottomDrawerHeight, 240, Math.Max(240, window.Bounds.Height - 300));
    }

    internal bool HandleEscape()
    {
        if (ExpandedComposerModal.IsOpen)
        {
            HideExpandedComposer();
            return true;
        }

        if (SessionDialogModal.IsOpen)
        {
            HideSessionDialog();
            return true;
        }

        if (UndoDialogModal.IsOpen)
        {
            HideUndoDialog();
            return true;
        }

        if (DependencyDeleteDialogModal.IsOpen)
        {
            HideDependencyDeleteDialog();
            return true;
        }

        if (LongUserMessageModal.IsOpen)
        {
            HideLongUserMessage();
            return true;
        }

        return false;
    }

    internal void ShowSessionDialog(string title, string value, string acceptText, SessionItem? target)
    {
        _sessionDialogTarget = target;
        SessionDialogModal.Title = title;
        SessionDialogModal.PrimaryButtonText = acceptText;
        SessionTitleInput.Text = value;
        SessionDialogModal.PrimaryEnabled = !string.IsNullOrWhiteSpace(value);
        SessionDialogModal.IsOpen = true;
        Dispatcher.UIThread.Post(() =>
        {
            SessionTitleInput.Focus();
            SessionTitleInput.SelectAll();
        }, DispatcherPriority.Input);
    }

    internal void ShowUndoDialog(CheckpointItem checkpoint)
    {
        _pendingCheckpoint = checkpoint;
        UndoPathsText.Text = checkpoint.PathsText;
        UndoDialogModal.IsOpen = true;
    }

    internal void ShowArchiveDialog(SessionItem session)
    {
        Owner?.ShowArchiveConfirmation(session);
    }

    internal void ShowDependencyDeleteDialog(ExplorerNode node)
    {
        _pendingDependencyDeletion = node;
        DependencyDeleteName.Text = node.Name;
        DependencyDeleteDialogModal.IsOpen = true;
    }

    private void ToggleNavigation(object? sender, RoutedEventArgs e)
    {
        if (ViewModel.NavigationVisible && !ViewModel.ExplorerVisible)
        {
            ViewModel.NavigationVisible = false;
            return;
        }
        ViewModel.ExplorerVisible = false;
        ViewModel.NavigationVisible = true;
    }

    private async void ToggleExplorer(object? sender, RoutedEventArgs e)
    {
        if (ViewModel.NavigationVisible && ViewModel.ExplorerVisible)
        {
            ViewModel.NavigationVisible = false;
            return;
        }
        ViewModel.ExplorerVisible = true;
        ViewModel.NavigationVisible = true;
        await ViewModel.LoadExplorerRootsAsync();
    }

    private void ToggleReview(object? sender, RoutedEventArgs e) =>
        ViewModel.ReviewVisible = !ViewModel.ReviewVisible;

    internal void ToggleGitViewer()
    {
        ViewModel.GitVisible = !ViewModel.GitVisible;
        if (ViewModel.GitVisible)
        {
            ViewModel.ProviderTraceVisible = false;
            _ = ViewModel.RefreshGitAsync();
        }
    }

    private void ToggleGit(object? sender, RoutedEventArgs e) => ToggleGitViewer();

    private void ToggleProviderTrace(object? sender, RoutedEventArgs e)
    {
        ViewModel.ProviderTraceVisible = !ViewModel.ProviderTraceVisible;
        if (ViewModel.ProviderTraceVisible)
        {
            ViewModel.GitVisible = false;
            _ = ViewModel.RefreshProviderTracesAsync();
        }
    }

    private void LayoutResizePressed(object? sender, PointerPressedEventArgs e)
    {
        if (sender is not Control handle || handle.Tag is not string target ||
            !e.GetCurrentPoint(handle).Properties.IsLeftButtonPressed ||
            TopLevel.GetTopLevel(this) is not Window window) return;
        _layoutResizeTarget = target;
        _layoutResizeStart = e.GetPosition(window);
        _layoutResizeStartNavigationWidth = ViewModel.NavigationPaneWidth;
        _layoutResizeStartReviewWidth = ViewModel.ReviewPaneWidth;
        _layoutResizeStartBottomHeight = ViewModel.BottomDrawerHeight;
        e.Pointer.Capture(handle);
        e.Handled = true;
    }

    private void LayoutResizeMoved(object? sender, PointerEventArgs e)
    {
        if (string.IsNullOrEmpty(_layoutResizeTarget) || TopLevel.GetTopLevel(this) is not Window window) return;
        var point = e.GetPosition(window);
        var deltaX = point.X - _layoutResizeStart.X;
        var deltaY = point.Y - _layoutResizeStart.Y;
        switch (_layoutResizeTarget)
        {
            case "Navigation":
                ViewModel.NavigationPaneWidth = ClampPaneWidth(_layoutResizeStartNavigationWidth + deltaX, window.Bounds.Width, 236, 300);
                break;
            case "Review":
                ViewModel.ReviewPaneWidth = ClampPaneWidth(_layoutResizeStartReviewWidth - deltaX, window.Bounds.Width, 276, 352);
                break;
            case "BottomDrawer":
                ViewModel.BottomDrawerHeight = Math.Clamp(_layoutResizeStartBottomHeight - deltaY, 240, Math.Max(240, window.Bounds.Height - 300));
                break;
        }
        e.Handled = true;
    }

    private void LayoutResizeReleased(object? sender, PointerReleasedEventArgs e)
    {
        if (string.IsNullOrEmpty(_layoutResizeTarget)) return;
        _layoutResizeTarget = string.Empty;
        e.Pointer.Capture(null);
        e.Handled = true;
    }

    private static double ClampPaneWidth(double width, double windowWidth, double min, double max) =>
        Math.Clamp(width, min, Math.Min(max, Math.Max(min, windowWidth - 560)));

    private void SessionTitleChanged(object? sender, TextChangedEventArgs e) =>
        SessionDialogModal.PrimaryEnabled = !string.IsNullOrWhiteSpace(SessionTitleInput.Text);

    private async void SessionTitleKeyDown(object? sender, KeyEventArgs e)
    {
        if (e.Key != Key.Enter || e.KeyModifiers.HasFlag(KeyModifiers.Shift)) return;
        e.Handled = true;
        await SubmitSessionDialogAsync();
    }

    private async void SubmitSessionDialog(object? sender, RoutedEventArgs e) =>
        await SubmitSessionDialogAsync();

    private async Task SubmitSessionDialogAsync()
    {
        var title = SessionTitleInput.Text?.Trim();
        if (string.IsNullOrWhiteSpace(title)) return;
        var target = _sessionDialogTarget;
        HideSessionDialog();
        if (target is null) await ViewModel.CreateSessionAsync(title);
        else await ViewModel.RenameSessionAsync(target, title);
    }

    private void CloseSessionDialog(object? sender, RoutedEventArgs e) => HideSessionDialog();

    private void HideSessionDialog()
    {
        SessionDialogModal.IsOpen = false;
        _sessionDialogTarget = null;
    }

    private void CloseUndoDialog(object? sender, RoutedEventArgs e) => HideUndoDialog();

    private async void ConfirmUndoDialog(object? sender, RoutedEventArgs e)
    {
        var checkpoint = _pendingCheckpoint;
        HideUndoDialog();
        if (checkpoint is not null) await ViewModel.RestoreCheckpointAsync(checkpoint);
    }

    private void HideUndoDialog()
    {
        UndoDialogModal.IsOpen = false;
        _pendingCheckpoint = null;
    }

    private void CloseDependencyDeleteDialog(object? sender, RoutedEventArgs e) => HideDependencyDeleteDialog();

    private async void ConfirmDependencyDeleteDialog(object? sender, RoutedEventArgs e)
    {
        var dependency = _pendingDependencyDeletion;
        HideDependencyDeleteDialog();
        if (dependency is not null) await ViewModel.RemoveProjectDependencyAsync(dependency);
    }

    private void HideDependencyDeleteDialog()
    {
        DependencyDeleteDialogModal.IsOpen = false;
        _pendingDependencyDeletion = null;
    }

    private void ShowLongUserMessage(MessageItem message)
    {
        _longUserMessage = message;
        LongUserMessageText.Text = message.Text;
        LongUserMessageCount.Text = $"{message.Text.Length} characters";
        LongUserMessageModal.IsOpen = true;
        Dispatcher.UIThread.Post(() => LongUserMessageCopyButton.Focus(), DispatcherPriority.Input);
    }

    private void CloseLongUserMessage(object? sender, RoutedEventArgs e) => HideLongUserMessage();

    private void HideLongUserMessage()
    {
        LongUserMessageModal.IsOpen = false;
        _longUserMessage = null;
    }

    private async void CopyLongUserMessage(object? sender, RoutedEventArgs e)
    {
        if (_longUserMessage is null || TopLevel.GetTopLevel(this)?.Clipboard is not { } clipboard) return;
        await clipboard.SetTextAsync(_longUserMessage.Text);
        LongUserMessageAnnouncement.Text = "Message copied to clipboard";
        if (sender is not Button button) return;
        ToolTip.SetTip(button, "Copied");
        if (button.GetVisualDescendants().OfType<SvgControl>().FirstOrDefault() is { } icon)
        {
            icon.Path = "/Assets/icons/check.svg";
            SvgControl.SetCss(icon, this.FindResource("CopySuccessSvgCss") as string);
        }
        await Task.Delay(1400);
        ToolTip.SetTip(button, "Copy message");
        LongUserMessageAnnouncement.Text = string.Empty;
        if (button.GetVisualDescendants().OfType<SvgControl>().FirstOrDefault() is { } resetIcon)
        {
            resetIcon.Path = "/Assets/icons/copy.svg";
            SvgControl.SetCss(resetIcon, this.FindResource("IconSvgCss") as string);
        }
    }

    private void ShowExpandedComposer(object? sender, EventArgs e)
    {
        _expandedComposerDraft = ChatArea.ExpandedComposerDraft;
        ExpandedComposerInput.Text = _expandedComposerDraft;
        ExpandedComposerCount.Text = $"{_expandedComposerDraft.Length} characters";
        ExpandedComposerModal.IsOpen = true;
        Dispatcher.UIThread.Post(() => ExpandedComposerInput.Focus(), DispatcherPriority.Input);
    }

    private void ExpandedComposerChanged(object? sender, TextChangedEventArgs e)
    {
        _expandedComposerDraft = ExpandedComposerInput.Text ?? string.Empty;
        ExpandedComposerCount.Text = $"{_expandedComposerDraft.Length} characters";
    }

    private void CloseExpandedComposer(object? sender, RoutedEventArgs e)
    {
        HideExpandedComposer();
    }

    private void HideExpandedComposer()
    {
        ChatArea.SetComposerText(_expandedComposerDraft);
        ExpandedComposerModal.IsOpen = false;
    }

    private async void SubmitExpandedComposer(object? sender, RoutedEventArgs e)
    {
        ChatArea.SetComposerText(_expandedComposerDraft);
        ExpandedComposerModal.IsOpen = false;
        if (!ViewModel.CanSubmit) return;

        await ViewModel.SubmitTurnAsync();
        if (string.IsNullOrEmpty(ViewModel.ComposerText))
        {
            ChatArea.ClearComposerText();
            _expandedComposerDraft = string.Empty;
        }
    }

    private void OpenSettings(object? sender, RoutedEventArgs e) => Owner?.ShowSettings();
    private void CloseProjectWindow(object? sender, RoutedEventArgs e) => Owner?.Close();
    private void MinimizeWindow(object? sender, RoutedEventArgs e) => Owner?.MinimizeWindow();
    private void ToggleFullScreen(object? sender, RoutedEventArgs e) => Owner?.ToggleFullScreen();
    private void TitleBarPressed(object? sender, PointerPressedEventArgs e) => Owner?.TitleBarPressed(sender, e);
    private void TitleBarMoved(object? sender, PointerEventArgs e) => Owner?.TitleBarMoved(sender, e);
    private void TitleBarReleased(object? sender, PointerReleasedEventArgs e) => Owner?.TitleBarReleased(sender, e);
    private void TitleBarDoubleTapped(object? sender, TappedEventArgs e) => Owner?.TitleBarDoubleTapped(sender, e);
    private void TrafficLightEntered(object? sender, PointerEventArgs e) => WorkspaceWindow.SetTrafficLightState(sender, "hover");
    private void TrafficLightExited(object? sender, PointerEventArgs e) => WorkspaceWindow.SetTrafficLightState(sender, "normal");
    private void TrafficLightPressed(object? sender, PointerPressedEventArgs e) => WorkspaceWindow.SetTrafficLightState(sender, "press");
    private void TrafficLightReleased(object? sender, PointerReleasedEventArgs e) => WorkspaceWindow.SetTrafficLightState(sender, "hover");
}
