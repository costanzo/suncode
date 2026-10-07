using System.Text.Json;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Tests;

/// <summary>Covers live-event families not exercised by SessionSnapshotProjectionTests.</summary>
public sealed class LiveEventRoutingTests
{
    [Fact]
    public void ApprovalEventsSetAndClearPendingApproval()
    {
        using var viewModel = new DesktopViewModel();
        viewModel.ApplyEvent(Event(AgentEventTypes.ApprovalRequested,
            """{"approval_id":"apr-1","turn_id":"turn-1","operation":"bash","arguments":{"command":"ls"}}"""), live: true);

        Assert.Equal("apr-1", viewModel.PendingApproval?.ApprovalId);
        Assert.Equal("bash", viewModel.PendingApproval?.Operation);
        var activity = Assert.Single(viewModel.Activities);
        Assert.Equal("Approval required for bash", activity.Text);
        Assert.Equal("bash", activity.Operation);
        Assert.False(viewModel.FullControlEnabled);

        viewModel.ApplyEvent(Event(AgentEventTypes.ApprovalResolved,
            """{"approval_id":"apr-1","turn_id":"turn-1","decision":"allow_session"}"""), live: true);

        Assert.Null(viewModel.PendingApproval);
        Assert.True(viewModel.FullControlEnabled);
        Assert.Equal(2, viewModel.Activities.Count);
    }

    [Fact]
    public void ApprovalResolvedWithoutSessionGrantKeepsFullControlOff()
    {
        using var viewModel = new DesktopViewModel();
        viewModel.ApplyEvent(Event(AgentEventTypes.ApprovalRequested,
            """{"approval_id":"apr-1","operation":"bash"}"""), live: false);
        viewModel.ApplyEvent(Event(AgentEventTypes.ApprovalResolved,
            """{"approval_id":"apr-1","decision":"allow_once"}"""), live: false);

        Assert.Null(viewModel.PendingApproval);
        Assert.False(viewModel.FullControlEnabled);
    }

    [Fact]
    public void QuestionRejectedClearsPendingQuestion()
    {
        using var viewModel = new DesktopViewModel();
        viewModel.ApplyEvent(Event(AgentEventTypes.QuestionAsked,
            """{"request_id":"que-1","turn_id":"turn-1","questions":[{"header":"Scope","question":"Proceed?","options":[{"label":"Yes","description":""}]}]}"""), live: true);
        Assert.Equal("que-1", viewModel.PendingQuestion?.RequestId);

        viewModel.ApplyEvent(Event(AgentEventTypes.QuestionRejected, """{"request_id":"que-1","turn_id":"turn-1"}"""), live: true);

        Assert.Null(viewModel.PendingQuestion);
        Assert.Equal("Question skipped", viewModel.Activities.Last().Text);
    }

    [Fact]
    public void ContextCompactedAddsProcessMessageWithoutActivity()
    {
        using var viewModel = new DesktopViewModel();
        viewModel.ApplyEvent(Event(AgentEventTypes.ContextCompacted,
            """{"turn_id":"turn-1","retained_tokens":10,"original_tokens":25}"""), live: true);

        var message = Assert.Single(viewModel.Messages);
        Assert.True(message.IsCompaction);
        Assert.True(message.IsProcess);
        Assert.Equal("turn-1", message.TurnId);
        Assert.Equal("Context compacted · retained 10 tokens", message.Text);
        Assert.Empty(viewModel.Activities);
    }

    [Fact]
    public void ToolStateAndResultUpdateToolActivityAndRecordActivities()
    {
        using var viewModel = new DesktopViewModel();
        viewModel.ApplyEvent(Event(AgentEventTypes.ToolRequested,
            """{"turn_id":"turn-1","tool_call_id":"tool-1","name":"read","state":"requested","arguments":{"path":"README.md"}}"""), live: false);
        viewModel.ApplyEvent(Event(AgentEventTypes.ToolState,
            """{"turn_id":"turn-1","tool_call_id":"tool-1","state":"failed","reason":"scope_denied"}"""), live: false);

        var tool = Assert.Single(Assert.Single(viewModel.ToolActivity.Turns).Tools);
        Assert.Equal("failed", tool.State);
        Assert.Equal("scope_denied", tool.Error);
        Assert.Equal("read", tool.Name);

        viewModel.ApplyEvent(Event(AgentEventTypes.ToolResult,
            """{"turn_id":"turn-1","tool_call_id":"tool-1","name":"read","result":{"ok":true}}"""), live: false);

        Assert.Contains("ok", tool.Result);
        Assert.Equal("failed", tool.State);
        Assert.Equal(
            [AgentEventTypes.ToolRequested, AgentEventTypes.ToolState, AgentEventTypes.ToolResult],
            viewModel.Activities.Select(item => item.EventType));
        Assert.Equal("read", viewModel.Activities[0].Operation);
    }

    [Fact]
    public void ToolEventWithoutCallIdStillRecordsActivity()
    {
        using var viewModel = new DesktopViewModel();
        viewModel.ApplyEvent(Event(AgentEventTypes.ToolRequested, """{"turn_id":"turn-1","name":"read"}"""), live: false);

        Assert.Empty(viewModel.ToolActivity.Turns);
        Assert.Single(viewModel.Activities);
    }

    [Fact]
    public void ProviderExchangeUpdatesUsageWithoutActivity()
    {
        using var viewModel = new DesktopViewModel();
        viewModel.ApplyEvent(Event(AgentEventTypes.ProviderExchangeCompleted,
            """{"exchange_id":"exchange-1","usage":{"input_tokens":1200,"output_tokens":300,"total_tokens":1500}}"""), live: false);

        Assert.Equal("1.2k", viewModel.ContextUsage.InputTokenText);
        Assert.Empty(viewModel.Activities);
        Assert.Empty(viewModel.Messages);
    }

    [Fact]
    public void UsageOutsideProviderExchangeIsIgnored()
    {
        using var viewModel = new DesktopViewModel();
        viewModel.ApplyEvent(Event(AgentEventTypes.TurnState,
            """{"turn_id":"turn-1","state":"calling_model","usage":{"input_tokens":1200,"output_tokens":300,"total_tokens":1500}}"""), live: false);

        Assert.Equal("--", viewModel.ContextUsage.InputTokenText);
    }

    [Fact]
    public void CheckpointEventRecordsActivityAndChangedPathOnce()
    {
        using var viewModel = new DesktopViewModel();
        viewModel.ApplyEvent(Event(AgentEventTypes.CheckpointCaptured,
            """{"turn_id":"turn-1","manifest_id":"m-1","path":"src/App.cs"}"""), live: false);
        viewModel.ApplyEvent(Event(AgentEventTypes.CheckpointCaptured,
            """{"turn_id":"turn-1","manifest_id":"m-1","path":"src/App.cs"}"""), live: false);

        Assert.Equal(["src/App.cs"], viewModel.ChangedPaths);
        Assert.True(viewModel.HasChangedPaths);
        Assert.Equal("Checkpoint captured for src/App.cs", viewModel.Activities[0].Text);
        Assert.Equal(2, viewModel.Activities.Count);
    }

    [Fact]
    public void CheckpointRestoreFailedUsesUndoText()
    {
        using var viewModel = new DesktopViewModel();
        viewModel.ApplyEvent(Event(AgentEventTypes.CheckpointRestoreFailed, """{"turn_id":"turn-1"}"""), live: false);

        Assert.Equal("Undo stopped because a file changed outside SunCode", Assert.Single(viewModel.Activities).Text);
    }

    [Fact]
    public void UnknownEventFallsBackToTypeAsActivityText()
    {
        using var viewModel = new DesktopViewModel();
        viewModel.ApplyEvent(Event("session.custom", """{"state":"ok","operation":"op"}"""), live: false);

        var activity = Assert.Single(viewModel.Activities);
        Assert.Equal("session.custom", activity.Text);
        Assert.Equal("ok", activity.State);
        Assert.Equal("op", activity.Operation);
    }

    [Fact]
    public void TurnStateRecordsActivityAndTracksActiveTurn()
    {
        using var viewModel = new DesktopViewModel();
        viewModel.ApplyEvent(Event(AgentEventTypes.TurnState, """{"turn_id":"turn-1","state":"calling_model"}"""), live: false);

        Assert.Equal("turn-1", viewModel.ActiveTurnId);
        Assert.Equal("Turn calling_model", Assert.Single(viewModel.Activities).Text);

        viewModel.ApplyEvent(Event(AgentEventTypes.TurnState, """{"turn_id":"turn-1","state":"completed"}"""), live: false);

        Assert.Equal(string.Empty, viewModel.ActiveTurnId);
        Assert.Equal("completed", viewModel.ActiveTurnState);
    }

    private static AgentEvent Event(string eventType, string payloadJson) => new(
        string.Empty,
        "2026-09-13T00:00:00.000Z",
        eventType,
        JsonSerializer.Deserialize<AgentEventPayload>(payloadJson, JsonOptions) ?? new AgentEventPayload());

    private static readonly JsonSerializerOptions JsonOptions = new() { PropertyNameCaseInsensitive = true };
}
