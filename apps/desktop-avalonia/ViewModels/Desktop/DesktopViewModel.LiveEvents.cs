using Avalonia.Threading;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

public sealed partial class DesktopViewModel
{
    private void OnNativeEvent(string sessionId, AgentEvent value) => Dispatcher.UIThread.Post(() =>
    {
        if (_disposed || SelectedSession?.SessionId != sessionId)
        {
            return;
        }
        if (value.EventType == AgentEventTypes.ResyncRequired)
        {
            LogSession("event", sessionId, $"{AgentEventTypes.ResyncRequired} reload_begin");
            RunInBackground(() => ReloadCurrentSessionAsync(sessionId), "session.reload");
            return;
        }
        ApplyEvent(value, true);
    });

    private async Task ReloadCurrentSessionAsync(string sessionId)
    {
        if (SelectedSession?.SessionId != sessionId) return;
        _loadedSessionId = null;
        CloseSubscription();
        await SelectSessionAsync(SelectedSession);
    }

    /// <summary>
    /// Routes one agent event to its family handler. Provider traffic and usage
    /// are applied first; live background refreshes are scheduled last.
    /// </summary>
    internal void ApplyEvent(AgentEvent value, bool live)
    {
        var type = value.EventType;
        var payload = value.Payload;
        if (type == AgentEventTypes.ProviderRetryScheduled && IsTurnActive && payload.TurnId == ActiveTurnId)
        {
            StartProviderRetry(payload);
        }
        else if (payload.TurnId == ActiveTurnId && (type is AgentEventTypes.ProviderExchangeStarted or AgentEventTypes.TurnState))
        {
            ClearProviderRetryStatus();
        }
        var text = type == AgentEventTypes.ProviderRetryScheduled ? ProviderRetryText : EventText(type, payload);
        var providerExchange = AgentEventTypes.IsProviderExchange(type);
        if (providerExchange)
        {
            ProviderTraffic.Apply(value);
            ObserveProviderFailure(type, payload);
            if (payload.Usage is { } usage) UpdateContextUsage(usage);
        }

        if (type == AgentEventTypes.AssistantDelta || AgentEventTypes.IsMessage(type)) ApplyMessageEvent(type, payload, text);
        else if (AgentEventTypes.IsTool(type)) ApplyToolEvent(type, payload, text);
        else if (type == AgentEventTypes.TodoUpdated) ApplyTodoEvent(type, payload, text);
        else if (type == AgentEventTypes.ContextCompacted) ApplyCompactionEvent(type, payload, text);
        else if (!providerExchange) AddActivity(type, text, payload.State ?? string.Empty, payload.Operation ?? string.Empty);

        var fileChanged = type == AgentEventTypes.CheckpointCaptured && ApplyTurnChangeEvent(payload);
        if (AgentEventTypes.IsPrompt(type)) ApplyPromptEvent(type, payload, live);
        if (type == AgentEventTypes.TurnState) ApplyTurnStateEvent(payload, value.OccurredAt);
        if (type == AgentEventTypes.TurnState && payload.State is "completed" or "failed" or "cancelled" or "interrupted")
            RunInBackground(() => LoadQueuedInputsAsync(), "queue.refresh");
        if (type == AgentEventTypes.TurnQueued)
            RunInBackground(() => LoadQueuedInputsAsync(), "queue.refresh");
        if (type == AgentEventTypes.MessageUser && !string.IsNullOrWhiteSpace(payload.QueuedId))
            RunInBackground(() => LoadQueuedInputsAsync(), "queue.steering_consumed");

        if (live) ScheduleLiveRefreshes(type, payload, fileChanged);
    }

    private void ApplyMessageEvent(string type, AgentEventPayload payload, string text)
    {
        if (type == AgentEventTypes.AssistantDelta) ApplyAssistantDelta(payload);
        else ApplyFinalMessage(type, payload, text);
    }

    private void ApplyAssistantDelta(AgentEventPayload payload)
    {
        var turnId = payload.TurnId ?? string.Empty;
        var assistant = Messages.LastOrDefault(message =>
            message.TurnId == turnId && message.Role == "assistant" && message.Streaming);
        var delta = payload.Text ?? string.Empty;
        if (assistant is null)
        {
            if (delta.Length > 0)
            {
                var activityTurn = EnsureToolActivityTurn(turnId);
                Messages.Add(new MessageItem
                {
                    Role = "assistant",
                    Text = delta,
                    ContentSequence = Messages.Count + 1,
                    TurnId = turnId,
                    TurnSequence = activityTurn.Sequence,
                    TurnPreview = activityTurn.Preview,
                    Streaming = true,
                    IsProcess = true,
                    CanBeFinalAssistant = false
                });
            }
        }
        else if (delta.Length > 0)
        {
            assistant.Text += delta;
        }
        if (delta.Length > 0)
        {
            // The collection itself does not change when a streaming
            // assistant row receives another text delta. Notify the chat
            // surface so it can keep the viewport at the latest content.
            OnPropertyChanged(nameof(Messages));
            NotifyAssistantStreamingChanged();
        }
    }

    private void ApplyFinalMessage(string type, AgentEventPayload payload, string text)
    {
        var turnId = payload.TurnId ?? string.Empty;
        var messageId = payload.MessageId ?? string.Empty;
        var message = payload.Message ?? EmptyMessage;
        var isAssistant = type == AgentEventTypes.MessageAssistant;
        var canBeFinalAssistant = isAssistant && (message.ToolCalls?.Count ?? 0) == 0;
        var changed = false;
        var streaming = isAssistant
            ? Messages.LastOrDefault(message =>
                message.TurnId == turnId && message.Role == "assistant" && message.Streaming)
            : null;
        if (messageId.Length > 0 && !_appliedMessageIds.Add(messageId))
        {
            DiagnosticLog.Debug("session.message", $"duplicate ignored type={type} message={messageId} turn={turnId}");
        }
        else if (type == AgentEventTypes.MessageUser)
        {
            EnsureToolActivityTurn(turnId, text);
            Messages.Add(new MessageItem
            {
                MessageId = messageId,
                Role = "user",
                Text = text,
                ContentSequence = Messages.Count + 1,
                TurnId = turnId,
                IsVisible = true,
                Attachments = PendingMessageAttachments(message)
            });
            changed = true;
        }
        else if (!string.IsNullOrWhiteSpace(text))
        {
            if (streaming is not null)
            {
                streaming.MessageId = messageId;
                streaming.Text = text;
                streaming.Streaming = false;
                streaming.CanBeFinalAssistant = canBeFinalAssistant;
            }
            else
            {
                var activityTurn = EnsureToolActivityTurn(turnId);
                foreach (var previous in Messages.Where(item => item.TurnId == turnId && item.IsAssistant))
                    previous.IsFinalAssistant = false;
                Messages.Add(new MessageItem
                {
                    MessageId = messageId,
                    Role = "assistant",
                    Text = text,
                    ContentSequence = Messages.Count + 1,
                    TurnId = turnId,
                    CanBeFinalAssistant = canBeFinalAssistant,
                    IsFinalAssistant = false,
                    TurnSequence = activityTurn.Sequence,
                    TurnPreview = activityTurn.Preview
                });
            }
            changed = true;
        }
        else if (streaming is not null)
        {
            streaming.MessageId = messageId;
            streaming.Streaming = false;
            streaming.CanBeFinalAssistant = canBeFinalAssistant;
            changed = true;
        }
        if (changed)
        {
            OnPropertyChanged(nameof(Messages));
            OnPropertyChanged(nameof(HasMessages));
            NotifyAssistantStreamingChanged();
        }
    }

    private void ApplyToolEvent(string type, AgentEventPayload payload, string text)
    {
        UpsertToolActivity(type, payload);
        AddActivity(type, text, payload.State ?? string.Empty, payload.Name ?? string.Empty);
    }

    private void UpsertToolActivity(string type, AgentEventPayload payload)
    {
        var turnId = payload.TurnId ?? string.Empty;
        var toolCallId = payload.ToolCallId ?? string.Empty;
        if (turnId.Length == 0 || toolCallId.Length == 0) return;
        var turn = EnsureToolActivityTurn(turnId);
        var existing = turn.Tools.FirstOrDefault(tool => tool.ToolCallId == toolCallId);
        var state = type == AgentEventTypes.ToolState
            ? payload.State ?? string.Empty
            : existing?.State ?? "requested";
        var name = payload.Name ?? string.Empty;
        if (name.Length == 0) name = existing?.Name ?? "tool";
        var request = type == AgentEventTypes.ToolRequested
            ? Pretty(payload.Arguments)
            : existing?.Request ?? string.Empty;
        var result = type == AgentEventTypes.ToolResult
            ? Pretty(payload.Result)
            : existing?.Result ?? string.Empty;
        var output = type == AgentEventTypes.ToolOutput
            ? AppendBoundedOutput(existing?.Output ?? string.Empty, DecodeOutputChunk(payload))
            : existing?.Output ?? string.Empty;
        var error = type == AgentEventTypes.ToolState
            ? payload.Reason ?? string.Empty
            : existing?.Error ?? string.Empty;
        ToolActivity.Upsert(turn, toolCallId, name, state, request, result, output, error);
        SyncActiveToolRow();
    }

    private static string DecodeOutputChunk(AgentEventPayload payload)
    {
        var encoded = payload.ChunkBase64 ?? string.Empty;
        if (encoded.Length == 0) return string.Empty;
        try { return System.Text.Encoding.UTF8.GetString(Convert.FromBase64String(encoded)); }
        catch (FormatException) { return string.Empty; }
    }

    private static string AppendBoundedOutput(string existing, string chunk)
    {
        const int maxCharacters = 256 * 1024;
        if (existing.Length >= maxCharacters || chunk.Length == 0) return existing;
        var remaining = maxCharacters - existing.Length;
        return existing + (chunk.Length <= remaining ? chunk : chunk[..remaining]);
    }

    private void ApplyTodoEvent(string type, AgentEventPayload payload, string text)
    {
        CurrentTodos.ReplaceAll((payload.Todos ?? [])
            .Select(TodoItem.FromSdk)
            .Where(item => item is not null)
            .Select(item => item!)
            .ToArray());
        OnPropertyChanged(nameof(HasCurrentTodos));
        OnPropertyChanged(nameof(IsReviewTodosVisible));
        AddActivity(type, text, payload.State ?? string.Empty, "todowrite");
    }

    private void ApplyCompactionEvent(string type, AgentEventPayload payload, string text)
    {
        Messages.Add(new MessageItem
        {
            Role = "assistant",
            Kind = AgentEventTypes.ContextCompacted,
            Text = text,
            ContentSequence = Messages.Count + 1,
            TurnId = payload.TurnId ?? string.Empty,
            IsProcess = true
        });
        OnPropertyChanged(nameof(Messages));
        OnPropertyChanged(nameof(HasMessages));
        NotifyAssistantStreamingChanged();
    }

    private void AddActivity(string type, string text, string state, string operation)
    {
        Activities.Add(new ActivityItem(type, text, Activities.Count + 1, state, operation));
        OnPropertyChanged(nameof(HasActivities));
        OnPropertyChanged(nameof(LatestActivityText));
    }
    
    /// <summary>
    /// Upserts one file's net change from <c>checkpoint.captured</c>. The payload
    /// carries the turn baseline and latest version, so a path whose content is
    /// back at its baseline, or that was created and then deleted in the same
    /// turn, drop out. A new turn replaces the sidebar changes.
    /// </summary>
    private bool ApplyTurnChangeEvent(AgentEventPayload payload)
    {
        var turnId = payload.TurnId ?? string.Empty;
        var path = payload.Path ?? string.Empty;
        if (turnId.Length == 0 || path.Length == 0) return false;
        var changes = turnId == TurnChangesTurnId ? TurnChanges.ToList() : [];
        var index = changes.FindIndex(change => change.Path == path);
        var unchanged = payload.BaselineSha256 == payload.CurrentSha256;
        var change = new TurnChangeItem(
            path,
            payload.BaselineSha256 is null ? "added" : payload.CurrentSha256 is null ? "deleted" : "modified",
            payload.Additions ?? 0,
            payload.Deletions ?? 0,
            payload.Binary ?? false);
        if (unchanged)
        {
            if (index >= 0) changes.RemoveAt(index);
        }
        else if (index >= 0) changes[index] = change;
        else changes.Add(change);
        SetTurnChanges(turnId, changes);
        ApplyTurnChangesToFinalAssistant(turnId);
        NotifyReviewPresentationChanged();
        return true;
    }
    
    private void ApplyTurnChangesToFinalAssistant(string turnId)
    {
        if (turnId.Length == 0 || turnId != TurnChangesTurnId) return;
        var finalAssistant = Messages.LastOrDefault(item => item.TurnId == turnId && item.IsFinalAssistant);
        finalAssistant?.SetTurnChanges(TurnChanges);
    }

    private void ApplyPromptEvent(string type, AgentEventPayload payload, bool live)
    {
        switch (type)
        {
            case AgentEventTypes.ApprovalRequested:
                PendingApproval = ApprovalItem.FromSdk(payload);
                break;
            case AgentEventTypes.ApprovalResolved:
                if (payload.Decision == "allow_session") FullControlEnabled = true;
                PendingApproval = null;
                break;
            case AgentEventTypes.QuestionAsked:
                PendingQuestion = PendingQuestionItem.FromSdk(payload);
                break;
            case AgentEventTypes.QuestionReplied or AgentEventTypes.QuestionRejected:
                if (PendingQuestion?.RequestId == payload.RequestId) PendingQuestion = null;
                break;
        }
        if (live) SyncSelectedSessionAgentStateFromReview();
    }

    private void ApplyTurnStateEvent(AgentEventPayload payload, string occurredAt)
    {
        var state = payload.State ?? string.Empty;
        var turnId = payload.TurnId ?? string.Empty;
        var terminal = IsTerminalTurnState(state);
        if (!string.IsNullOrWhiteSpace(turnId)) LastTurnId = turnId;
        if (payload.SubmissionIdempotencyKey is { } submissionKey && _inFlightSubmissions.ContainsKey(submissionKey))
            _inFlightSubmissions[submissionKey] = true;
        if (state == "admitted")
        {
            CurrentTodos.Clear();
            OnPropertyChanged(nameof(HasCurrentTodos));
            OnPropertyChanged(nameof(IsReviewTodosVisible));
            if (turnId.Length > 0 && turnId != TurnChangesTurnId) SetTurnChanges(turnId, []);
        }
        ActiveTurnId = terminal ? string.Empty : turnId;
        ActiveTurnState = state;
        if (state == "failed") SetReviewFailure(turnId, payload.Reason);
        else ReviewFailure = null;
        var activityTurn = EnsureToolActivityTurn(turnId);
        activityTurn.SetTiming(
            payload.StartedAt,
            terminal ? (payload.CompletedAt is { Length: > 0 } completed ? completed : occurredAt) : null);
        if (string.IsNullOrWhiteSpace(activityTurn.StartedAt) && state == "admitted")
            activityTurn.SetTiming(occurredAt, null);
        activityTurn.Update(state);
        UpdateActiveTurnTiming(turnId, activityTurn.StartedAt);
        if (!terminal) activityTurn.IsExpanded = true;
        if (terminal)
        {
            var finalAssistant = Messages.LastOrDefault(item => item.TurnId == turnId && item.IsAssistant && item.CanBeFinalAssistant);
            if (finalAssistant is not null)
            {
                finalAssistant.DurationText = FormatDuration(activityTurn.StartedAt, activityTurn.CompletedAt, state);
                finalAssistant.IsFinalAssistant = true;
                finalAssistant.CompletionTimeText = FormatCompletionTime(activityTurn.CompletedAt);
                ApplyTurnChangesToFinalAssistant(turnId);
            }
        }
        SyncActiveToolRow();
        ToolActivity.NotifyTurnsChanged();
    }

    /// <summary>Schedules the background reloads a live event implies, after all state is applied.</summary>
    private void ScheduleLiveRefreshes(string type, AgentEventPayload payload, bool fileChanged)
    {
        if (type == AgentEventTypes.TurnState) SyncSelectedSessionAgentStateFromReview();
        if (AgentEventTypes.IsProviderExchange(type) && ProviderTraceVisible
            || type == AgentEventTypes.TurnState && payload.State == "failed")
            RunInBackground(ProviderTrace.RefreshAsync, "provider_trace.refresh");
        if (type == AgentEventTypes.ToolResult
            || (type == AgentEventTypes.TurnState && IsTerminalTurnState(payload.State ?? string.Empty)))
            RunInBackground(LoadChildSessionsAsync, "child_sessions.load");
        if (Git.IsAvailable && (fileChanged || type == AgentEventTypes.TurnState && IsTerminalTurnState(payload.State ?? string.Empty)))
            RunInBackground(Git.RefreshAsync, "git.refresh");
        if (fileChanged && Layout.TurnDiffVisible && TurnDiff.TurnId == payload.TurnId && SelectedSession is { } session)
            RunInBackground(() => TurnDiff.LoadAsync(session.SessionId, TurnDiff.TurnId), "turn_diff.refresh");
    }

    private static string EventText(string type, AgentEventPayload payload)
    {
        var messageText = payload.Message is { } message ? MessageText(message) : string.Empty;
        if (type is AgentEventTypes.MessageUser or AgentEventTypes.MessageAssistant or AgentEventTypes.MessageTool
            || !string.IsNullOrEmpty(messageText)) return messageText;
        return type switch
        {
            AgentEventTypes.ApprovalRequested => $"Approval required for {payload.Operation}",
            AgentEventTypes.QuestionAsked => "Waiting for an answer",
            AgentEventTypes.QuestionReplied => "Question answered",
            AgentEventTypes.QuestionRejected => "Question skipped",
            AgentEventTypes.TodoUpdated => "Todo list updated",
            AgentEventTypes.ContextCompacted => $"Context compacted · retained {payload.RetainedTokens} tokens",
            AgentEventTypes.CheckpointCaptured => $"Changed {payload.Path}",
            AgentEventTypes.TurnState => $"Turn {payload.State}",
            AgentEventTypes.AssistantDelta => payload.Text ?? string.Empty,
            AgentEventTypes.ToolOutput => $"Command output · {payload.Stream}",
            _ => type
        };
    }

    private static readonly AgentMessage EmptyMessage = new(string.Empty, [], [], null);

    private IReadOnlyList<ComposerAttachment> PendingMessageAttachments(AgentMessage message)
    {
        var imageIds = (message.Content ?? [])
            .Where(part => part.Kind == "image_ref")
            .Select(part => part.Text)
            .ToHashSet(StringComparer.Ordinal);
        if (imageIds.Count == 0) return [];
        var attachments = _submittedAttachments.Where(item => imageIds.Contains(item.ImageId)).ToArray();
        _submittedAttachments = _submittedAttachments.Where(item => !imageIds.Contains(item.ImageId)).ToArray();
        foreach (var attachment in attachments)
        {
            // Transfer ownership from the composer to the live user message without
            // disposing the shared preview bitmap.
            ComposerAttachments.Remove(attachment);
        }
        return attachments;
    }
}
