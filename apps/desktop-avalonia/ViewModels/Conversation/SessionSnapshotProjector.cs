using System.Text.Json;
using SunCode.Desktop.Models;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

/// <summary>Pure projection of SDK session snapshots into timeline UI models.</summary>
internal static class SessionSnapshotProjector
{
    internal static SessionSnapshotProjection ProjectSnapshot(SessionSnapshot snapshot)
    {
        var messages = new List<MessageItem>();
        var toolActivityTurns = new List<ToolActivityTurnItem>();
        var activities = new List<ActivityItem>();
        IReadOnlyList<TodoItem> currentTodos = [];
        var changedPaths = new List<string>();
        var changedPathSet = new HashSet<string>(StringComparer.Ordinal);
        ApprovalItem? pendingApproval = snapshot.PendingApproval is { } approval
            ? ApprovalItem.FromSdk(approval)
            : null;
        PendingQuestionItem? pendingQuestion = snapshot.PendingQuestion is { } pending
            ? PendingQuestionItem.FromSdk(pending)
            : null;
        var activeTurnId = string.Empty;
        var activeTurnState = string.Empty;
        var imagePayloads = snapshot.Images
            .Where(image => image.ImageId.Length > 0)
            .ToDictionary(image => image.ImageId, StringComparer.Ordinal);

        var conversationTurns = snapshot.ConversationTurns;
        var todoTurnId = conversationTurns
            .Where(turn => !TurnStates.IsTerminal(turn.State))
            .Select(turn => turn.TurnId)
            .LastOrDefault(id => id.Length > 0)
            ?? conversationTurns
                .Select(turn => turn.TurnId)
                .LastOrDefault(id => id.Length > 0)
            ?? string.Empty;
        if (conversationTurns.Count > 0)
        {
            for (var turnIndex = 0; turnIndex < conversationTurns.Count; turnIndex++)
            {
                var turn = conversationTurns[turnIndex];
                var turnId = turn.TurnId;
                var state = turn.State;
                var startedAt = turn.StartedAt ?? string.Empty;
                var completedAt = turn.CompletedAt ?? string.Empty;
                if (!TurnStates.IsTerminal(state)) activeTurnId = turnId;
                activeTurnState = state;
                var toolUses = turn.ToolUses;
                foreach (var path in turn.ChangedPaths ?? [])
                {
                    if (path.Length > 0 && changedPathSet.Add(path)) changedPaths.Add(path);
                }
                if (turnId == todoTurnId)
                    currentTodos = turn.Todos
                        .Select(TodoItem.FromSdk)
                        .Where(item => item is not null)
                        .Select(item => item!)
                        .ToArray();
                var turnMessages = turn.Messages
                    .OrderBy(item => item.CreatedAt, StringComparer.Ordinal)
                    .ToArray();
                var userPreview = turnMessages
                    .Where(item => item.Role == "user")
                    .Select(item => MessageText(item.Message))
                    .FirstOrDefault(text => !string.IsNullOrWhiteSpace(text)) ?? string.Empty;
                var activityTurn = new ToolActivityTurnItem(
                    turnId,
                    turnIndex + 1,
                    state,
                    BoundedPreview(userPreview),
                    turn.CreatedAt,
                    startedAt,
                    completedAt)
                {
                    IsExpanded = !TurnStates.IsTerminal(state)
                };
                foreach (var toolUse in toolUses.OrderBy(item => item.CreatedAt, StringComparer.Ordinal).ThenBy(item => item.Ordinal))
                {
                    activityTurn.Tools.Add(ToolActivityItemFromSdk(toolUse, turnId));
                }
                toolActivityTurns.Add(activityTurn);
                foreach (var item in turnMessages)
                {
                    var role = item.Role;
                    var message = item.Message;
                    var text = MessageText(message);
                    if (role is "user" or "assistant" && !string.IsNullOrWhiteSpace(text))
                    {
                        messages.Add(new MessageItem
                        {
                            MessageId = item.MessageId,
                            Role = role,
                            Text = text,
                            ContentSequence = messages.Count + 1,
                            TurnId = turnId,
                            Attachments = role == "user" ? MessageAttachments(message, imagePayloads) : [],
                            CanBeFinalAssistant = role == "assistant" && (message.ToolCalls?.Count ?? 0) == 0,
                            IsFinalAssistant = role == "assistant" && (message.ToolCalls?.Count ?? 0) == 0,
                            IsVisible = true,
                            TurnSequence = turnIndex + 1,
                            TurnPreview = BoundedPreview(userPreview)
                        });
                    }
                }
                var finalAssistant = messages.LastOrDefault(item => item.TurnId == turnId && item.IsAssistant && item.CanBeFinalAssistant);
                foreach (var assistant in messages.Where(item => item.TurnId == turnId && item.IsAssistant))
                    assistant.IsFinalAssistant = false;
                if (finalAssistant is not null)
                {
                    finalAssistant.IsFinalAssistant = true;
                    finalAssistant.DurationText = FormatDuration(activityTurn.StartedAt, activityTurn.CompletedAt, state);
                    finalAssistant.CompletionTimeText = FormatCompletionTime(activityTurn.CompletedAt);
                }
                foreach (var toolUse in toolUses.OrderBy(item => item.CreatedAt, StringComparer.Ordinal).ThenBy(item => item.Ordinal))
                {
                    var toolMessage = ToolMessageItem(toolUse, turnId, messages.Count + 1);
                    toolMessage.IsVisible = false;
                    messages.Add(toolMessage);
                }
            }
        }
        else
        {
            foreach (var item in snapshot.Messages)
            {
                var role = item.Role;
                if (role is not ("user" or "assistant")) continue;
                var text = MessageText(item);
                if (string.IsNullOrWhiteSpace(text)) continue;
                messages.Add(new MessageItem
                {
                    Role = role,
                    Text = text,
                    ContentSequence = messages.Count + 1,
                    IsVisible = true,
                    Attachments = role == "user" ? MessageAttachments(item, imagePayloads) : [],
                    IsFinalAssistant = role == "assistant" && (item.ToolCalls?.Count ?? 0) == 0,
                    CanBeFinalAssistant = role == "assistant" && (item.ToolCalls?.Count ?? 0) == 0
                });
            }
        }

        return new SessionSnapshotProjection(messages, toolActivityTurns, activities, changedPaths, currentTodos, pendingApproval, pendingQuestion, activeTurnId, activeTurnState);
    }

    internal static ToolActivityItem ToolActivityItemFromSdk(SessionCallToolUse item, string turnId) => new(
        turnId,
        item.ToolCallId,
        item.Name,
        item.State,
        JsonText(item.Request),
        JsonText(item.Result),
        string.Empty,
        item.ErrorCode ?? string.Empty,
        item.CreatedAt);

    internal static MessageItem ToolMessageItem(SessionCallToolUse item, string turnId, long sequence) => new()
    {
        Role = "tool",
        Kind = "tool",
        Text = item.Name,
        ContentSequence = sequence,
        TurnId = turnId,
        ToolCallId = item.ToolCallId,
        ToolName = item.Name,
        ToolState = item.State,
        ToolRequest = JsonText(item.Request),
        ToolError = item.ErrorCode ?? string.Empty,
        IsProcess = true,
        IsVisible = false
    };

    internal static string BoundedPreview(string text)
    {
        const int previewLength = 72;
        var compact = string.Join(" ", text.Split((char[]?)null, StringSplitOptions.RemoveEmptyEntries));
        return compact.Length <= previewLength ? compact : compact[..previewLength].TrimEnd() + "...";
    }

    internal static string FormatDuration(string startedAt, string completedAt, string state = "")
    {
        var startText = string.IsNullOrWhiteSpace(startedAt) ? string.Empty : startedAt;
        if (!DateTimeOffset.TryParse(startText, out var started)) return string.Empty;
        var hasCompleted = DateTimeOffset.TryParse(completedAt, out var completed);
        if (TurnStates.IsTerminal(state) && !hasCompleted) return string.Empty;
        var ended = hasCompleted ? completed : DateTimeOffset.Now;
        if (ended < started) ended = started;
        var elapsed = ended - started;
        if (elapsed.TotalSeconds < 1) return $"{elapsed.TotalMilliseconds:0} ms";
        if (elapsed.TotalMinutes < 1) return $"{elapsed.TotalSeconds:0.#} s";
        return $"{elapsed.TotalMinutes:0.#} m";
    }

    internal static string FormatCompletionTime(string completedAt)
    {
        return DateTimeOffset.TryParse(completedAt, out var completed)
            ? completed.ToLocalTime().ToString("HH:mm")
            : string.Empty;
    }

    internal static string MessageText(AgentMessage message) => message.Text;

    internal static IReadOnlyList<ComposerAttachment> MessageAttachments(
        AgentMessage message,
        IReadOnlyDictionary<string, SessionImage> images)
    {
        var attachments = new List<ComposerAttachment>();
        foreach (var imageId in MessageImageIds(message))
        {
            if (images.TryGetValue(imageId, out var payload))
                attachments.Add(ComposerAttachment.FromSdk(payload));
        }
        return attachments;
    }

    internal static IReadOnlyList<string> MessageImageIds(AgentMessage message) =>
        (message.Content ?? [])
            .Where(part => part.Kind == "image_ref")
            .Select(part => part.Text)
            .Where(imageId => imageId.Length > 0)
            .ToArray();

    internal static string JsonText(JsonElement? node) =>
        node is null || node.Value.ValueKind is JsonValueKind.Undefined or JsonValueKind.Null
            ? string.Empty
            : node.Value.GetRawText();
}
