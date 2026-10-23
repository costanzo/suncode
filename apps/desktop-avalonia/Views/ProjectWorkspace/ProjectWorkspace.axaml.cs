using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Input.Platform;
using Avalonia.Interactivity;
using Avalonia.Threading;
using Avalonia.VisualTree;
using SvgControl = Avalonia.Svg.Skia.Svg;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Views.ProjectWorkspace.Review;

namespace SunCode.Desktop.Views.ProjectWorkspace;

public sealed partial class ProjectWorkspace : UserControl
{
    private WorkspaceResizeTarget? _layoutResizeTarget;
    private Point _layoutResizeStart;
    private double _layoutResizeStartSize;
    private string _expandedComposerDraft = string.Empty;
    private MessageItem? _longUserMessage;

    public ProjectWorkspace()
    {
        InitializeComponent();
        var isWindows = OperatingSystem.IsWindows();
        MacTitleBarControls.IsVisible = !isWindows;
        MacTitleBarActions.IsVisible = !isWindows;
        WindowsTitleBarControls.IsVisible = isWindows;
        if (isWindows)
        {
            // The Windows title bar has no leading traffic lights, so keep
            // the project switcher aligned with the client-area edge.
            ProjectSwitcherControl.Margin = new Thickness(12, 0, 0, 0);
        }
        ChatArea.ExpandedComposerRequested += ShowExpandedComposer;
        ChatArea.LongUserMessageRequested += ShowLongUserMessage;
        ChatArea.ToolDetailRequested += ShowToolActivity;
        ProjectSwitcherControl.OpenProjectRequested += OpenProjectRequested;
        ProjectSwitcherControl.ProjectRequested += ProjectRequested;
        ContentSwitcherControl.ContentRequested += ContentRequested;
        AddHandler(PointerPressedEvent, LayoutResizePressed);
        AddHandler(PointerMovedEvent, LayoutResizeMoved);
        AddHandler(PointerReleasedEvent, LayoutResizeReleased);
    }

    private WorkspaceWindow? Owner => TopLevel.GetTopLevel(this) as WorkspaceWindow;
    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;
    private WorkspaceLayoutViewModel Layout => ViewModel.Layout;

    internal void ScrollConversationToEndForSessionEntry() =>
        ChatArea.ScrollConversationToEndForSessionEntry();

    internal bool HasMenuBlockingOverlay => ExpandedComposerModal.IsOpen || LongUserMessageModal.IsOpen;

    internal async Task FocusConversationAsync()
    {
        if (ViewModel.SelectedSession is not { } session) return;
        await ViewModel.SelectSessionAsync(session);
        if (ViewModel.SelectedSession?.SessionId == session.SessionId)
            Dispatcher.UIThread.Post(() => ChatArea.FocusComposer(), DispatcherPriority.Input);
    }

    internal void ToggleSessionsPanel() => Layout.ToggleSessions();

    internal async Task ToggleExplorerPanelAsync()
    {
        if (Layout.ToggleExplorer()) await ViewModel.Explorer.LoadRootsAsync();
    }

    internal void ToggleReviewPanel() => Layout.ToggleReview();
    internal void ToggleBrowserPanel() => Layout.PreviewVisible = !Layout.PreviewVisible;

    internal async Task ToggleChildSessionsPanelAsync()
    {
        Layout.ToggleChildSessions();
        if (Layout.ChildSessionsVisible) await ViewModel.LoadChildSessionsAsync();
    }

    internal async Task ToggleProviderTracePanelAsync()
    {
        if (Layout.ToggleBottomDrawer(WorkspaceBottomDrawer.ProviderTrace)) await ViewModel.ProviderTrace.RefreshAsync();
    }

    internal void ToggleToolActivityPanel() => Layout.ToggleBottomDrawer(WorkspaceBottomDrawer.ToolActivity);

    internal Task ShowPendingApprovalAsync() => ShowPendingRequestAsync(approval: true);
    internal Task ShowPendingQuestionAsync() => ShowPendingRequestAsync(approval: false);

    private async Task ShowPendingRequestAsync(bool approval)
    {
        var sessionId = ViewModel.SelectedSession?.SessionId;
        await FocusConversationAsync();
        if (sessionId is null || ViewModel.SelectedSession?.SessionId != sessionId) return;
        Layout.PreviewVisible = false;
        if (!Layout.ReviewVisible) Layout.ToggleReview();
        Dispatcher.UIThread.Post(() =>
        {
            if (ViewModel.SelectedSession?.SessionId == sessionId)
                this.GetVisualDescendants().OfType<AgentSidebar>().FirstOrDefault()?.FocusPendingRequest(approval);
        }, DispatcherPriority.Loaded);
    }

    internal void SetMergedTabs(Control? tabs)
    {
        var isVisible = tabs is not null;
        if (ReferenceEquals(ProjectTabsHost.Child, tabs) && ProjectTabsHost.IsVisible == isVisible) return;
        ProjectTabsHost.Child = tabs;
        ProjectTabsHost.Height = tabs is null ? 0 : 38;
        ProjectTabsHost.IsVisible = isVisible;
    }

    internal void ClampGitViewerHeight()
    {
        if (TopLevel.GetTopLevel(this) is not Window window) return;
        Layout.UpdateLayoutSize(window.Bounds.Width, window.Bounds.Height);
    }

    internal bool HandleEscape()
    {
        if (ContentSwitcherControl.CloseFlyout()) return true;

        if (ExpandedComposerModal.IsOpen)
        {
            HideExpandedComposer();
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
        var description = target is null
            ? "Give this conversation a name before you begin"
            : "Choose a new name for this conversation.";
        Owner?.ShowSessionNameDialog(
            title,
            description,
            value,
            acceptText,
            name => _ = SubmitSessionNameAsync(target, name),
            new Size(480, 260));
    }

    internal void ShowUndoDialog(CheckpointItem checkpoint)
    {
        Owner?.ShowConfirmation(
            "Undo this turn's file changes?",
            "SunCode will restore the files changed during this turn.",
            "FILES",
            checkpoint.PathsText,
            LocalizationService.GetString("LocUndoChanges", "Undo changes"),
            () => _ = ViewModel.RestoreCheckpointAsync(checkpoint),
            LocalizationService.GetString("LocExternalSideEffects", "External side effects cannot be reversed."));
    }

    internal void ShowArchiveDialog(SessionItem session)
    {
        Owner?.ShowArchiveConfirmation(session);
    }

    internal void ShowRestoreDialog(SessionItem session) =>
        Owner?.ShowSessionConfirmation(
            session,
            LocalizationService.GetString("LocRestoreSessionTitle", "Restore this session?"),
            LocalizationService.GetString("LocRestoreSessionMessage", "It will return to the active session list and become editable again."),
            LocalizationService.GetString("LocRestoreSession", "Restore session"),
            () => _ = ViewModel.RestoreSessionAsync(session));

    internal void ShowPermanentDeleteDialog(SessionItem session) =>
        Owner?.ShowSessionConfirmation(
            session,
            LocalizationService.GetString("LocDeleteSessionPermanentlyTitle", "Delete this session permanently?"),
            LocalizationService.GetString("LocDeleteSessionPermanentlyMessage", "This permanently removes the conversation, child sessions, delegated work, and managed images. It cannot be undone."),
            LocalizationService.GetString("LocDeletePermanently", "Delete permanently"),
            () => _ = ViewModel.DeleteSessionPermanentlyAsync(session));

    internal void ShowDependencyDeleteDialog(ExplorerNode node)
    {
        Owner?.ShowConfirmation(
            "Delete dependency?",
            "It will be removed from the project's dependencies. The folder and its files will not be deleted.",
            "DEPENDENCY",
            node.Name,
            LocalizationService.GetString("LocDeleteDependency", "Delete dependency"),
            () => _ = ViewModel.RemoveProjectDependencyAsync(node),
            size: new Size(480, 320));
    }

    private void ToggleNavigation(object? sender, RoutedEventArgs e) => ToggleSessionsPanel();

    private async void ToggleExplorer(object? sender, RoutedEventArgs e)
    {
        await ToggleExplorerPanelAsync();
    }

    private void ToggleReview(object? sender, RoutedEventArgs e) => ToggleReviewPanel();

    private void TogglePreview(object? sender, RoutedEventArgs e) => ToggleBrowserPanel();

    internal void ClosePreview()
    {
        Layout.PreviewVisible = false;
        BrowserPreview.CloseBrowser();
    }

    internal void EnsureBrowserUsePage(TaskCompletionSource<bool> initialized)
    {
        Layout.PreviewVisible = true;
        BrowserPreview.EnsureBrowserUsePage(initialized);
    }

    private async void ToggleChildSessions(object? sender, RoutedEventArgs e) => await ToggleChildSessionsPanelAsync();

    internal void ToggleGitViewer()
    {
        if (!ViewModel.Git.IsAvailable) return;
        if (Layout.ToggleBottomDrawer(WorkspaceBottomDrawer.Git)) _ = ViewModel.Git.RefreshAsync();
    }

    private void ToggleGit(object? sender, RoutedEventArgs e) => ToggleGitViewer();

    private async void ToggleProviderTrace(object? sender, RoutedEventArgs e) => await ToggleProviderTracePanelAsync();

    private void ToggleToolActivity(object? sender, RoutedEventArgs e) =>
        ToggleToolActivityPanel();

    // Resize handles share the Border.resize-handle style; their Tag names
    // the layout region they resize. Handlers are attached once at the
    // workspace root rather than repeated on every handle.
    private void LayoutResizePressed(object? sender, PointerPressedEventArgs e)
    {
        if (e.Source is not Border handle || !handle.Classes.Contains("resize-handle") ||
            !Enum.TryParse<WorkspaceResizeTarget>(handle.Tag as string, out var target) ||
            !e.GetCurrentPoint(handle).Properties.IsLeftButtonPressed ||
            TopLevel.GetTopLevel(this) is not Window window) return;
        ClampGitViewerHeight();
        _layoutResizeTarget = target;
        _layoutResizeStart = e.GetPosition(window);
        _layoutResizeStartSize = Layout.ResizeStartSize(target);
        e.Pointer.Capture(handle);
        e.Handled = true;
    }

    private void LayoutResizeMoved(object? sender, PointerEventArgs e)
    {
        if (_layoutResizeTarget is not { } target || TopLevel.GetTopLevel(this) is not Window window) return;
        var point = e.GetPosition(window);
        Layout.ApplyResize(target, _layoutResizeStartSize, point.X - _layoutResizeStart.X, point.Y - _layoutResizeStart.Y);
        e.Handled = true;
    }

    private void LayoutResizeReleased(object? sender, PointerReleasedEventArgs e)
    {
        if (_layoutResizeTarget is not { } target) return;
        Layout.CompleteResize(target);
        _layoutResizeTarget = null;
        e.Pointer.Capture(null);
        e.Handled = true;
    }

    private async Task SubmitSessionNameAsync(SessionItem? target, string title)
    {
        if (target is null)
        {
            if (await ViewModel.CreateSessionAsync(title)) ChatArea.FocusComposer();
        }
        else
        {
            await ViewModel.RenameSessionAsync(target, title);
        }
            
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

    private void ShowToolActivity(MessageItem message) =>
        ViewModel.ShowToolActivity(message.TurnId, message.ToolCallId);

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

    private void OpenSettings(object? sender, RoutedEventArgs e) =>
        Owner?.ShowSettings(sender as Control);
    private async void OpenProjectRequested(object? sender, EventArgs e)
    {
        if (Owner is { } owner) await owner.OpenProjectPickerAsync();
    }
    private async void ProjectRequested(ProjectItem project)
    {
        if (Owner is { } owner) await owner.OpenProjectAsync(project);
    }
    private async void ContentRequested(RecentContentItem item)
    {
        if (item.File is { } file)
            await ViewModel.SelectExplorerFileAsync(file);
        else if (item.ChildSession is { } child)
            await ViewModel.SelectChildSessionAsync(child);
        else if (item.Session is { } session)
            await ViewModel.SelectSessionAsync(session);
    }
    private void CloseProjectWindow(object? sender, RoutedEventArgs e) => Owner?.Close();
    private void MinimizeWindow(object? sender, RoutedEventArgs e) => Owner?.MinimizeWindow();
    private void ToggleWindowMaximized(object? sender, RoutedEventArgs e) => Owner?.ToggleWindowMaximized();
    private void ToggleFullScreen(object? sender, RoutedEventArgs e) => Owner?.ToggleFullScreen();
    private void TitleBarPressed(object? sender, PointerPressedEventArgs e) => Owner?.TitleBarPressed(sender, e);
    private void TitleBarMoved(object? sender, PointerEventArgs e) => Owner?.TitleBarMoved(sender, e);
    private void TitleBarReleased(object? sender, PointerReleasedEventArgs e) => Owner?.TitleBarReleased(sender, e);
    private void TitleBarDoubleTapped(object? sender, TappedEventArgs e) => Owner?.TitleBarDoubleTapped(sender, e);
    internal void SetTrafficLightFocus(bool isActive)
    {
        ProjectCloseLight.IsWindowActive = isActive;
        ProjectMinimizeLight.IsWindowActive = isActive;
        ProjectMaximizeLight.IsWindowActive = isActive;
    }

    private void WindowsCloseEntered(object? sender, PointerEventArgs e) => SetWindowsCloseIconState(sender, "hover");
    private void WindowsCloseExited(object? sender, PointerEventArgs e) => SetWindowsCloseIconState(sender, "normal");
    private void WindowsClosePressed(object? sender, PointerPressedEventArgs e) => SetWindowsCloseIconState(sender, "press");
    private void WindowsCloseReleased(object? sender, PointerReleasedEventArgs e) => SetWindowsCloseIconState(sender, "hover");

    private void SetWindowsCloseIconState(object? sender, string state)
    {
        if (sender is not Button button || button.GetVisualDescendants().OfType<SvgControl>().FirstOrDefault() is not { } icon) return;
        if (state is "hover" or "press")
        {
            // The white asset has a literal white stroke; do not apply the
            // normal theme CSS afterward or it would recolor it gray again.
            icon.Path = "/Assets/icons/windows-close-white.svg";
            SvgControl.SetCss(icon, string.Empty);
        }
        else
        {
            icon.Path = "/Assets/icons/windows-close.svg";
            SvgControl.SetCss(icon, this.FindResource("IconSvgCss") as string);
        }
    }
}
