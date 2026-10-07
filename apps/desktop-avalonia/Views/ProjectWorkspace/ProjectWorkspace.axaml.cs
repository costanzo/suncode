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

namespace SunCode.Desktop.Views.ProjectWorkspace;

public sealed partial class ProjectWorkspace : UserControl
{
    private SessionItem? _sessionDialogTarget;
    private CheckpointItem? _pendingCheckpoint;
    private ExplorerNode? _pendingDependencyDeletion;
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
        BrowserPreview.CloseRequested += ClosePreview;
        AddHandler(PointerPressedEvent, LayoutResizePressed);
        AddHandler(PointerMovedEvent, LayoutResizeMoved);
        AddHandler(PointerReleasedEvent, LayoutResizeReleased);
    }

    private WorkspaceWindow? Owner => TopLevel.GetTopLevel(this) as WorkspaceWindow;
    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;
    private WorkspaceLayoutViewModel Layout => ViewModel.Layout;

    internal void ScrollConversationToEndForSessionEntry() =>
        ChatArea.ScrollConversationToEndForSessionEntry();

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
        _pendingDependencyDeletion = node;
        DependencyDeleteName.Text = node.Name;
        DependencyDeleteDialogModal.IsOpen = true;
    }

    private void ToggleNavigation(object? sender, RoutedEventArgs e) => Layout.ToggleSessions();

    private async void ToggleExplorer(object? sender, RoutedEventArgs e)
    {
        if (Layout.ToggleExplorer()) await ViewModel.Explorer.LoadRootsAsync();
    }

    private void ToggleReview(object? sender, RoutedEventArgs e)
    {
        if (Layout.ToggleReview()) BrowserPreview.CloseBrowser();
    }

    private void TogglePreview(object? sender, RoutedEventArgs e)
    {
        Layout.PreviewVisible = !Layout.PreviewVisible;
        if (!Layout.PreviewVisible) BrowserPreview.CloseBrowser();
    }

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

    private void ToggleChildSessions(object? sender, RoutedEventArgs e)
    {
        if (Layout.ToggleChildSessions()) BrowserPreview.CloseBrowser();
        if (Layout.ChildSessionsVisible) _ = ViewModel.LoadChildSessionsAsync();
    }

    internal void ToggleGitViewer()
    {
        if (Layout.ToggleBottomDrawer(WorkspaceBottomDrawer.Git)) _ = ViewModel.Git.RefreshAsync();
    }

    private void ToggleGit(object? sender, RoutedEventArgs e) => ToggleGitViewer();

    private void ToggleProviderTrace(object? sender, RoutedEventArgs e)
    {
        if (Layout.ToggleBottomDrawer(WorkspaceBottomDrawer.ProviderTrace)) _ = ViewModel.ProviderTrace.RefreshAsync();
    }

    private void ToggleToolActivity(object? sender, RoutedEventArgs e) =>
        Layout.ToggleBottomDrawer(WorkspaceBottomDrawer.ToolActivity);

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
        if (target is null)
        {
            if (await ViewModel.CreateSessionAsync(title)) ChatArea.FocusComposer();
        }
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
