using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Threading;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.ProjectWorkspace.Terminal;

public sealed partial class TerminalPanel : UserControl
{
    private bool _sessionOpenQueued;

    public TerminalPanel() => InitializeComponent();

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;

    private void SessionSelected(object? sender, SelectionChangedEventArgs e)
    {
        if (e.AddedItems.OfType<TerminalSessionViewModel>().FirstOrDefault() is { } session)
            ViewModel.Terminal.SelectSession(session);
    }

    private void SessionSelectedButton(object? sender, RoutedEventArgs e)
    {
        if ((sender as Control)?.DataContext is TerminalSessionViewModel session)
            ViewModel.Terminal.SelectSession(session);
    }

    private void NewSession(object? sender, RoutedEventArgs e)
    {
        if (_sessionOpenQueued || !ViewModel.Terminal.CanOpen) return;
        _sessionOpenQueued = true;
        Dispatcher.UIThread.Post(() =>
        {
            _sessionOpenQueued = false;
            if (VisualRoot is not null && ViewModel.Terminal.CanOpen) ViewModel.Terminal.OpenSession();
        }, DispatcherPriority.Background);
    }

    private void CloseSession(object? sender, RoutedEventArgs e)
    {
        if ((sender as Control)?.DataContext is TerminalSessionViewModel session)
            ViewModel.Terminal.CloseSession(session);
    }

    private void RestartSession(object? sender, RoutedEventArgs e)
    {
        ViewModel.Terminal.SelectedSession?.RequestRestart();
    }

    private void HideTerminal(object? sender, RoutedEventArgs e) => ViewModel.Layout.TerminalVisible = false;
}
