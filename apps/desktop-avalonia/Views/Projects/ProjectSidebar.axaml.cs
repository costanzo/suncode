using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.VisualTree;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.Projects;

public sealed partial class ProjectSidebar : UserControl
{
    public ProjectSidebar()
    {
        InitializeComponent();
        SessionList.AddHandler(
            InputElement.PointerPressedEvent,
            SessionListPointerPressed,
            RoutingStrategies.Tunnel | RoutingStrategies.Bubble,
            handledEventsToo: true);
    }

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;
    private ProjectWorkspace? Workspace => this.FindAncestorOfType<ProjectWorkspace>();

    private async void SessionSelectionChanged(object? sender, SelectionChangedEventArgs e)
    {
        if (e.AddedItems.OfType<SessionItem>().FirstOrDefault() is not { } session)
            return;

        await SelectSessionFromInputAsync(session, "selection_changed");
    }

    private void SessionListPointerPressed(object? sender, PointerPressedEventArgs e)
    {
        var source = e.Source as Control;
        var session = source?.DataContext as SessionItem
            ?? source?.FindAncestorOfType<ListBoxItem>()?.DataContext as SessionItem;
        if (e.Handled || session is null) return;

        DiagnosticLog.Info("session.pointer.route", $"fallback_select session={session.SessionId}");
        _ = SelectSessionFromInputAsync(session, "pointer_route");
    }

    private async Task SelectSessionFromInputAsync(SessionItem session, string source)
    {
        DiagnosticLog.Debug("session.input", $"begin source={source} session={session.SessionId}");
        try
        {
            await ViewModel.SelectSessionAsync(session);
            DiagnosticLog.Debug("session.input", $"end source={source} session={session.SessionId}");
        }
        catch (Exception exception)
        {
            DiagnosticLog.Error("session.input", $"failed source={source} session={session.SessionId} type={exception.GetType().Name} message={exception.Message}");
        }
    }

    private void CreateSession(object? sender, RoutedEventArgs e) =>
        Workspace?.ShowSessionDialog("New session", string.Empty, "Create", null);

    private void RenameSessionItem(object? sender, RoutedEventArgs e)
    {
        if (sender is MenuItem { CommandParameter: SessionItem session })
            Workspace?.ShowSessionDialog("Rename session", session.DisplayTitle, "Save", session);
    }

    private async void ArchiveSessionItem(object? sender, RoutedEventArgs e)
    {
        if (sender is MenuItem { CommandParameter: SessionItem session })
            await ViewModel.ArchiveSessionAsync(session);
    }

    private async void PinSessionItem(object? sender, RoutedEventArgs e)
    {
        if (sender is MenuItem { CommandParameter: SessionItem session })
            await ViewModel.SetSessionPinnedAsync(session, true);
    }

    private async void UnpinSessionItem(object? sender, RoutedEventArgs e)
    {
        if (sender is MenuItem { CommandParameter: SessionItem session })
            await ViewModel.SetSessionPinnedAsync(session, false);
    }

    private async void NavigationPointerExited(object? sender, PointerEventArgs e)
    {
        if (ViewModel.NavigationPinned) return;
        await Task.Delay(420);
        if (!ViewModel.NavigationPinned) ViewModel.NavigationVisible = false;
    }
}
