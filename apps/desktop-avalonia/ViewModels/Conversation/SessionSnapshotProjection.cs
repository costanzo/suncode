using SunCode.Desktop.Models;

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
    string ActiveTurnState = "");
