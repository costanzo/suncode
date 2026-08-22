using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.Chat;

public sealed partial class ChatInput : UserControl
{
    public ChatInput()
    {
        InitializeComponent();
        ComposerInput.AddHandler(KeyDownEvent, ComposerKeyDown, RoutingStrategies.Tunnel);
    }

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;

    private async void SubmitTurn(object? sender, RoutedEventArgs e) =>
        await ViewModel.SubmitTurnAsync();

    private async void CancelTurn(object? sender, RoutedEventArgs e) =>
        await ViewModel.CancelTurnAsync();

    private async void ComposerKeyDown(object? sender, KeyEventArgs e)
    {
        if (e.Key != Key.Enter || e.KeyModifiers.HasFlag(KeyModifiers.Shift)) return;
        if (!ViewModel.CanSubmit) return;

        e.Handled = true;
        ViewModel.ComposerText = ComposerInput.Text ?? string.Empty;
        await ViewModel.SubmitTurnAsync();
    }
}
