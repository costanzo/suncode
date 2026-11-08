using System.Reflection;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Markup.Xaml.Styling;
using Avalonia.Styling;
using Avalonia.Themes.Fluent;
using Avalonia.Threading;
using Avalonia.VisualTree;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Views.ProjectWorkspace.Review;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Tests;

[Collection("Avalonia UI")]
public sealed class ProviderFailureNavigationViewTests(AvaloniaUiFixture ui)
{
    [Theory]
    [InlineData(1100, false, "en-US")]
    [InlineData(1100, true, "zh-CN")]
    [InlineData(620, false, "zh-CN")]
    [InlineData(620, true, "en-US")]
    public Task ActivatingFailureRevealsTheSelectedCallAndScrollsToItsError(int width, bool dark, string locale) => ui.Run(() => WithResources(locale, async () =>
    {
        using var desktop = new DesktopViewModel();
        var failed = ProviderFailureNavigationTests.Call("failed");
        var calls = Enumerable.Range(1, 15).Select(index => ProviderFailureNavigationTests.Call($"old-{index}", turnId: $"old-turn-{index}")).Append(failed).ToArray();
        var host = ProviderFailureNavigationTests.Host(calls);
        var detail = ProviderFailureNavigationTests.Details(failed) with
        {
            Messages = Enumerable.Range(1, 12).Select(index => new SessionCallMessage(
                $"message-{index}", "session", "turn", "failed", "user",
                new AgentMessage("user", [new("text", string.Join('\n', Enumerable.Repeat("Long request context before the provider error.", 8)))], null, null),
                "2026-11-08T10:00:00Z")).ToArray()
        };
        host.ProviderExchangeLoader = (_, id) => Task.FromResult(id == "failed" ? detail : ProviderFailureNavigationTests.Details(calls.Single(item => item.ExchangeId == id)));
        var trace = BindTrace(desktop, host);
        var sidebar = new AgentSidebar { DataContext = desktop, Width = 312 };
        var viewer = new ProviderTraceViewer { DataContext = desktop };
        var grid = new Grid { RowDefinitions = new("*,300") };
        grid.Children.Add(sidebar);
        Grid.SetRow(viewer, 1);
        grid.Children.Add(viewer);
        var window = new Window { Width = width, Height = 820, Content = grid, RequestedThemeVariant = dark ? ThemeVariant.Dark : ThemeVariant.Light };
        try
        {
            ProviderFailureNavigationTests.ApplyFailure(desktop);
            trace.SetFilter("hidden-destination");
            window.Show(); Flush(window);
            var summary = sidebar.FindControl<Button>("FailureSummaryButton")!;
            Assert.True(summary.IsEffectivelyEnabled);
            Assert.Contains(sidebar.GetVisualDescendants().OfType<TextBlock>(), text => text.Text == "Connection reset by peer");
            summary.Focus(NavigationMethod.Tab);
            window.KeyPress(Key.Enter, RawInputModifiers.None, PhysicalKey.Enter, null);
            window.KeyRelease(Key.Enter, RawInputModifiers.None, PhysicalKey.Enter, null);
            Flush(window);
            Assert.True(desktop.Layout.ProviderTraceVisible);
            Assert.Equal("ready", trace.FailureNavigationState);
            Assert.Equal("failed", trace.SelectedTraceDetails?.ExchangeId);
            Assert.Equal(string.Empty, viewer.GetVisualDescendants().OfType<TextBox>().Single().Text);
            var tree = viewer.FindControl<TreeView>("TraceTree")!;
            var selected = Assert.Single(tree.GetVisualDescendants().OfType<TreeViewItem>(), item => item.IsSelected);
            Assert.Equal("failed", Assert.IsType<SunCode.Desktop.Models.ProviderTraceItem>(selected.DataContext).ExchangeId);
            var heading = viewer.FindControl<Border>("ProviderErrorHeading")!;
            var scroll = viewer.FindControl<ScrollViewer>("TraceDetailScroll")!;
            Assert.True(heading.IsFocused);
            Assert.Null(heading.FocusAdorner);
            Assert.Equal(new Thickness(1), heading.BorderThickness);
            Assert.Equal(new CornerRadius(6), heading.CornerRadius);
            Assert.Equal(Avalonia.Media.Colors.Transparent, Assert.IsAssignableFrom<Avalonia.Media.ISolidColorBrush>(heading.BorderBrush).Color);
            summary.Focus(NavigationMethod.Tab);
            heading.Focus(NavigationMethod.Tab);
            Flush(window);
            Assert.True(heading.TryFindResource("AccentBrush", window.ActualThemeVariant, out var accent));
            Assert.Equal(accent, heading.BorderBrush);
            Assert.True(scroll.Offset.Y > 0);
            var position = heading.TranslatePoint(default, scroll)!.Value;
            Assert.InRange(position.Y, -1, scroll.Viewport.Height);
            Assert.InRange(position.X, 0, scroll.Viewport.Width);
            Assert.Equal(locale == "zh-CN" ? "提供商错误" : "Provider error", viewer.FindControl<TextBlock>("ProviderErrorLabel")!.Text);
            Assert.InRange(position.Y, -1, 9);
            Assert.InRange(viewer.FindControl<SelectableTextBlock>("ProviderErrorText")!.TranslatePoint(default, scroll)!.Value.Y, 0, scroll.Viewport.Height);
            Assert.Equal(width <= 800 ? 3 : 6, viewer.FindControl<Grid>("TraceMetrics")!.ColumnDefinitions.Count);
            Assert.InRange(summary.Bounds.Width, 1, sidebar.Bounds.Width);
            var clickPoint = summary.TranslatePoint(new Point(summary.Bounds.Width / 2, summary.Bounds.Height / 2), window)!.Value;
            window.MouseDown(clickPoint, MouseButton.Left, RawInputModifiers.None);
            window.MouseUp(clickPoint, MouseButton.Left, RawInputModifiers.None);
            Flush(window);
            Assert.True(desktop.Layout.ProviderTraceVisible);
            Assert.Equal("failed", trace.SelectedTraceDetails?.ExchangeId);
            Assert.Equal(Avalonia.Media.Colors.Transparent, Assert.IsAssignableFrom<Avalonia.Media.ISolidColorBrush>(heading.BorderBrush).Color);
            trace.SetFilter("old-1");
            Flush(window);
            Assert.Equal("old-1", trace.SelectedTraceDetails?.ExchangeId);
            Assert.True(trace.ShowSelectedTraceOverview);
            await Task.CompletedTask;
        }
        finally { window.Close(); }
    }));

    [Fact]
    public Task LoadingAndReadFailureExposeRecoveryWithoutSubmittingAnotherTurn() => ui.Run(() => WithResources("en-US", async () =>
    {
        using var desktop = new DesktopViewModel();
        var failed = ProviderFailureNavigationTests.Call("failed");
        var host = ProviderFailureNavigationTests.Host(failed);
        var pending = new TaskCompletionSource<ProviderExchangeDetails>();
        host.ProviderExchangeLoader = (_, _) => pending.Task;
        var trace = BindTrace(desktop, host);
        var sidebar = new AgentSidebar { DataContext = desktop, Width = 312 };
        var viewer = new ProviderTraceViewer { DataContext = desktop };
        var grid = new Grid { ColumnDefinitions = new("312,*") };
        grid.Children.Add(sidebar); Grid.SetColumn(viewer, 1); grid.Children.Add(viewer);
        var window = new Window { Width = 1000, Height = 600, Content = grid };
        try
        {
            ProviderFailureNavigationTests.ApplyFailure(desktop);
            window.Show(); Flush(window);
            var summary = sidebar.FindControl<Button>("FailureSummaryButton")!;
            var navigation = desktop.OpenFailureDetailsAsync();
            Flush(window);
            Assert.False(summary.IsEffectivelyEnabled);
            Assert.True(viewer.FindControl<StackPanel>("FailureNavigationFeedback")!.IsEffectivelyVisible);
            pending.SetException(new IOException("Read failed"));
            await navigation; Flush(window);
            Assert.True(summary.IsEffectivelyEnabled);
            Assert.True(trace.CanRetryFailureNavigation);
            Assert.True(viewer.FindControl<StackPanel>("FailureNavigationFeedback")!.IsFocused);
            host.ProviderExchangeLoader = (_, _) => Task.FromResult(ProviderFailureNavigationTests.Details(failed));
            var retry = viewer.GetVisualDescendants().OfType<Button>().Single(button => Equals(button.Content, "Retry loading details"));
            retry.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
            Flush(window);
            Assert.Equal("ready", trace.FailureNavigationState);
            Assert.True(viewer.FindControl<Border>("ProviderErrorHeading")!.IsFocused);
            Assert.True(desktop.HasFailedTurn);
            Assert.Equal("turn", desktop.LastTurnId);
        }
        finally { window.Close(); }
    }));

    // Keep SDK-free UI checks on the normal Desktop bindings and click handlers.
    internal static ProviderTraceViewModel BindTrace(DesktopViewModel desktop, FakeViewModelHost host)
    {
        var trace = new ProviderTraceViewModel(host);
        typeof(DesktopViewModel).GetField("<ProviderTrace>k__BackingField", BindingFlags.Instance | BindingFlags.NonPublic)!.SetValue(desktop, trace);
        trace.Refreshed += desktop.RefreshReviewFailure;
        return trace;
    }

    internal static void Flush(Window window)
    {
        Dispatcher.UIThread.RunJobs(); window.UpdateLayout(); Dispatcher.UIThread.RunJobs(); window.UpdateLayout();
    }

    private static async Task WithResources(string locale, Func<Task> action)
    {
        var app = Application.Current!;
        var theme = new FluentTheme();
        var foundation = new StyleInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/FoundationStyles.axaml") };
        var controls = new StyleInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/ControlStyles.axaml") };
        var resources = new ResourceInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/ThemeResources.axaml") };
        var language = LocalizationService.CreateDictionary(locale);
        app.Styles.Add(theme); app.Styles.Add(foundation); app.Styles.Add(controls);
        app.Resources.MergedDictionaries.Add(resources); app.Resources.MergedDictionaries.Add(language);
        try { await action(); }
        finally
        {
            app.Resources.MergedDictionaries.Remove(language); app.Resources.MergedDictionaries.Remove(resources);
            app.Styles.Remove(controls); app.Styles.Remove(foundation); app.Styles.Remove(theme);
        }
    }
}
