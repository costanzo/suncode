using System.Text.Json;
using System.Reflection;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Tests;

public sealed class ProviderFailureNavigationTests
{
    [Fact]
    public async Task NavigationSelectsExactAttemptRevealsFilteredTurnAndLoadsOnlyTheDestination()
    {
        var previous = Call("attempt-1", startedAt: "2026-11-08T10:00:00Z");
        var failed = Call("attempt-2", startedAt: "2026-11-08T10:00:01Z");
        var host = Host(previous, failed);
        var loaded = new List<string>();
        host.ProviderExchangeLoader = (_, exchangeId) =>
        {
            loaded.Add(exchangeId);
            return Task.FromResult(Details(failed));
        };
        var model = new ProviderTraceViewModel(host);
        model.SetFilter("does-not-match");
        var focused = 0;
        model.FailureFocusRequested += () => focused++;

        await model.RevealFailureAsync("turn", "attempt-2", "transient");

        Assert.Equal("attempt-2", model.SelectedTrace?.ExchangeId);
        Assert.Equal("Connection reset by peer", TurnFailureItem.FromCall(model.SelectedTraceDetails!).Message);
        Assert.Equal(["attempt-2"], loaded);
        Assert.Equal(string.Empty, model.Filter);
        Assert.True(Assert.Single(model.FilteredTurns).IsExpanded);
        Assert.Equal("ready", model.FailureNavigationState);
        Assert.Equal(1, focused);
    }

    [Fact]
    public async Task RestoredFailureUsesLatestAttemptAndChecksTheTurnErrorCode()
    {
        var older = Call("old", startedAt: "2026-11-08T10:00:00Z");
        var latest = Call("last", startedAt: "2026-11-08T10:00:02Z");
        var model = new ProviderTraceViewModel(Host(older, latest));
        await model.RevealFailureAsync("turn", string.Empty, "transient");
        Assert.Equal("last", model.SelectedTrace?.ExchangeId);
        Assert.Null(model.FindFailure("different-turn", "transient"));
        Assert.Null(model.FindFailure("turn", "tool_error"));
        Assert.Null(model.FindFailure("turn", "transient", "deleted-call"));
    }

    [Fact]
    public async Task SuccessfulRequestAfterRetryOrCompactionFailurePreventsEarlierErrorAttribution()
    {
        var failed = Call("compaction-or-retry", startedAt: "2026-11-08T10:00:00Z");
        var success = Call("recovered", state: "completed", startedAt: "2026-11-08T10:00:02Z");
        var model = new ProviderTraceViewModel(Host(failed, success));
        await model.RevealFailureAsync("turn", string.Empty, "transient");
        Assert.Equal("missing", model.FailureNavigationState);
        Assert.Null(model.SelectedTrace);
        Assert.False(model.ShowSelectedTraceOverview);
    }

    [Fact]
    public async Task AmbiguousHistoricalAttemptsRequireAnExactCallId()
    {
        var first = Call("first");
        var second = Call("second");
        var model = new ProviderTraceViewModel(Host(first, second));
        await model.RevealFailureAsync("turn", string.Empty, "transient");
        Assert.Equal("missing", model.FailureNavigationState);
        await model.RevealFailureAsync("turn", "second", "transient");
        Assert.Equal("second", model.SelectedTraceDetails?.ExchangeId);
    }

    [Fact]
    public async Task DetailFailureRetainsTheDestinationForRetry()
    {
        var failed = Call("failed");
        var host = Host(failed);
        host.ProviderExchangeLoader = (_, _) => throw new IOException("Local call read failed");
        var model = new ProviderTraceViewModel(host);
        await model.RevealFailureAsync("turn", "failed", "transient");
        Assert.True(model.CanRetryFailureNavigation);
        Assert.Contains("Local call read failed", model.Error);
        Assert.False(model.ShowSelectedTraceOverview);

        host.ProviderExchangeLoader = (_, _) => Task.FromResult(Details(failed));
        await model.RetryFailureNavigationAsync();
        Assert.Equal("ready", model.FailureNavigationState);
        Assert.Equal("failed", model.SelectedTraceDetails?.ExchangeId);
        Assert.False(model.ShowFailureNavigationFeedback);
    }

    [Fact]
    public async Task MissingRequestClearsAnUnrelatedSelection()
    {
        var call = Call("another");
        var model = new ProviderTraceViewModel(Host(call));
        await model.RefreshAsync();
        Assert.NotNull(model.SelectedTraceDetails);
        await model.RevealFailureAsync("turn", "missing", "transient");
        Assert.Equal("missing", model.FailureNavigationState);
        Assert.Null(model.SelectedTraceDetails);
        Assert.False(model.CanRetryFailureNavigation);
        Assert.False(model.ShowEmptyState);
    }

    [Fact]
    public async Task LoadingPreventsDuplicateNavigationAndBackgroundRefresh()
    {
        var pending = new TaskCompletionSource<ProviderExchangesResult>();
        var failed = Call("failed");
        var host = Host(failed);
        var reads = 0;
        host.ProviderExchangesLoader = _ => { reads++; return pending.Task; };
        var model = new ProviderTraceViewModel(host);
        var first = model.RevealFailureAsync("turn", "failed", "transient");
        Assert.True(model.IsRevealingFailure);
        await model.RevealFailureAsync("turn", "failed", "transient");
        await model.RefreshAsync();
        Assert.Equal(1, reads);
        pending.SetResult(Result(failed));
        await first;
        Assert.Equal("ready", model.FailureNavigationState);
    }

    [Fact]
    public async Task SessionSwitchDiscardsPendingNavigationAndItsDetailCacheEvenWhenReturningToTheSameSession()
    {
        var failed = Call("failed");
        var pending = new TaskCompletionSource<ProviderExchangeDetails>();
        var host = Host(failed);
        host.ProviderExchangeLoader = (_, _) => pending.Task;
        var model = new ProviderTraceViewModel(host);
        var focused = 0;
        model.FailureFocusRequested += () => focused++;
        var navigation = model.RevealFailureAsync("turn", "failed", "transient");
        host.SelectedSessionId = "other";
        model.OnSessionChanged();
        host.SelectedSessionId = "session";
        model.OnSessionChanged();
        pending.SetResult(Details(failed));
        await navigation;
        Assert.Equal(0, focused);
        Assert.Null(model.SelectedTraceDetails);
        Assert.Empty(model.Traces);
        Assert.False(model.IsRevealingFailure);

        var reads = 0;
        host.ProviderExchangeLoader = (_, _) => { reads++; return Task.FromResult(Details(failed)); };
        await model.RevealFailureAsync("turn", "failed", "transient");
        Assert.Equal(1, reads);
    }

    [Fact]
    public async Task OlderListRefreshCannotOverwriteNewerNavigation()
    {
        var failed = Call("failed");
        var old = new TaskCompletionSource<ProviderExchangesResult>();
        var host = Host(failed);
        host.ProviderExchangesLoader = _ => old.Task;
        var model = new ProviderTraceViewModel(host);
        var refresh = model.RefreshAsync();
        host.ProviderExchangesLoader = _ => Task.FromResult(Result(failed));
        await model.RevealFailureAsync("turn", "failed", "transient");
        old.SetResult(Result(Call("obsolete")));
        await refresh;
        Assert.Equal("failed", Assert.Single(model.Traces).ExchangeId);
        Assert.Equal("failed", model.SelectedTraceDetails?.ExchangeId);
    }

    [Fact]
    public async Task UserSelectionSupersedesPendingErrorNavigation()
    {
        var failed = Call("failed");
        var another = Call("another", turnId: "another-turn");
        var pending = new TaskCompletionSource<ProviderExchangeDetails>();
        var host = Host(failed, another);
        host.ProviderExchangeLoader = (_, id) => id == "failed" ? pending.Task : Task.FromResult(Details(another));
        var model = new ProviderTraceViewModel(host);
        var focused = 0;
        model.FailureFocusRequested += () => focused++;
        var navigation = model.RevealFailureAsync("turn", "failed", "transient");
        await model.LoadAsync(model.Traces.Single(item => item.ExchangeId == "another"));
        pending.SetResult(Details(failed));
        await navigation;
        Assert.Equal("another", model.SelectedTraceDetails?.ExchangeId);
        Assert.Equal(0, focused);
        Assert.False(model.ShowFailureNavigationFeedback);
    }

    [Fact]
    public void SessionMetadataUpdatesPreserveTraceSelectionAndFiltering()
    {
        using var model = new DesktopViewModel();
        var selection = typeof(DesktopViewModel).GetProperty(nameof(DesktopViewModel.SelectedSession))!;
        var session = new SessionItem("session", "Session", "", false);
        selection.SetValue(model, session);
        model.Sessions.Add(session);
        var call = ProviderTraceMapper.FromSdk(Call("failed"));
        model.ProviderTrace.Traces.Add(call);
        model.ProviderTrace.Turns.Add(new("turn", "failed", "gpt-model", "", "", "", 0, 0, 0, 1, [call], "transient"));
        model.ProviderTrace.SetFilter("gpt-model");
        model.SyncSelectedSessionAgentState("failed");
        Assert.Equal("failed", model.ProviderTrace.SelectedTrace?.ExchangeId);
        Assert.Equal("gpt-model", model.ProviderTrace.Filter);
        selection.SetValue(model, session with { SessionId = "other" });
        Assert.Empty(model.ProviderTrace.Traces);
        Assert.Equal(string.Empty, model.ProviderTrace.Filter);
    }

    [Fact]
    public void LiveFailureSummaryUsesFinalAttemptAndKeepsNonProviderReasonsAccurate()
    {
        using var model = new DesktopViewModel();
        ApplyFailure(model, "first");
        model.ApplyEvent(Event(AgentEventTypes.ProviderExchangeStarted,
            """{"turn_id":"turn","exchange_id":"final","provider":"openai","model_id":"gpt-model"}"""), live: false);
        model.ApplyEvent(Event(AgentEventTypes.ProviderExchangeFailed,
            """{"turn_id":"turn","exchange_id":"final","error":{"code":"transient","message":"Connection reset by peer","retryable":true}}"""), live: false);
        model.ApplyEvent(Event(AgentEventTypes.TurnState,
            """{"turn_id":"turn","state":"failed","reason":"transient"}"""), live: false);
        Assert.Equal("final", model.ReviewFailure?.ExchangeId);
        Assert.Equal("openai · gpt-model", model.ReviewFailure?.ProviderModel);
        Assert.Equal("Connection reset by peer", model.ReviewFailure?.Summary);

        model.ApplyEvent(Event(AgentEventTypes.TurnState,
            """{"turn_id":"turn","state":"failed","reason":"tool_error"}"""), live: false);
        Assert.False(model.ReviewFailure!.HasProviderCall);
        Assert.Equal("tool_error", model.ReviewFailure.Summary);
        model.ApplyEvent(Event(AgentEventTypes.TurnState,
            """{"turn_id":"next","state":"admitted"}"""), live: false);
        Assert.Null(model.ReviewFailure);
    }

    [Fact]
    public void RestoredSnapshotKeepsTheFailedTurnIdAndHydratesItsPersistedReason()
    {
        using var model = new DesktopViewModel();
        var snapshot = JsonSerializer.Deserialize<SessionSnapshot>("""
            {"conversationTurns":[{"turnId":"turn","state":"failed","createdAt":"2026-11-08T10:00:00Z","messages":[],"toolUses":[],"todos":[]}],"messages":[],"images":[]}
            """, new JsonSerializerOptions { PropertyNameCaseInsensitive = true })!;
        model.ApplySnapshot(DesktopViewModel.ProjectSnapshot(snapshot));
        Assert.Equal("turn", model.LastTurnId);
        Assert.Equal("turn", model.ReviewFailure?.TurnId);
        var call = ProviderTraceMapper.FromSdk(Call("persisted"));
        model.ProviderTrace.Traces.Add(call);
        model.ProviderTrace.Turns.Add(new("turn", "failed", "gpt-model", "", "", "", 0, 0, 0, 1, [call], "transient"));
        model.RefreshReviewFailure();
        Assert.Equal("persisted", model.ReviewFailure?.ExchangeId);
        Assert.Equal("Connection reset by peer", model.ReviewFailure?.Summary);
        model.ClearSessionForTests();
        Assert.Null(model.ReviewFailure);
    }

    internal static void ApplyFailure(DesktopViewModel model, string exchangeId = "failed")
    {
        model.ApplyEvent(Event(AgentEventTypes.ProviderExchangeStarted,
            JsonSerializer.Serialize(new { turn_id = "turn", exchange_id = exchangeId, provider = "openai", model_id = "gpt-model" })), live: false);
        model.ApplyEvent(Event(AgentEventTypes.ProviderExchangeFailed,
            JsonSerializer.Serialize(new { turn_id = "turn", exchange_id = exchangeId, error = new { code = "transient", message = "Connection reset by peer", retryable = true } })), live: false);
        model.ApplyEvent(Event(AgentEventTypes.TurnState,
            """{"turn_id":"turn","state":"failed","reason":"transient"}"""), live: false);
    }

    internal static FakeViewModelHost Host(params ProviderExchange[] calls) => new()
    {
        SelectedSessionId = "session",
        ProviderExchangesLoader = _ => Task.FromResult(Result(calls)),
        ProviderExchangeLoader = (_, exchangeId) => Task.FromResult(Details(calls.Single(item => item.ExchangeId == exchangeId)))
    };

    internal static ProviderExchangesResult Result(params ProviderExchange[] calls) => new("session",
        calls.Select(item => item.TurnId).Distinct().Select(turnId => new SessionTraceTurn(
            turnId, "session", "failed", "gpt-model", "2026-11-08T10:00:00Z", "2026-11-08T10:00:03Z",
            "2026-11-08T10:00:00Z", "2026-11-08T10:00:03Z", "transient", 0, 0, 0)).ToArray(), calls);

    internal static ProviderExchange Call(string exchangeId, string turnId = "turn", string state = "failed", string startedAt = "2026-11-08T10:00:00Z") => new(
        exchangeId, "session", turnId, "openai", "gpt-model", "gpt-model", null, null, state, 1,
        startedAt, startedAt, null, [], null, null,
        state == "failed" ? JsonSerializer.SerializeToElement(new { code = "transient", message = "Connection reset by peer", retryable = true, details = new { response_started = false } }) : null);

    internal static ProviderExchangeDetails Details(ProviderExchange call) => new(
        call.ExchangeId, call.SessionId, call.TurnId, call.Provider, call.ModelId, call.WireModel,
        call.ProviderRequestId, call.ProviderResponseId, call.State, call.Iteration, call.StartedAt,
        call.CompletedAt, call.OutputMessage, call.ToolCalls, call.Usage, call.FinishReason, call.Error, [], []);

    private static AgentEvent Event(string type, string payload) => new("session", "2026-11-08T10:00:00Z", type,
        JsonSerializer.Deserialize<AgentEventPayload>(payload, new JsonSerializerOptions { PropertyNameCaseInsensitive = true })!);
}
