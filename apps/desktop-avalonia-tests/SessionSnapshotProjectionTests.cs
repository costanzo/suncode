using System.Text.Json.Nodes;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Tests;

public sealed class SessionSnapshotProjectionTests
{
    [Fact]
    public void ProjectionPreservesFinalReplayState()
    {
        var snapshot = JsonNode.Parse("""
        {
          "messages": [
            {"role":"user","content":[{"type":"text","text":"first"}]},
            {"role":"assistant","content":[{"type":"text","text":"second"}]}
          ],
          "events": [
            {"content_sequence":1,"event_type":"message.user","payload":{}},
            {"content_sequence":2,"event_type":"operation.completed","payload":{"path":"a.txt","from":"old.txt","to":"a.txt","operation":"write"}},
            {"content_sequence":3,"event_type":"provider.exchange.completed","payload":{"path":"ignored.txt"}},
            {"content_sequence":4,"event_type":"approval.requested","payload":{"approval_id":"approval-1","operation":"process/run","arguments":{"command":"build"}}},
            {"content_sequence":5,"event_type":"turn.state","payload":{"state":"running","turn_id":"turn-1"}}
          ],
          "latest_sequence":8
        }
        """)!.AsObject();

        var projection = DesktopViewModel.ProjectSnapshot(snapshot);

        Assert.Collection(
            projection.Messages,
            message => Assert.Equal(("user", "first", 1L), (message.Role, message.Text, message.ContentSequence)),
            message => Assert.Equal(("assistant", "second", 2L), (message.Role, message.Text, message.ContentSequence)));
        Assert.Equal(["a.txt", "old.txt", "ignored.txt"], projection.ChangedPaths);
        Assert.Equal(3, projection.Activities.Count);
        Assert.Equal("approval-1", projection.PendingApproval?.ApprovalId);
        Assert.Equal("turn-1", projection.ActiveTurnId);
        Assert.Equal(8, projection.LastSequence);
    }

    [Fact]
    public void ApplyingProjectionReplacesTheMessageSourceAndSignalsOnce()
    {
        var viewModel = new DesktopViewModel();
        var original = viewModel.Messages;
        var changes = 0;
        var sourceChanges = 0;
        viewModel.ConversationChanged += () => changes++;
        viewModel.PropertyChanged += (_, args) =>
        {
            if (args.PropertyName == nameof(DesktopViewModel.Messages)) sourceChanges++;
        };
        var projection = new SessionSnapshotProjection(
            [new() { Role = "assistant", Text = "new session", ContentSequence = 1 }],
            [],
            [],
            null,
            string.Empty,
            1);

        viewModel.ApplySnapshot(projection);

        Assert.NotSame(original, viewModel.Messages);
        Assert.Equal("new session", Assert.Single(viewModel.Messages).Text);
        Assert.Equal(1, sourceChanges);
        Assert.Equal(1, changes);
    }
}
