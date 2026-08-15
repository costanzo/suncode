using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Input.Platform;
using Avalonia.Interactivity;
using Avalonia.Threading;
using Avalonia.VisualTree;
using SvgControl = Avalonia.Svg.Skia.Svg;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views;

public sealed partial class ChatArea : UserControl
{
    public ChatArea()
    {
        InitializeComponent();
        ComposerInput.AddHandler(KeyDownEvent, ComposerKeyDown, RoutingStrategies.Tunnel);
    }

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;

    internal void ScrollConversationToEnd() =>
        Dispatcher.UIThread.Post(ConversationScroll.ScrollToEnd, DispatcherPriority.Background);

    private async void SubmitTurn(object? sender, RoutedEventArgs e) => await ViewModel.SubmitTurnAsync();
    private async void CancelTurn(object? sender, RoutedEventArgs e) => await ViewModel.CancelTurnAsync();

    private async void CopyMessage(object? sender, RoutedEventArgs e)
    {
        if ((sender as Control)?.DataContext is not MessageItem message ||
            TopLevel.GetTopLevel(this)?.Clipboard is not { } clipboard) return;

        await clipboard.SetTextAsync(message.Text);
        if (sender is not Button button) return;

        ToolTip.SetTip(button, "Copied");
        if (button.GetVisualDescendants().OfType<SvgControl>().FirstOrDefault() is { } icon)
            SvgControl.SetCss(icon, this.FindResource("SuccessSvgCss") as string);
        await Task.Delay(1400);
        ToolTip.SetTip(button, "Copy response");
        if (button.GetVisualDescendants().OfType<SvgControl>().FirstOrDefault() is { } resetIcon)
            SvgControl.SetCss(resetIcon, this.FindResource("IconSvgCss") as string);
    }

    private async void ComposerKeyDown(object? sender, KeyEventArgs e)
    {
        if (e.Key != Key.Enter || e.KeyModifiers.HasFlag(KeyModifiers.Shift)) return;
        e.Handled = true;
        ViewModel.ComposerText = ComposerInput.Text ?? string.Empty;
        await ViewModel.SubmitTurnAsync();
    }
}
