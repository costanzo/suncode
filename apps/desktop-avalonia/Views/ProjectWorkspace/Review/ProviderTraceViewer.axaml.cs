using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Threading;
using Avalonia.VisualTree;
using Avalonia.Input.Platform;
using Avalonia.Interactivity;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.ProjectWorkspace.Review;

public sealed partial class ProviderTraceViewer : UserControl
{
    private ProviderTraceViewModel? _trace;
    private bool? _compactMetrics;
    public ProviderTraceViewer()
    {
        InitializeComponent();
        DataContextChanged += (_, _) => ObserveTrace();
        SizeChanged += (_, _) => UpdateMetricsLayout();
    }

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;

    protected override void OnAttachedToVisualTree(VisualTreeAttachmentEventArgs e)
    {
        base.OnAttachedToVisualTree(e);
        ObserveTrace();
    }

    protected override void OnDetachedFromVisualTree(VisualTreeAttachmentEventArgs e)
    {
        if (_trace is not null) _trace.FailureFocusRequested -= FocusFailure;
        _trace = null;
        base.OnDetachedFromVisualTree(e);
    }

    private void ObserveTrace()
    {
        if (_trace is not null) _trace.FailureFocusRequested -= FocusFailure;
        _trace = (DataContext as DesktopViewModel)?.ProviderTrace;
        if (_trace is not null) _trace.FailureFocusRequested += FocusFailure;
    }

    private void FocusFailure()
    {
        var trace = _trace;
        var exchangeId = trace?.SelectedTrace?.ExchangeId;
        Dispatcher.UIThread.Post(() =>
        {
            if (trace is null || !ReferenceEquals(_trace, trace) || !IsEffectivelyVisible
                || trace.SelectedTrace?.ExchangeId != exchangeId || trace.FailureNavigationState is not ("ready" or "missing" or "error")) return;
            if (trace.ShowFailureNavigationFeedback)
            {
                FailureNavigationFeedback.Focus();
                return;
            }
            if (trace.SelectedTraceDetails?.HasError != true) return;
            UpdateLayout();
            var row = TraceTree.GetVisualDescendants().OfType<TreeViewItem>()
                .FirstOrDefault(item => item.DataContext is ProviderTraceItem call && call.ExchangeId == exchangeId);
            row?.BringIntoView();
            ProviderErrorHeading.Focus();
            ProviderErrorHeading.BringIntoView();
            TraceDetailScroll.UpdateLayout();
            if (ProviderErrorHeading.TranslatePoint(default, TraceDetailScroll) is { } position)
                TraceDetailScroll.Offset = new Vector(0, Math.Max(0, TraceDetailScroll.Offset.Y + position.Y - 8));
        }, DispatcherPriority.Loaded);
    }

    private void UpdateMetricsLayout()
    {
        var compact = Bounds.Width <= 800;
        if (_compactMetrics == compact) return;
        _compactMetrics = compact;
        var columns = compact ? 3 : 6;
        TraceMetrics.ColumnDefinitions = new(compact ? "*,*,*" : "*,*,*,*,*,*");
        var rowHeight = this.FindResource("ProviderTraceMetricRowHeight") is double value ? value : 58;
        TraceMetrics.RowDefinitions = new RowDefinitions { new RowDefinition(new GridLength(rowHeight)) };
        if (compact) TraceMetrics.RowDefinitions.Add(new RowDefinition(new GridLength(rowHeight)));
        for (var index = 0; index < TraceMetrics.Children.Count; index++)
        {
            Grid.SetColumn(TraceMetrics.Children[index], index % columns);
            Grid.SetRow(TraceMetrics.Children[index], index / columns);
        }
    }

    internal void ClampHeightToWindow()
    {
        if (TopLevel.GetTopLevel(this) is not Window window) return;
        Height = Math.Clamp(Height, 260, Math.Max(260, window.Bounds.Height - 300));
    }

    private async void RefreshTrace(object? sender, RoutedEventArgs e) => await ViewModel.ProviderTrace.RefreshAsync();

    private async void TraceSelected(object? sender, SelectionChangedEventArgs e)
    {
        if (e.AddedItems.OfType<ProviderTraceItem>().FirstOrDefault() is { } trace)
        {
            var model = ViewModel.ProviderTrace;
            if (!ReferenceEquals(model.SelectedTrace, trace) || !model.IsRevealingFailure && model.SelectedTraceDetails is null)
                await model.LoadAsync(trace);
        }
        else if (e.AddedItems.OfType<ProviderTraceTurnItem>().Any())
            ViewModel.ProviderTrace.SelectTurn();
    }

    private async void RetryFailureDetails(object? sender, RoutedEventArgs e) =>
        await ViewModel.ProviderTrace.RetryFailureNavigationAsync();

    private async void CopyTrace(object? sender, RoutedEventArgs e)
    {
        if (TopLevel.GetTopLevel(this)?.Clipboard is not { } clipboard || ViewModel.ProviderTrace.SelectedTraceDetails is not { } trace) return;
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
            "Model Response",
            trace.OutputText,
            "",
            "Error",
            trace.ErrorText,
        }));
    }

    private void CloseTrace(object? sender, RoutedEventArgs e) => ViewModel.Layout.ProviderTraceVisible = false;
}
