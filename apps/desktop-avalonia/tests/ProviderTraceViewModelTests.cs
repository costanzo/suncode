using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Tests;

public sealed class ProviderTraceViewModelTests
{
    [Fact]
    public void FilterKeepsMatchingTurnsAndCallsAndSelectsFirstVisibleCall()
    {
        var viewModel = Seeded(new FakeViewModelHost { SelectedSessionId = "session" });

        viewModel.ApplyFilter();
        Assert.Equal(2, viewModel.FilteredTurns.Count);
        Assert.Equal("exchange-b1", viewModel.SelectedTrace?.ExchangeId);
        Assert.Equal("2 turns · 3 calls", viewModel.CountText);

        viewModel.SelectedTrace = viewModel.FilteredTurns[1].Calls[1];
        viewModel.SetFilter("gpt");
        Assert.Equal("exchange-a2", viewModel.SelectedTrace?.ExchangeId);

        viewModel.SetFilter("claude");
        var turn = Assert.Single(viewModel.FilteredTurns);
        Assert.Equal("exchange-b1", Assert.Single(turn.Calls).ExchangeId);
        Assert.Equal("exchange-b1", viewModel.SelectedTrace?.ExchangeId);

        viewModel.SetFilter("nothing-matches");
        Assert.False(viewModel.HasFilteredTraces);
        Assert.Null(viewModel.SelectedTrace);
        Assert.Equal("No turns or model calls match this filter.", viewModel.EmptyMessage);
    }

    [Fact]
    public void EmptyMessageFollowsHostSession()
    {
        var host = new FakeViewModelHost();
        var viewModel = new ProviderTraceViewModel(host);
        Assert.Equal("Select a session to inspect provider requests.", viewModel.EmptyMessage);

        host.SelectedSessionId = "session";
        Assert.Equal("No turns have been recorded for this session.", viewModel.EmptyMessage);
    }

    [Fact]
    public async Task RefreshWithoutSdkClearsTrace()
    {
        var viewModel = Seeded(new FakeViewModelHost { SelectedSessionId = "session" });
        viewModel.SetFilter("claude");

        await viewModel.RefreshAsync();

        Assert.Empty(viewModel.Traces);
        Assert.Empty(viewModel.Turns);
        Assert.Empty(viewModel.FilteredTurns);
        Assert.Equal(string.Empty, viewModel.Filter);
        Assert.Equal("idle", viewModel.State);
        Assert.Equal("No provider requests", viewModel.Summary);
    }

    [Fact]
    public void SelectingTurnClearsCallSelection()
    {
        var viewModel = Seeded(new FakeViewModelHost { SelectedSessionId = "session" });
        viewModel.ApplyFilter();

        viewModel.SelectTurn();

        Assert.Null(viewModel.SelectedTrace);
        Assert.False(viewModel.HasSelectedTrace);
        Assert.Equal("No model call selected", viewModel.SelectedTraceTitle);
    }

    [Fact]
    public void WindowViewModelExposesProviderTraceChild()
    {
        using var window = new DesktopViewModel();

        Assert.Equal("Select a session to inspect provider requests.", window.ProviderTrace.EmptyMessage);
        Assert.Equal("idle", window.ProviderTrace.State);
    }

    private static ProviderTraceViewModel Seeded(FakeViewModelHost host)
    {
        var viewModel = new ProviderTraceViewModel(host);
        var a1 = Call("exchange-a1", "turn-a", "gpt-5.6-sol", 1);
        var a2 = Call("exchange-a2", "turn-a", "gpt-5.6-sol", 2);
        var b1 = Call("exchange-b1", "turn-b", "claude-opus-5-5", 1);
        foreach (var call in new[] { a1, a2, b1 }) viewModel.Traces.Add(call);
        viewModel.Turns.Add(Turn("turn-b", "claude-opus-5-5", 2, [b1]));
        viewModel.Turns.Add(Turn("turn-a", "gpt-5.6-sol", 1, [a1, a2]));
        return viewModel;
    }

    private static ProviderTraceItem Call(string exchangeId, string turnId, string modelId, int iteration) =>
        new(exchangeId, turnId, "provider", modelId, modelId, string.Empty, string.Empty, "completed", iteration,
            "2026-09-30T10:00:00.000Z", "2026-09-30T10:00:01.000Z", null, null, null, null, null,
            string.Empty, string.Empty, "[]", string.Empty, [], []);

    private static ProviderTraceTurnItem Turn(string turnId, string modelId, int sequence, IReadOnlyList<ProviderTraceItem> calls) =>
        new(turnId, "completed", modelId, "2026-09-30T09:59:59.000Z", string.Empty, string.Empty, 0, 0, 0, sequence, calls);
}
