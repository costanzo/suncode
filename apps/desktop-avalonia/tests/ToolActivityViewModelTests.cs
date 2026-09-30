using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Tests;

public sealed class ToolActivityViewModelTests
{
    [Fact]
    public void EnsureTurnCreatesOnceAndKeepsSequence()
    {
        var viewModel = new ToolActivityViewModel();

        var first = viewModel.EnsureTurn("turn-a", "Inspect the project");
        var again = viewModel.EnsureTurn("turn-a");
        var second = viewModel.EnsureTurn("turn-b");

        Assert.Same(first, again);
        Assert.Equal(1, first.Sequence);
        Assert.Equal(2, second.Sequence);
        Assert.True(first.IsExpanded);
        Assert.True(viewModel.HasTurns);
        Assert.Equal("2 turns · 0 calls", viewModel.Summary);
    }

    [Fact]
    public void UpsertSelectsFirstCallThenUpdatesInPlace()
    {
        var viewModel = new ToolActivityViewModel();
        var turn = viewModel.EnsureTurn("turn-a");

        viewModel.Upsert(turn, "call-1", "read_file", "requested", "{}", string.Empty, string.Empty, string.Empty);
        var tool = Assert.Single(turn.Tools);
        Assert.Same(tool, viewModel.SelectedTool);
        Assert.Same(turn, viewModel.SelectedTurn);
        Assert.Equal("1 turn · 1 calls", viewModel.Summary);

        viewModel.Upsert(turn, "call-1", "read_file", "running", "{}", string.Empty, "partial", string.Empty);
        Assert.Same(tool, Assert.Single(turn.Tools));
        Assert.Equal("running", tool.State);
        Assert.Equal("partial", tool.Output);
    }

    [Fact]
    public void ActiveToolFollowsRunningTurnAndTool()
    {
        var viewModel = new ToolActivityViewModel();
        var turn = viewModel.EnsureTurn("turn-a");
        viewModel.Upsert(turn, "call-1", "shell", "running", "{}", string.Empty, string.Empty, string.Empty);

        var active = Assert.NotNull(viewModel.ActiveTool);
        Assert.Same(turn, active.Turn);
        Assert.Equal("call-1", active.Tool.ToolCallId);

        turn.Update("completed");
        Assert.Null(viewModel.ActiveTool);
    }

    [Fact]
    public void ReplaceAllSelectsLatestCallAndTrySelectFindsById()
    {
        var viewModel = new ToolActivityViewModel();
        var older = new ToolActivityTurnItem("turn-a", 1, "completed", string.Empty, string.Empty);
        older.Tools.Add(Tool("turn-a", "call-1", "completed"));
        var newer = new ToolActivityTurnItem("turn-b", 2, "completed", string.Empty, string.Empty);
        newer.Tools.Add(Tool("turn-b", "call-2", "completed"));

        viewModel.ReplaceAll([older, newer]);
        Assert.Equal("call-2", viewModel.SelectedTool?.ToolCallId);

        Assert.True(viewModel.TrySelect("turn-a", "call-1"));
        Assert.Equal("call-1", viewModel.SelectedTool?.ToolCallId);
        Assert.False(viewModel.TrySelect("turn-a", "missing"));
        Assert.Equal("call-1", viewModel.SelectedTool?.ToolCallId);

        viewModel.ReplaceAll([]);
        Assert.Null(viewModel.SelectedTool);
        Assert.Equal("Select a tool call", viewModel.SelectedTitle);
    }

    [Fact]
    public void ToggleTurnSwitchesSelectionToThatTurn()
    {
        var viewModel = new ToolActivityViewModel();
        var first = viewModel.EnsureTurn("turn-a");
        viewModel.Upsert(first, "call-1", "shell", "completed", "{}", string.Empty, string.Empty, string.Empty);
        var second = viewModel.EnsureTurn("turn-b");
        second.Tools.Add(Tool("turn-b", "call-2", "completed"));

        viewModel.ToggleTurn(second);

        Assert.Same(second, viewModel.SelectedTurn);
        Assert.Equal("call-2", viewModel.SelectedTool?.ToolCallId);
    }

    private static ToolActivityItem Tool(string turnId, string toolCallId, string state) =>
        new(turnId, toolCallId, "shell", state, "{}", string.Empty, string.Empty, string.Empty, string.Empty);
}
