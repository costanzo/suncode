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
    public void ProjectionOmitsAssistantMessagesWithoutVisibleText()
    {
        var snapshot = JsonNode.Parse("""
        {
          "messages": [
            {"role":"user","content":[{"type":"text","text":"inspect this"}]},
            {
              "role":"assistant",
              "content":[],
              "tool_calls":[{"call_id":"call-1","name":"read","arguments":{"path":"README.md"}}]
            },
            {"role":"assistant","content":[{"type":"text","text":"done"}]}
          ]
        }
        """)!.AsObject();

        var projection = DesktopViewModel.ProjectSnapshot(snapshot);

        Assert.Collection(
            projection.Messages,
            message => Assert.Equal(("user", "inspect this", 1L), (message.Role, message.Text, message.ContentSequence)),
            message => Assert.Equal(("assistant", "done", 2L), (message.Role, message.Text, message.ContentSequence)));
    }

    [Fact]
    public void LiveProjectionOmitsAssistantMessagesWithoutVisibleText()
    {
        var viewModel = new DesktopViewModel();
        var changes = 0;
        viewModel.ConversationChanged += () => changes++;
        var toolCallMessage = JsonNode.Parse("""
        {
          "event_type":"message.assistant",
          "payload":{
            "turn_id":"turn-1",
            "message":{
              "role":"assistant",
              "content":[],
              "tool_calls":[{"call_id":"call-1","name":"read","arguments":{"path":"README.md"}}]
            }
          }
        }
        """)!.AsObject();

        viewModel.ApplyEvent(toolCallMessage, live: true);

        Assert.Empty(viewModel.Messages);
        Assert.Equal(0, changes);
    }

    [Fact]
    public void EmptyFinalEventPreservesAlreadyStreamedAssistantText()
    {
        var viewModel = new DesktopViewModel();
        viewModel.ApplyEvent(JsonNode.Parse("""
        {
          "event_type":"assistant.delta",
          "payload":{"turn_id":"turn-1","text":"visible response"}
        }
        """)!.AsObject(), live: true);
        var emptyFinalMessage = JsonNode.Parse("""
        {
          "event_type":"message.assistant",
          "payload":{
            "message_id":"message-1",
            "turn_id":"turn-1",
            "message":{"role":"assistant","content":[]}
          }
        }
        """)!.AsObject();

        viewModel.ApplyEvent(emptyFinalMessage, live: true);

        var message = Assert.Single(viewModel.Messages);
        Assert.Equal("message-1", message.MessageId);
        Assert.Equal("visible response", message.Text);
        Assert.False(message.Streaming);
    }

    [Fact]
    public void DuplicateLiveMessageIdIsAppliedOnce()
    {
        var viewModel = new DesktopViewModel();
        var userMessage = JsonNode.Parse("""
        {
          "event_type":"message.user",
          "payload":{
            "message_id":"message-1",
            "turn_id":"turn-1",
            "message":{"role":"user","content":[{"type":"text","text":"only once"}]}
          }
        }
        """)!.AsObject();

        viewModel.ApplyEvent(userMessage, live: true);
        viewModel.ApplyEvent(userMessage, live: true);

        var message = Assert.Single(viewModel.Messages);
        Assert.Equal("message-1", message.MessageId);
        Assert.Equal("only once", message.Text);
    }

    [Fact]
    public void DistinctMessageIdsPreserveRepeatedText()
    {
        var viewModel = new DesktopViewModel();
        foreach (var messageId in new[] { "message-1", "message-2" })
        {
            viewModel.ApplyEvent(JsonNode.Parse($$"""
            {
              "event_type":"message.user",
              "payload":{
                "message_id":"{{messageId}}",
                "turn_id":"turn-1",
                "message":{"role":"user","content":[{"type":"text","text":"same text"}]}
              }
            }
            """)!.AsObject(), live: true);
        }

        Assert.Equal(2, viewModel.Messages.Count);
    }

    [Fact]
    public void MessageProjectionCombinesAllTextParts()
    {
        var snapshot = JsonNode.Parse("""
        {
          "messages": [
            {
              "role":"assistant",
              "content":[
                {"type":"text","text":"first"},
                {"type":"text","text":"second"}
              ]
            }
          ]
        }
        """)!.AsObject();

        var projection = DesktopViewModel.ProjectSnapshot(snapshot);

        Assert.Equal("first\nsecond", Assert.Single(projection.Messages).Text);
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
