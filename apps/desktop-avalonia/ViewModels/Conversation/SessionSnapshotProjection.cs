using SunCode.Desktop.Models;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

internal sealed record SessionSnapshotProjection(
    IReadOnlyList<MessageItem> Messages,
    IReadOnlyList<ToolActivityTurnItem> ToolActivityTurns,
    IReadOnlyList<ActivityItem> Activities,
    IReadOnlyList<string> ChangedPaths,
    IReadOnlyList<TodoItem> CurrentTodos,
    ApprovalItem? PendingApproval,
    PendingQuestionItem? PendingQuestion,
    string ActiveTurnId,
    string ActiveTurnState = "",
    bool HasMoreMessages = false,
    string? OldestMessageCreatedAt = null,
    string? OldestMessageId = null,
    IReadOnlyDictionary<string, SessionImage>? Images = null);
