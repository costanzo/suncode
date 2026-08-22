using Avalonia.Controls;
using Avalonia.Input.Platform;
using Avalonia.Interactivity;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.Review;

public sealed partial class ProviderTraceViewer : UserControl
{
    public ProviderTraceViewer()
    {
        InitializeComponent();
    }

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;

    internal void ClampHeightToWindow()
    {
        if (TopLevel.GetTopLevel(this) is not Window window) return;
        Height = Math.Clamp(Height, 260, Math.Max(260, window.Bounds.Height - 300));
    }

    private async void RefreshTrace(object? sender, RoutedEventArgs e) => await ViewModel.RefreshProviderTracesAsync();

    private async void TraceSelected(object? sender, SelectionChangedEventArgs e)
    {
        if (e.AddedItems.OfType<ProviderTraceItem>().FirstOrDefault() is { } trace)
            await ViewModel.LoadProviderTraceAsync(trace);
        else if (e.AddedItems.OfType<ProviderTraceTurnItem>().Any())
            ViewModel.SelectProviderTraceTurn();
    }

    private void TraceFilterChanged(object? sender, TextChangedEventArgs e)
    {
        if (sender is TextBox field) ViewModel.SetProviderTraceFilter(field.Text ?? string.Empty);
    }

    private async void CopyTrace(object? sender, RoutedEventArgs e)
    {
        if (TopLevel.GetTopLevel(this)?.Clipboard is not { } clipboard || ViewModel.SelectedProviderTraceDetails is not { } trace) return;
        await clipboard.SetTextAsync(string.Join(Environment.NewLine, new[]
        {
            trace.Title,
            trace.TurnText,
            $"SunCode call {trace.ExchangeId}",
            $"Provider request {trace.ProviderRequestId}",
            $"Provider response {trace.ProviderResponseId}",
            trace.UsageSummary,
            $"Cache hit {trace.CacheHitRateText}",
            $"Duration {trace.DurationText}",
            "",
            "Messages",
            string.Join(Environment.NewLine + Environment.NewLine, trace.Messages.Select(message => $"[{message.RoleText}] {message.Content}")),
            "",
            "Tools",
            string.Join(Environment.NewLine + Environment.NewLine, trace.Tools.Select(tool => $"{tool.Name} [{tool.StateText}]{Environment.NewLine}{tool.Request}{Environment.NewLine}{tool.Result}")),
            "",
            "Model Request",
            trace.InputText,
            "",
            "Model Response",
            trace.OutputText,
            "",
            "Error",
            trace.ErrorText,
        }));
    }

    private void CloseTrace(object? sender, RoutedEventArgs e) => ViewModel.ProviderTraceVisible = false;
}
