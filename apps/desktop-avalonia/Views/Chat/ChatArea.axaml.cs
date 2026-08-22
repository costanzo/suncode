using Avalonia.Controls;
using Avalonia.Input.Platform;
using Avalonia.Interactivity;
using Avalonia.Threading;
using Avalonia.VisualTree;
using SvgControl = Avalonia.Svg.Skia.Svg;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.Chat;

public sealed partial class ChatArea : UserControl
{
    private bool _scrollPending;

    public ChatArea()
    {
        InitializeComponent();
    }

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;

    internal void ScrollConversationToEnd()
    {
        if (_scrollPending) return;
        _scrollPending = true;
        Dispatcher.UIThread.Post(() =>
        {
            _scrollPending = false;
            if (ViewModel.Messages.Count > 0) ConversationList.ScrollIntoView(ViewModel.Messages.Count - 1);
        }, DispatcherPriority.Background);
    }

    private async void RetrySession(object? sender, RoutedEventArgs e)
    {
        if (ViewModel.SelectedSession is { } session) await ViewModel.SelectSessionAsync(session);
    }

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
}
