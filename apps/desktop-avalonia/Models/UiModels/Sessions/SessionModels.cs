using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Text.Encodings.Web;
using System.Text.Json;
using System.Text.Json.Nodes;
using Avalonia.Media.Imaging;
using SunCode.Desktop.Infrastructure;

namespace SunCode.Desktop.Models;

public sealed record SessionItem(string SessionId, string Title, string LastActivityAt, bool IsPinned, string AgentState = "idle")
{
    public string DisplayTitle => string.IsNullOrWhiteSpace(Title) ? "Untitled session" : Title;
    public string RelativeActivity
    {
        get
        {
            if (!DateTimeOffset.TryParse(LastActivityAt, out var timestamp)) return "No activity yet";
            var elapsed = DateTimeOffset.Now - timestamp;
            if (elapsed.TotalMinutes < 1) return "Just now";
            if (elapsed.TotalHours < 1) return $"{(int)elapsed.TotalMinutes}m ago";
            if (elapsed.TotalDays < 1) return $"{(int)elapsed.TotalHours}h ago";
            if (elapsed.TotalDays < 7) return $"{(int)elapsed.TotalDays}d ago";
            return timestamp.ToString("d");
        }
    }
    public bool IsRunning => AgentState == "running";
    public bool IsWaitingForApproval => AgentState == "approval";
    public bool IsWaitingForAnswer => AgentState == "question";
    public bool IsFailed => AgentState == "failed";
    public bool HasAgentState => AgentState != "idle";
    public string AgentStateLabel => AgentState switch
    {
        "running" => "Agent running",
        "approval" => "Waiting for approval",
        "question" => "Waiting for answer",
        "failed" => "Turn failed",
        _ => "Agent idle"
    };
}

public sealed record ProviderItem(string Id, string DisplayName, bool Configured, string ApiBase = "")
{
    public string Display => Configured ? DisplayName : $"{DisplayName} (needs key)";
}

public sealed record ModelItem(
    string Id,
    string Provider,
    string ProviderLabel,
    string Availability,
    bool SupportsReasoningEffort,
    bool SupportsVision = false,
    string ApiBase = "",
    IReadOnlyList<string>? ReasoningEfforts = null)
{
    public bool Configured => Availability == "configured";
    // Availability is represented by the composer state after selection; keep
    // model names clean in dropdown options instead of appending status text.
    public string Display => Id;
}

public sealed record CredentialItem(string Provider, bool Configured);
