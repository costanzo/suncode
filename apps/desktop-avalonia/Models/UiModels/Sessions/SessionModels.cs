using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Text.Encodings.Web;
using System.Text.Json;
using Avalonia.Media.Imaging;
using SunCode.Desktop.Infrastructure;

namespace SunCode.Desktop.Models;

public sealed record SessionItem(string SessionId, string Title, string LastActivityAt, bool IsPinned, string AgentState = "idle", string ModelId = "", string ReasoningEffort = "", bool IsArchived = false)
{
    public string DisplayTitle => string.IsNullOrWhiteSpace(Title) ? "Untitled session" : Title;
    public string RelativeActivity => RelativeActivityFor(LastActivityAt);
    internal static string RelativeActivityFor(string value)
    {
            if (!DateTimeOffset.TryParse(value, out var timestamp)) return "No activity yet";
            var elapsed = DateTimeOffset.Now - timestamp;
            if (elapsed.TotalMinutes < 1) return "Just now";
            if (elapsed.TotalHours < 1) return $"{(int)elapsed.TotalMinutes}m ago";
            if (elapsed.TotalDays < 1) return $"{(int)elapsed.TotalHours}h ago";
            if (elapsed.TotalDays < 7) return $"{(int)elapsed.TotalDays}d ago";
            return timestamp.ToString("d");
    }
    public bool IsRunning => AgentState == "running";
    public bool IsWaitingForApproval => AgentState == "approval";
    public bool IsWaitingForAnswer => AgentState == "question";
    public bool IsFailed => AgentState == "failed";
    public bool HasAgentState => AgentState != "idle";
    public bool IsTurnActive => IsRunning || IsWaitingForApproval || IsWaitingForAnswer;
    public bool IsWaitingOnUser => IsWaitingForApproval || IsWaitingForAnswer;
    public string AgentStateLabel => AgentState switch
    {
        "running" => "Agent running",
        "approval" => "Waiting for approval",
        "question" => "Waiting for answer",
        "failed" => "Turn failed",
        _ => "Agent idle"
    };
}

public sealed record ProviderItem(string Id, string DisplayName, bool Configured, string ApiBase = "", string DefaultApiBase = "")
{
    public string Display => Configured ? DisplayName : $"{DisplayName} (needs key)";
    public string StatusText => Configured ? "Ready" : "API key needed";
}

public sealed record ProviderModelItem(string Display, bool Configured)
{
    public string StatusText => Configured ? "Ready to use" : "Add API key to use";
}

public sealed record ModelItem(
    string Id,
    string Provider,
    string ProviderLabel,
    string Availability,
    bool SupportsReasoningEffort,
    bool SupportsVision = false,
    bool SupportsComputerUse = false,
    string ApiBase = "",
    string DefaultApiBase = "",
    IReadOnlyList<string>? ReasoningEfforts = null)
{
    public bool Configured => Availability == "configured";
    // Availability is represented by the composer state after selection; keep
    // model names clean in dropdown options instead of appending status text.
    public string Display => Id;
}

public sealed record CredentialItem(string Provider, bool Configured);

public sealed record AgentItem(
    string Id,
    string Name,
    string DisplayName,
    string Description,
    long Version,
    IReadOnlyList<string> AllowedTools,
    string ModelPolicy,
    string McpPolicy,
    bool CanDelegate,
    uint ToolCallLimit)
{
    public string AllowedToolsText => string.Join("  ", AllowedTools);
    public string DelegateText => CanDelegate ? "Allowed" : "Not allowed";
    public string ToolLimitText => $"{ToolCallLimit} calls";
}
