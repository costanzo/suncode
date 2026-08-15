using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views;

public sealed partial class ProjectWorkspace : UserControl
{
    private SessionItem? _sessionDialogTarget;
    private CheckpointItem? _pendingCheckpoint;

    public ProjectWorkspace()
    {
        InitializeComponent();
    }

    private MainWindow? Owner => TopLevel.GetTopLevel(this) as MainWindow;
    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;

    internal void ScrollConversationToEnd() =>
        ChatArea.ScrollConversationToEnd();

    internal void ClampGitViewerHeight()
    {
        if (GitViewer.IsVisible) GitViewer.ClampHeightToWindow();
    }

    internal void SetFullScreenChrome(bool fullScreen)
    {
        ProjectChrome.Padding = fullScreen ? new Thickness(0) : new Thickness(4, 6, 4, 6);
        ProjectChrome.CornerRadius = fullScreen ? new CornerRadius(0) : new CornerRadius(12);
    }

    internal bool HandleEscape()
    {
        if (SessionDialogOverlay.IsVisible)
        {
            HideSessionDialog();
            return true;
        }

        if (UndoDialogOverlay.IsVisible)
        {
            HideUndoDialog();
            return true;
        }

        return false;
    }

    internal void ShowSessionDialog(string title, string value, string acceptText, SessionItem? target)
    {
        _sessionDialogTarget = target;
        SessionDialogTitle.Text = title;
        SessionDialogSubmitButton.Content = acceptText;
        SessionTitleInput.Text = value;
        SessionDialogOverlay.IsVisible = true;
        SessionDialogSubmitButton.IsEnabled = !string.IsNullOrWhiteSpace(value);
        SessionTitleInput.Focus();
        SessionTitleInput.SelectAll();
    }

    internal void ShowUndoDialog(CheckpointItem checkpoint)
    {
        _pendingCheckpoint = checkpoint;
        UndoPathsText.Text = checkpoint.PathsText;
        UndoDialogOverlay.IsVisible = true;
    }

    private void ToggleNavigation(object? sender, RoutedEventArgs e) =>
        ViewModel.NavigationVisible = !ViewModel.NavigationVisible;

    private void ToggleReview(object? sender, RoutedEventArgs e) =>
        ViewModel.ReviewVisible = !ViewModel.ReviewVisible;

    private void ToggleGit(object? sender, RoutedEventArgs e)
    {
        ViewModel.GitVisible = !ViewModel.GitVisible;
        if (ViewModel.GitVisible) _ = ViewModel.RefreshGitAsync();
    }

    private void SessionTitleChanged(object? sender, TextChangedEventArgs e) =>
        SessionDialogSubmitButton.IsEnabled = !string.IsNullOrWhiteSpace(SessionTitleInput.Text);

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

    private void SessionOverlayPointerPressed(object? sender, PointerPressedEventArgs e)
    {
        if (e.Source == sender) HideSessionDialog();
    }

    private void HideSessionDialog()
    {
        SessionDialogOverlay.IsVisible = false;
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
        UndoDialogOverlay.IsVisible = false;
        _pendingCheckpoint = null;
    }

    private void OpenSettings(object? sender, RoutedEventArgs e) => Owner?.ShowSettings();
    private void CloseProjectWindow(object? sender, RoutedEventArgs e) => Owner?.Close();
    private void MinimizeWindow(object? sender, RoutedEventArgs e) => Owner?.MinimizeWindow();
    private void ToggleMaximized(object? sender, RoutedEventArgs e) => Owner?.ToggleMaximized();
    private void TitleBarPressed(object? sender, PointerPressedEventArgs e) => Owner?.TitleBarPressed(sender, e);
    private void TitleBarMoved(object? sender, PointerEventArgs e) => Owner?.TitleBarMoved(sender, e);
    private void TitleBarReleased(object? sender, PointerReleasedEventArgs e) => Owner?.TitleBarReleased(sender, e);
    private void TitleBarDoubleTapped(object? sender, TappedEventArgs e) => Owner?.TitleBarDoubleTapped(sender, e);
    private void TrafficLightEntered(object? sender, PointerEventArgs e) => MainWindow.SetTrafficLightState(sender, "hover");
    private void TrafficLightExited(object? sender, PointerEventArgs e) => MainWindow.SetTrafficLightState(sender, "normal");
    private void TrafficLightPressed(object? sender, PointerPressedEventArgs e) => MainWindow.SetTrafficLightState(sender, "press");
    private void TrafficLightReleased(object? sender, PointerReleasedEventArgs e) => MainWindow.SetTrafficLightState(sender, "hover");
}
