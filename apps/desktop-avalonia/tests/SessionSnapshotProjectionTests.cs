using System.Text.Json.Nodes;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Tests;

public sealed class SessionSnapshotProjectionTests
{
    [Fact]
    public void ToolMessagesShowAnOperationSummaryAndKeepDetailsForTheDialog()
    {
        var message = new MessageItem
        {
            Role = "tool",
            Text = "bash",
            ContentSequence = 1,
            Kind = "tool",
            ToolName = "bash",
            ToolState = "failed",
            ToolRequest = "{\"command\":\"echo hello\"}",
            ToolError = "invalid_arguments"
        };

        Assert.Equal("Run shell command", message.ToolSummaryText);
        Assert.Equal("Failed", message.ToolStateText);
        Assert.Equal("The operation arguments were invalid.", message.ToolErrorText);
        Assert.True(message.HasToolRequest);
        Assert.True(message.HasToolError);
    }

    [Fact]
    public void ApprovalItemFormatsShellRequestsForReview()
    {
        var payload = JsonNode.Parse("""
        {
          "approval_id": "approval-1",
          "operation": "bash",
          "arguments": {
            "command": "find . -type f -name \"*.cs\" | head -200 && echo done",
            "timeout": 120000,
            "workdir": "src"
          }
        }
        """)!.AsObject();

        var approval = ApprovalItem.FromPayload(payload)!;

        Assert.Equal("Run a shell command", approval.ActionText);
        Assert.Equal("Command", approval.DetailLabel);
        Assert.Equal("find . -type f -name \"*.cs\" | head -200 && echo done", approval.DetailText);
        Assert.Equal("src", approval.WorkingDirectoryText);
        Assert.DoesNotContain("\\u0022", approval.DetailText);
        Assert.Contains("&&", approval.Arguments);
        Assert.DoesNotContain("\\u0026", approval.Arguments);
    }

    [Fact]
    public void ToolDetailsKeepShellOperatorsReadable()
    {
        var snapshot = JsonNode.Parse("""
        {
          "conversationTurns": [
            {
              "turnId": "turn-1",
              "state": "completed",
              "toolUses": [
                {
                  "toolCallId": "tool-1",
                  "name": "bash",
                  "state": "succeeded",
                  "request": {"command": "echo one && echo two"},
                  "result": {"stdout": "one && two"}
                }
              ]
            }
          ]
        }
        """)!.AsObject();

        var message = Assert.Single(DesktopViewModel.ProjectSnapshot(snapshot).Messages);

        Assert.Contains("&&", message.ToolRequest);
        Assert.Contains("&&", message.ToolResult);
        Assert.DoesNotContain("\\u0026", message.ToolRequest);
        Assert.DoesNotContain("\\u0026", message.ToolResult);
    }

    [Fact]
    public void ApprovalItemSummarizesFileTargetsAndKeepsRawRequestReadable()
    {
        var payload = JsonNode.Parse("""
        {
          "approval_id": "approval-2",
          "operation": "write",
          "arguments": {"path":"src/App.cs","content":"class App {}"}
        }
        """)!.AsObject();

        var approval = ApprovalItem.FromPayload(payload)!;

        Assert.Equal("Write to a project file", approval.ActionText);
        Assert.Equal("Target", approval.DetailLabel);
        Assert.Equal("src/App.cs", approval.DetailText);
        Assert.Contains("\n", approval.Arguments);
    }

    [Fact]
    public void ApprovalItemSummarizesWebFetchByUrl()
    {
        var payload = new JsonObject
        {
            ["approval_id"] = "approval-web",
            ["operation"] = "webfetch",
            ["arguments"] = new JsonObject
            {
                ["url"] = "https://example.com/reference",
                ["format"] = "markdown"
            }
        };

        var approval = ApprovalItem.FromPayload(payload)!;

        Assert.Equal("Fetch web content", approval.ActionText);
        Assert.Equal("Web request", approval.OperationText);
        Assert.Equal("URL", approval.DetailLabel);
        Assert.Equal("https://example.com/reference", approval.DetailText);

        var message = new MessageItem
        {
            Role = "tool",
            Text = "webfetch",
            ContentSequence = 1,
            Kind = "tool",
            ToolName = "webfetch"
        };
        Assert.Equal("Fetch web content", message.ToolSummaryText);
    }

    [Fact]
    public void ProjectionRestoresPendingQuestionFromSnapshot()
    {
        var snapshot = JsonNode.Parse("""
        {
          "pendingQuestion": {
            "request_id": "que-1",
            "turn_id": "turn-1",
            "tool_call_id": "call-1",
            "questions": [{
              "header": "Mode",
              "question": "Which mode should I use?",
              "multiple": false,
              "custom": true,
              "options": [{"label":"Fast","description":"Minimize setup"}]
            }]
          },
          "conversationTurns": [{"turnId":"turn-1","state":"resolving_calls","messages":[],"toolUses":[]}]
        }
        """)!.AsObject();

        var projection = DesktopViewModel.ProjectSnapshot(snapshot);

        Assert.Equal("que-1", projection.PendingQuestion?.RequestId);
        var prompt = Assert.Single(projection.PendingQuestion!.Questions);
        Assert.Equal("Which mode should I use?", prompt.Question);
        Assert.True(prompt.AllowCustom);
        Assert.Equal("Fast", Assert.Single(prompt.Options).Label);
    }

    [Fact]
    public void LiveQuestionEventsSetAndClearPendingQuestion()
    {
        using var viewModel = new DesktopViewModel();
        viewModel.ApplyEvent(JsonNode.Parse("""
        {
          "event_type":"question.asked",
          "payload":{"request_id":"que-2","turn_id":"turn-2","tool_call_id":"call-2","questions":[{"header":"Scope","question":"Use project scope?","options":[{"label":"Yes","description":"Keep it local"}]}]}
        }
        """)!.AsObject(), true);

        Assert.Equal("que-2", viewModel.PendingQuestion?.RequestId);
        viewModel.ApplyEvent(JsonNode.Parse("""
        {"event_type":"question.replied","payload":{"request_id":"que-2","turn_id":"turn-2","answers":[["Yes"]]}}
        """)!.AsObject(), true);
        Assert.Null(viewModel.PendingQuestion);
    }

    [Fact]
    public void ProjectionRestoresCurrentTodosFromTheLatestTodoWrite()
    {
        var snapshot = JsonNode.Parse("""
        {
          "conversationTurns": [{
            "turnId": "turn-1",
            "state": "resolving_calls",
            "todos": [
              {"content":"Implement tool","status":"completed","priority":"high","ordinal":0},
              {"content":"Run tests","status":"completed","priority":"medium","ordinal":1}
            ],
            "toolUses": [
              {"toolCallId":"todo-1","name":"todowrite","state":"succeeded","ordinal":1,"result":{"todos":[]}}
            ]
          }, {
            "turnId": "turn-2",
            "state": "resolving_calls",
            "todos": [
              {"content":"Implement tool","status":"completed","priority":"high","ordinal":0},
              {"content":"Run tests","status":"completed","priority":"medium","ordinal":1}
            ],
            "toolUses": [
              {"toolCallId":"todo-3","name":"todowrite","state":"succeeded","ordinal":1,"result":{"todos":[]}}
            ]
          }]
        }
        """)!.AsObject();

        var todos = DesktopViewModel.ProjectSnapshot(snapshot).CurrentTodos;

        Assert.Collection(
            todos,
            item =>
            {
                Assert.Equal("Implement tool", item.Content);
                Assert.Equal("x", item.StatusMarker);
                Assert.Equal(0.58, item.Opacity);
            },
            item =>
            {
                Assert.Equal("Run tests", item.Content);
                Assert.Equal("Completed", item.StatusText);
            });
    }

    [Fact]
    public void LiveTodoUpdatedEventsReplaceCurrentTurnTodos()
    {
        using var viewModel = new DesktopViewModel();

        viewModel.ApplyEvent(JsonNode.Parse("""
        {
          "event_type":"todo.updated",
          "payload":{"turn_id":"turn-0","todos":[
            {"content":"Previous turn","status":"completed","priority":"low"}
          ]}
        }
        """)!.AsObject(), true);
        Assert.Single(viewModel.CurrentTodos);

        viewModel.ApplyEvent(JsonNode.Parse("""
        {
          "event_type":"turn.state",
          "payload":{"turn_id":"turn-1","state":"admitted"}
        }
        """)!.AsObject(), true);
        Assert.Empty(viewModel.CurrentTodos);

        viewModel.ApplyEvent(JsonNode.Parse("""
        {
          "event_type":"todo.updated",
          "payload":{"turn_id":"turn-1","todos":[
            {"content":"Inspect project","status":"in_progress","priority":"high"}
          ]}
        }
        """)!.AsObject(), true);

        var todo = Assert.Single(viewModel.CurrentTodos);
        Assert.Equal("Inspect project", todo.Content);
        Assert.Equal(">", todo.StatusMarker);
        Assert.True(viewModel.HasCurrentTodos);

        viewModel.ApplyEvent(JsonNode.Parse("""
        {
          "event_type":"todo.updated",
          "payload":{"turn_id":"turn-1","todos":[]}
        }
        """)!.AsObject(), true);

        Assert.Empty(viewModel.CurrentTodos);
        Assert.False(viewModel.HasCurrentTodos);
    }

    [Fact]
    public void SnapshotUsesPersistedTodosInsteadOfTodoToolResult()
    {
        var snapshot = JsonNode.Parse("""
        {
          "conversationTurns": [{
            "turnId":"turn-1",
            "state":"resolving_calls",
            "todos":[{"content":"Persisted progress","status":"in_progress","priority":"high","ordinal":0}],
            "toolUses":[{"name":"todowrite","state":"succeeded","result":{"todos":[{"content":"Stale result","status":"pending","priority":"low"}]}}]
          }]
        }
        """)!.AsObject();

        var todo = Assert.Single(DesktopViewModel.ProjectSnapshot(snapshot).CurrentTodos);

        Assert.Equal("Persisted progress", todo.Content);
        Assert.Equal("In progress", todo.StatusText);
    }

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
    public void LegacyProjectionPreservesVisibleAssistantMessages()
    {
        var snapshot = JsonNode.Parse("""
        {
          "messages": [
            {"role":"user","content":[{"type":"text","text":"first question"}]},
            {"role":"assistant","content":[{"type":"text","text":"working"}]},
            {"role":"assistant","content":[{"type":"text","text":"first summary"}]},
            {"role":"user","content":[{"type":"text","text":"second question"}]},
            {"role":"assistant","content":[{"type":"text","text":"second summary"}]}
          ]
        }
        """)!.AsObject();

        var projection = DesktopViewModel.ProjectSnapshot(snapshot);

        Assert.Collection(
            projection.Messages,
            message => Assert.Equal(("user", "first question", 1L), (message.Role, message.Text, message.ContentSequence)),
            message => Assert.Equal(("assistant", "working", 2L), (message.Role, message.Text, message.ContentSequence)),
            message => Assert.Equal(("assistant", "first summary", 3L), (message.Role, message.Text, message.ContentSequence)),
            message => Assert.Equal(("user", "second question", 4L), (message.Role, message.Text, message.ContentSequence)),
            message => Assert.Equal(("assistant", "second summary", 5L), (message.Role, message.Text, message.ContentSequence)));
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

        viewModel.ApplyEvent(TurnState("turn-1", "completed"), live: true);

        Assert.True(message.IsFinalAssistant);
        Assert.True(message.ShowCopy);
    }

    [Fact]
    public void LiveAssistantMessagesRemainAsTurnProcessUntilCompletion()
    {
        var viewModel = new DesktopViewModel();
        viewModel.ApplyEvent(UserMessage("user-1", "turn-1", "question"), live: true);
        viewModel.ApplyEvent(AssistantMessage("assistant-1", "turn-1", "working"), live: true);
        viewModel.ApplyEvent(AssistantMessage("assistant-2", "turn-1", "final summary"), live: true);

        viewModel.ApplyEvent(TurnState("turn-1", "completed"), live: true);

        Assert.Collection(viewModel.Messages,
            message => Assert.True(message.IsUser),
            message =>
            {
                Assert.Equal("working", message.Text);
                Assert.True(message.IsProcess);
                Assert.True(message.IsVisible);
                Assert.False(message.ProcessContentVisible);
                Assert.True(message.ShowProcessToggle);
                Assert.Equal(1, message.ProcessItemCount);
                Assert.False(message.ShowCopy);
            },
            message =>
            {
                Assert.Equal("final summary", message.Text);
                Assert.True(message.IsFinalAssistant);
                Assert.True(message.ShowCopy);
                Assert.False(message.ShowProcessToggle);
            });
    }

    [Fact]
    public void RetainedAssistantMessageIdRemainsDeduplicated()
    {
        var viewModel = new DesktopViewModel();
        var intermediate = AssistantMessage("assistant-1", "turn-1", "working");
        viewModel.ApplyEvent(intermediate, live: true);
        viewModel.ApplyEvent(AssistantMessage("assistant-2", "turn-1", "final summary"), live: true);
        viewModel.ApplyEvent(intermediate, live: true);

        Assert.Collection(
            viewModel.Messages,
            message => Assert.Equal("assistant-1", message.MessageId),
            message => Assert.Equal("assistant-2", message.MessageId));
    }

    [Fact]
    public void NewAssistantStreamAddsTheNextProcessStageAndAppendsDeltas()
    {
        var viewModel = new DesktopViewModel();
        viewModel.ApplyEvent(AssistantMessage("assistant-1", "turn-1", "previous stage"), live: true);
        viewModel.ApplyEvent(AssistantDelta("turn-1", "final "), live: true);
        var streaming = viewModel.Messages.Last();
        viewModel.ApplyEvent(AssistantDelta("turn-1", "summary"), live: true);

        Assert.Collection(
            viewModel.Messages,
            message => Assert.Equal("previous stage", message.Text),
            message =>
            {
                Assert.Same(streaming, message);
                Assert.Equal("final summary", message.Text);
                Assert.True(message.Streaming);
                Assert.True(message.IsProcess);
            });
    }

    [Fact]
    public void FinalAssistantEventKeepsTheStreamingMessageInstance()
    {
        var viewModel = new DesktopViewModel();
        viewModel.ApplyEvent(AssistantDelta("turn-1", "final summary"), live: true);
        var streaming = Assert.Single(viewModel.Messages);

        viewModel.ApplyEvent(AssistantMessage("assistant-1", "turn-1", "final summary"), live: true);

        var completed = Assert.Single(viewModel.Messages);
        Assert.Same(streaming, completed);
        Assert.False(completed.Streaming);
        Assert.Equal("assistant-1", completed.MessageId);
        Assert.True(completed.CanBeFinalAssistant);
    }

    [Fact]
    public void SeparateTurnsKeepSeparateAssistantItems()
    {
        var viewModel = new DesktopViewModel();
        viewModel.ApplyEvent(UserMessage("user-1", "turn-1", "first"), live: true);
        viewModel.ApplyEvent(AssistantMessage("assistant-1", "turn-1", "first summary"), live: true);
        viewModel.ApplyEvent(UserMessage("user-2", "turn-2", "second"), live: true);
        viewModel.ApplyEvent(AssistantMessage("assistant-2", "turn-2", "second summary"), live: true);

        Assert.Collection(
            viewModel.Messages,
            message => Assert.Equal("first", message.Text),
            message => Assert.Equal("first summary", message.Text),
            message => Assert.Equal("second", message.Text),
            message => Assert.Equal("second summary", message.Text));
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
    public void NormalizedTurnSnapshotCollapsesProcessItemsWithoutDeletingThem()
    {
        var snapshot = JsonNode.Parse("""
        {
          "conversationTurns": [
            {
              "turnId":"turn-1",
              "state":"completed",
              "createdAt":"2026-08-22T01:00:00.000Z",
              "messages":[
                {"messageId":"user-1","role":"user","createdAt":"2026-08-22T01:00:01.000Z","message":{"role":"user","content":[{"type":"text","text":"inspect"}]}},
                {"messageId":"assistant-1","role":"assistant","createdAt":"2026-08-22T01:00:01.000Z","message":{"role":"assistant","content":[{"type":"text","text":"reading"}],"tool_calls":[{"call_id":"tool-1","name":"read","arguments":{"path":"README.md"}}]}},
                {"messageId":"assistant-2","role":"assistant","createdAt":"2026-08-22T01:00:01.000Z","message":{"role":"assistant","content":[{"type":"text","text":"done"}]}}
              ],
              "toolUses":[
                {"toolCallId":"tool-1","name":"read","state":"succeeded","createdAt":"2026-08-22T01:00:01.000Z","request":{"path":"README.md"},"result":{"content":"hello"}}
              ]
            }
          ]
        }
        """)!.AsObject();

        var projection = DesktopViewModel.ProjectSnapshot(snapshot);

        Assert.Collection(projection.Messages,
            message => Assert.True(message.IsUser),
            message =>
            {
                Assert.Equal("reading", message.Text);
                Assert.True(message.IsProcess);
                Assert.True(message.IsVisible);
                Assert.False(message.ProcessContentVisible);
                Assert.True(message.ShowProcessToggle);
                Assert.Equal(2, message.ProcessItemCount);
            },
            message =>
            {
                Assert.True(message.IsTool);
                Assert.Equal("tool-1", message.ToolCallId);
                Assert.False(message.IsVisible);
                Assert.False(message.ProcessContentVisible);
            },
            message =>
            {
                Assert.Equal("done", message.Text);
                Assert.True(message.IsFinalAssistant);
                Assert.False(message.ShowProcessToggle);
            });
    }

    [Fact]
    public void AssistantTextWithToolCallsRemainsProcessInsteadOfBecomingFinal()
    {
        var snapshot = JsonNode.Parse("""
        {
          "conversationTurns": [
            {
              "turnId":"turn-1",
              "state":"completed",
              "messages":[
                {
                  "messageId":"assistant-1",
                  "role":"assistant",
                  "createdAt":"2026-08-22T01:00:01.000Z",
                  "message":{
                    "role":"assistant",
                    "content":[{"type":"text","text":"I will inspect it."}],
                    "tool_calls":[{"call_id":"tool-1","name":"read","arguments":{"path":"README.md"}}]
                  }
                },
                {
                  "messageId":"assistant-2",
                  "role":"assistant",
                  "createdAt":"2026-08-22T01:00:03.000Z",
                  "message":{"role":"assistant","content":[{"type":"text","text":"Inspection complete."}]}
                }
              ],
              "toolUses":[
                {"toolCallId":"tool-1","name":"read","state":"succeeded","createdAt":"2026-08-22T01:00:02.000Z"}
              ]
            }
          ]
        }
        """)!.AsObject();

        var projection = DesktopViewModel.ProjectSnapshot(snapshot);

        Assert.Collection(projection.Messages,
            message =>
            {
                Assert.Equal("I will inspect it.", message.Text);
                Assert.True(message.IsProcess);
                Assert.True(message.IsVisible);
                Assert.False(message.ProcessContentVisible);
                Assert.True(message.ShowProcessToggle);
                Assert.False(message.IsFinalAssistant);
            },
            message =>
            {
                Assert.True(message.IsTool);
                Assert.False(message.IsVisible);
            },
            message =>
            {
                Assert.Equal("Inspection complete.", message.Text);
                Assert.True(message.IsFinalAssistant);
                Assert.False(message.ShowProcessToggle);
            });
    }

    [Fact]
    public void TerminalTurnWithoutFinalAssistantKeepsProcessVisible()
    {
        var viewModel = new DesktopViewModel();
        viewModel.ApplyEvent(AssistantWithToolCall("assistant-1", "turn-1", "working"), live: true);
        viewModel.ApplyEvent(ToolEvent("tool.requested", "turn-1", "tool-1", "read", "requested"), live: true);

        viewModel.ApplyEvent(TurnState("turn-1", "failed"), live: true);

        Assert.All(viewModel.Messages, message => Assert.True(message.IsVisible));
        Assert.All(viewModel.Messages, message => Assert.False(message.IsFinalAssistant));
        Assert.All(viewModel.Messages, message => Assert.False(message.ShowProcessToggle));
    }

    [Fact]
    public void ActiveTurnKeepsTimelineExpandedWithoutFinalControls()
    {
        var viewModel = new DesktopViewModel();
        viewModel.ApplyEvent(AssistantMessage("assistant-1", "turn-1", "working"), live: true);
        viewModel.ApplyEvent(ToolEvent("tool.requested", "turn-1", "tool-1", "read", "requested"), live: true);

        viewModel.ApplyEvent(TurnState("turn-1", "resolving_calls"), live: true);

        Assert.All(viewModel.Messages, message => Assert.True(message.IsVisible));
        Assert.All(viewModel.Messages, message => Assert.True(message.IsProcess));
        Assert.All(viewModel.Messages, message => Assert.False(message.ShowCopy));
        Assert.All(viewModel.Messages, message => Assert.False(message.ShowProcessToggle));
    }

    [Fact]
    public void ToggleTurnProcessChangesVisibilityWithoutRemovingItems()
    {
        var viewModel = new DesktopViewModel();
        viewModel.ApplyEvent(UserMessage("user-1", "turn-1", "question"), live: true);
        viewModel.ApplyEvent(AssistantMessage("assistant-1", "turn-1", "working"), live: true);
        viewModel.ApplyEvent(ToolEvent("tool.requested", "turn-1", "tool-1", "read", "requested"), live: true);
        viewModel.ApplyEvent(AssistantMessage("assistant-2", "turn-1", "done"), live: true);
        viewModel.ApplyEvent(TurnState("turn-1", "completed"), live: true);
        var toggle = viewModel.Messages.Single(message => message.ShowProcessToggle);
        var count = viewModel.Messages.Count;

        viewModel.ToggleTurnProcess(toggle);

        Assert.Equal(count, viewModel.Messages.Count);
        Assert.True(toggle.ProcessExpanded);
        Assert.All(viewModel.Messages.Where(message => message.IsProcess), message => Assert.True(message.IsVisible));
        Assert.All(viewModel.Messages.Where(message => message.IsProcess), message => Assert.True(message.ProcessContentVisible));

        viewModel.ToggleTurnProcess(toggle);

        Assert.Equal(count, viewModel.Messages.Count);
        Assert.False(toggle.ProcessExpanded);
        Assert.True(toggle.IsVisible);
        Assert.False(toggle.ProcessContentVisible);
        Assert.All(
            viewModel.Messages.Where(message => message.IsProcess && message != toggle),
            message => Assert.False(message.IsVisible));
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
            [],
            null,
            null,
            string.Empty);

        viewModel.ApplySnapshot(projection);

        Assert.NotSame(original, viewModel.Messages);
        Assert.Equal("new session", Assert.Single(viewModel.Messages).Text);
        Assert.Equal(1, sourceChanges);
        Assert.Equal(1, changes);
    }

    private static JsonObject UserMessage(string messageId, string turnId, string text) =>
        MessageEvent("message.user", "user", messageId, turnId, text);

    private static JsonObject AssistantMessage(string messageId, string turnId, string text) =>
        MessageEvent("message.assistant", "assistant", messageId, turnId, text);

    private static JsonObject AssistantWithToolCall(string messageId, string turnId, string text) =>
        JsonNode.Parse($$$"""
        {
          "event_type":"message.assistant",
          "payload":{
            "message_id":"{{{messageId}}}",
            "turn_id":"{{{turnId}}}",
            "message":{
              "role":"assistant",
              "content":[{"type":"text","text":"{{{text}}}"}],
              "tool_calls":[{"call_id":"tool-1","name":"read","arguments":{"path":"README.md"}}]
            }
          }
        }
        """)!.AsObject();

    private static JsonObject MessageEvent(string eventType, string role, string messageId, string turnId, string text) =>
        JsonNode.Parse($$"""
        {
          "event_type":"{{eventType}}",
          "payload":{
            "message_id":"{{messageId}}",
            "turn_id":"{{turnId}}",
            "message":{"role":"{{role}}","content":[{"type":"text","text":"{{text}}"}]}
          }
        }
        """)!.AsObject();

    private static JsonObject AssistantDelta(string turnId, string text) =>
        JsonNode.Parse($$"""
        {
          "event_type":"assistant.delta",
          "payload":{"turn_id":"{{turnId}}","text":"{{text}}"}
        }
        """)!.AsObject();

    private static JsonObject TurnState(string turnId, string state) =>
        JsonNode.Parse($$"""
        {
          "event_type":"turn.state",
          "payload":{"turn_id":"{{turnId}}","state":"{{state}}"}
        }
        """)!.AsObject();

    private static JsonObject ToolEvent(string eventType, string turnId, string toolCallId, string name, string state) =>
        JsonNode.Parse($$"""
        {
          "event_type":"{{eventType}}",
          "payload":{
            "turn_id":"{{turnId}}",
            "tool_call_id":"{{toolCallId}}",
            "name":"{{name}}",
            "state":"{{state}}",
            "arguments":{"path":"README.md"}
          }
        }
        """)!.AsObject();
}
