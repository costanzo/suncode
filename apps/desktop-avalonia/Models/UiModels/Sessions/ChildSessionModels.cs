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
        "running" => LocalizationService.GetString("Loc_Running", "Running"),
        "approval" or "awaiting_approval" => LocalizationService.GetString("Loc_WaitingForApproval", "Waiting for approval"),
        "completed" or "idle" => LocalizationService.GetString("Loc_Completed", "Completed"),
        "failed" => LocalizationService.GetString("Loc_Failed", "Failed"),
        "cancelled" => LocalizationService.GetString("Loc_Cancelled", "Cancelled"),
        "interrupted" => LocalizationService.GetString("Loc_Interrupted", "Interrupted"),
        _ => State
    };
    public bool IsRunning => State == "running";
    public bool IsApproval => State is "approval" or "awaiting_approval";
    public bool IsCompleted => State is "completed" or "idle";
    public bool IsFailed => State is "failed" or "cancelled" or "interrupted";
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
    public bool IsTask => Kind == "task";
    public bool IsMessage => Kind == "message";
    public bool IsTool => Kind == "tool";
}
