using Avalonia;
using Avalonia.Controls;
using Avalonia.Markup.Xaml.Styling;
using Avalonia.Themes.Fluent;
using Avalonia.Threading;
using Avalonia.VisualTree;
using Microsoft.Extensions.Time.Testing;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Views.ProjectWorkspace.Chat;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Tests;

[Collection("Avalonia UI")]
public sealed class ProviderRetryCountdownTests(AvaloniaUiFixture ui)
{
    [Fact]
    public Task RemainingTimeAdvancesAcrossMissedTicksAndWaitsAtZero() => ui.Run(() =>
    {
        var clock = new FakeTimeProvider();
        using var model = new DesktopViewModel(timeProvider: clock);
        StartTurn(model);
        Retry(model, 58000);
        Assert.Contains("58s", model.ProviderRetryText);
        clock.Advance(TimeSpan.FromSeconds(1));
        model.RefreshProviderRetryStatus();
        Assert.Contains("57s", model.ProviderRetryText);
        clock.Advance(TimeSpan.FromSeconds(16));
        model.RefreshProviderRetryStatus();
        Assert.Contains("41s", model.ProviderRetryText);
        clock.Advance(TimeSpan.FromMinutes(1));
        model.RefreshProviderRetryStatus();
        Assert.Equal("Waiting for retry to start · retry 1 of 2", model.ProviderRetryText);
        Assert.True(model.IsProviderRetrying);
        Assert.False(model.IsTurnThinking);
        Assert.False(model.IsTurnIndicatorDots);
    });

    [Fact]
    public Task RemainingSecondsRoundUpAndANewScheduleReplacesTheOldWait() => ui.Run(() =>
    {
        var clock = new FakeTimeProvider();
        using var model = new DesktopViewModel(timeProvider: clock);
        StartTurn(model);
        Retry(model, 1250);
        Assert.Contains("2s", model.ProviderRetryText);
        clock.Advance(TimeSpan.FromMilliseconds(250));
        model.RefreshProviderRetryStatus();
        Assert.Contains("1s", model.ProviderRetryText);
        Retry(model, 58000, attempt: 2);
        clock.Advance(TimeSpan.FromSeconds(1));
        model.RefreshProviderRetryStatus();
        Assert.Equal("Retrying request in 57s · retry 2 of 2", model.ProviderRetryText);
    });

    [Theory]
    [InlineData("provider.exchange.started", "")]
    [InlineData("turn.state", "completed")]
    [InlineData("turn.state", "cancelled")]
    [InlineData("turn.state", "failed")]
    [InlineData("turn.state", "interrupted")]
    public Task RequestAndTerminalEventsClearTheCountdownPermanently(string type, string state) => ui.Run(() =>
    {
        var clock = new FakeTimeProvider();
        using var model = new DesktopViewModel(timeProvider: clock);
        StartTurn(model);
        Retry(model, 58000);
        Apply(model, type, new(TurnId: "turn", State: state));
        clock.Advance(TimeSpan.FromSeconds(2));
        model.RefreshProviderRetryStatus();
        Assert.Empty(model.ProviderRetryText);
        Assert.False(model.IsProviderRetrying);
    });

    [Fact]
    public Task StaleEventsDoNotReplaceTheCurrentTurnsWait() => ui.Run(() =>
    {
        var clock = new FakeTimeProvider();
        using var model = new DesktopViewModel(timeProvider: clock);
        StartTurn(model);
        Retry(model, 58000);
        Apply(model, AgentEventTypes.ProviderRetryScheduled, new(TurnId: "older", DelayMs: 9000));
        clock.Advance(TimeSpan.FromSeconds(1));
        model.RefreshProviderRetryStatus();
        Assert.Contains("57s", model.ProviderRetryText);
        StartTurn(model, "next-turn");
        Retry(model, 58000);
        model.RefreshProviderRetryStatus();
        Assert.False(model.IsProviderRetrying);
    });

    [Fact]
    public Task SessionSnapshotAndDisposalClearTheWait() => ui.Run(() =>
    {
        using var model = new DesktopViewModel();
        StartTurn(model);
        Retry(model, 58000);
        model.ApplySnapshot(new([], [], [], "", [], [], null, null, "turn", "calling_model", false, null, null));
        model.RefreshProviderRetryStatus();
        Assert.False(model.IsProviderRetrying);
        Retry(model, 58000);
        typeof(DesktopViewModel).GetProperty(nameof(DesktopViewModel.SelectedSession))!
            .SetValue(model, new SessionItem("another-session", "Another session", "", false));
        model.RefreshProviderRetryStatus();
        Assert.False(model.IsProviderRetrying);
        Retry(model, 58000);
        model.Dispose();
        model.RefreshProviderRetryStatus();
        Assert.Empty(model.ProviderRetryText);
    });

    [Fact]
    public Task ExistingConversationTimerRefreshesTheCountdown() => ui.Run(async () =>
    {
        var app = Application.Current!;
        var theme = new FluentTheme();
        var resources = new ResourceInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/ThemeResources.axaml") };
        app.Styles.Add(theme);
        app.Resources.MergedDictionaries.Add(resources);
        var clock = new FakeTimeProvider();
        using var model = new DesktopViewModel(timeProvider: clock);
        typeof(DesktopViewModel).GetProperty(nameof(DesktopViewModel.SelectedSession))!
            .SetValue(model, new SessionItem("session", "Session", "", false));
        StartTurn(model);
        Retry(model, 58000);
        var chat = new ChatArea { DataContext = model };
        var window = new Window { Width = 640, Height = 500, Content = chat };
        try
        {
            window.Show();
            Dispatcher.UIThread.RunJobs(); window.UpdateLayout();
            var indicator = chat.GetVisualDescendants().OfType<TextBlock>()
                .Single(text => text.Text == model.ProviderRetryText);
            Assert.True(indicator.IsEffectivelyVisible);
            clock.Advance(TimeSpan.FromSeconds(1));
            await Task.Delay(1200);
            Dispatcher.UIThread.RunJobs();
            Assert.Contains("57s", indicator.Text);
            Assert.Equal(model.ProviderRetryText, indicator.Text);
        }
        finally
        {
            window.Close();
            app.Resources.MergedDictionaries.Remove(resources);
            app.Styles.Remove(theme);
        }
    });

    [Theory]
    [InlineData("en-US", "57s", "Waiting for retry to start")]
    [InlineData("zh-CN", "57 秒", "正在等待重试开始")]
    public Task CountdownUsesTheCurrentLocale(string locale, string remaining, string starting) => ui.Run(() =>
    {
        var language = LocalizationService.CreateDictionary(locale);
        Application.Current!.Resources.MergedDictionaries.Add(language);
        try
        {
            var clock = new FakeTimeProvider();
            using var model = new DesktopViewModel(timeProvider: clock);
            StartTurn(model);
            Retry(model, 58000);
            clock.Advance(TimeSpan.FromSeconds(1));
            model.RefreshProviderRetryStatus();
            Assert.Contains(remaining, model.ProviderRetryText);
            clock.Advance(TimeSpan.FromMinutes(1));
            model.RefreshProviderRetryStatus();
            Assert.Contains(starting, model.ProviderRetryText);
        }
        finally { Application.Current.Resources.MergedDictionaries.Remove(language); }
    });

    private static void StartTurn(DesktopViewModel model, string id = "turn") =>
        Apply(model, AgentEventTypes.TurnState, new(TurnId: id, State: "calling_model"));

    private static void Retry(DesktopViewModel model, ulong delayMs, uint attempt = 1) =>
        Apply(model, AgentEventTypes.ProviderRetryScheduled, new(TurnId: "turn", DelayMs: delayMs, RetryAttempt: attempt, MaxRetries: 2));

    private static void Apply(DesktopViewModel model, string type, AgentEventPayload payload) =>
        model.ApplyEvent(new AgentEvent("session", "2026-11-13T00:00:00Z", type, payload), live: false);
}
