using System.Text.Json.Nodes;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Tests;

public sealed class SessionSnapshotProjectionTests
{
    [Fact]
    public void ProjectionPreservesNormalizedMessages()
    {
        var snapshot = JsonNode.Parse("""
        {
          "messages": [
            {"role":"user","content":[{"type":"text","text":"first"}]},
            {"role":"assistant","content":[{"type":"text","text":"second"}]}
          ]
        }
        """)!.AsObject();

        var projection = DesktopViewModel.ProjectSnapshot(snapshot);

        Assert.Collection(
            projection.Messages,
            message => Assert.Equal(("user", "first", 1L), (message.Role, message.Text, message.ContentSequence)),
            message => Assert.Equal(("assistant", "second", 2L), (message.Role, message.Text, message.ContentSequence)));
        Assert.Empty(projection.ChangedPaths);
        Assert.Empty(projection.Activities);
        Assert.Null(projection.PendingApproval);
        Assert.Empty(projection.ActiveTurnId);
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
            string.Empty);

        viewModel.ApplySnapshot(projection);

        Assert.NotSame(original, viewModel.Messages);
        Assert.Equal("new session", Assert.Single(viewModel.Messages).Text);
        Assert.Equal(1, sourceChanges);
        Assert.Equal(1, changes);
    }
}
