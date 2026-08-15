using System.Text.Json.Nodes;

namespace SunCode.Desktop.Models;

public sealed record ProjectItem(string ProjectId, string DisplayName, string CanonicalRoot);

public sealed record SessionItem(string SessionId, string Title, string LastActivityAt)
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
}

public sealed record ModelItem(string Id, string Provider, string Availability)
{
    public bool Configured => Availability == "configured";
    public string Display => Configured ? Id : $"{Id} (needs key)";
}

public sealed record CredentialItem(string Provider, bool Configured);

public sealed class MessageItem
{
    public required string Role { get; init; }
    public required string Text { get; set; }
    public required long ContentSequence { get; set; }
    public string TurnId { get; init; } = string.Empty;
    public bool Streaming { get; set; }
    public bool IsUser => Role == "user";
    public string Author => IsUser ? "You" : "SunCode";
}

public sealed record ActivityItem(string EventType, string Text, long ContentSequence, string State, string Operation);

public sealed record CheckpointItem(string ManifestId, string TurnId, string Status, IReadOnlyList<string> Paths)
{
    public string FileCount => $"{Paths.Count} {(Paths.Count == 1 ? "file" : "files")}";
    public string PathsText => string.Join(Environment.NewLine, Paths);
    public bool CanReview => Status is "available" or "conflict" or "partial";
}

public sealed record GitFileItem(
    string Path,
    string Status,
    bool Staged,
    bool Unstaged,
    bool Conflicted,
    int Additions,
    int Deletions,
    string OldPath,
    bool Binary)
{
    public string Summary => Binary ? "BIN" : $"+{Additions}  -{Deletions}";
    public string StatusLetter => Conflicted ? "!" : Status switch
    {
        "added" or "untracked" => "A",
        "deleted" => "D",
        "renamed" => "R",
        _ => "M"
    };
    public bool HasOldPath => !string.IsNullOrWhiteSpace(OldPath);
    public bool IsSuccess => !Conflicted && Status is "added" or "untracked";
    public bool IsDanger => Conflicted || Status == "deleted";
    public bool IsWarning => !IsSuccess && !IsDanger;
}

public sealed record DiffLineItem(string Kind, string Text, string OldLine, string NewLine)
{
    public bool IsAddition => Kind == "addition";
    public bool IsDeletion => Kind == "deletion";
    public bool IsHunk => Kind == "hunk";
    public string DisplayText => IsHunk ? Text : $"{(IsAddition ? "+" : IsDeletion ? "-" : " ")}{Text}";
}

public sealed record ApprovalItem(string ApprovalId, string Operation, string Arguments)
{
    public static ApprovalItem? FromPayload(JsonObject payload)
    {
        var id = payload.String("approval_id", "approvalId");
        return string.IsNullOrWhiteSpace(id)
            ? null
            : new ApprovalItem(id, payload.String("operation"), payload["arguments"]?.ToJsonString() ?? "{}");
    }
}

internal static class JsonExtensions
{
    public static string String(this JsonObject value, params string[] names)
    {
        foreach (var name in names)
        {
            if (value[name] is JsonValue item && item.TryGetValue<string>(out var result))
            {
                return result ?? string.Empty;
            }
        }
        return string.Empty;
    }

    public static int Int(this JsonObject value, string name) =>
        value[name]?.GetValue<int>() ?? 0;

    public static long Long(this JsonObject value, params string[] names)
    {
        foreach (var name in names)
        {
            if (value[name] is JsonValue item && item.TryGetValue<long>(out var result))
            {
                return result;
            }
        }
        return 0;
    }

    public static bool Bool(this JsonObject value, string name) =>
        value[name]?.GetValue<bool>() ?? false;

    public static JsonObject Object(this JsonObject value, string name) =>
        value[name] as JsonObject ?? [];

    public static JsonArray Array(this JsonObject value, string name) =>
        value[name] as JsonArray ?? [];
}
