using SunCode.Desktop.Infrastructure;

namespace SunCode.Desktop.Models;

public sealed record ChildSessionItem(
    string SessionId,
    string ParentSessionId,
    string Title,
    string AgentId,
    string AgentDisplayName,
    string State,
    string CreatedAt,
    string ModelId,
    string Task,
    string Result,
    string ApprovalId)
{
    public string StateText => State switch
    {
        "running" => LocalizationService.GetString("LocRunning", "Running"),
        "approval" or "awaiting_approval" => LocalizationService.GetString("LocWaitingForApproval", "Waiting for approval"),
        "completed" or "idle" => LocalizationService.GetString("LocCompleted", "Completed"),
        "failed" => LocalizationService.GetString("LocFailed", "Failed"),
        "cancelled" => LocalizationService.GetString("LocCancelled", "Cancelled"),
        "interrupted" => LocalizationService.GetString("LocInterrupted", "Interrupted"),
        _ => State
    };
    public bool IsRunning => State == "running";
    public bool IsApproval => State is "approval" or "awaiting_approval";
    public bool IsCompleted => State is "completed" or "idle";
    public bool IsFailed => TurnStates.IsUnsuccessfulTerminal(State);
    public bool HasApproval => IsApproval && !string.IsNullOrWhiteSpace(ApprovalId);
    public string RelativeActivity => SessionItem.RelativeActivityFor(CreatedAt);
}

public sealed record ChildSessionTimelineItem(
    string Kind,
    string Title,
    string Detail,
    string State,
    string CreatedAt)
{
    public bool IsTool => Kind == "tool";
}
