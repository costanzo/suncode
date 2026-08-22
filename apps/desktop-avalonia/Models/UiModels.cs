using System.Collections.ObjectModel;
using System.Text.Json.Nodes;
using SunCode.Desktop.Infrastructure;

namespace SunCode.Desktop.Models;

public sealed record ProjectItem(string ProjectId, string DisplayName, string CanonicalRoot);

public sealed record ProjectDependencyItem(string DependencyId, string DisplayName);

public sealed class ExplorerNode : ObservableObject
{
    private bool _isLoading;
    private bool _isLoaded;
    private bool _isExpanded;

    public ExplorerNode(
        string name,
        string path,
        string kind,
        string? dependencyId = null,
        bool isRoot = false,
        bool isDependency = false,
        bool isGroup = false)
    {
        Name = name;
        Path = path;
        Kind = kind;
        DependencyId = dependencyId;
        IsRoot = isRoot;
        IsDependency = isDependency;
        IsGroup = isGroup;
        _isExpanded = isRoot || isGroup;
        if (IsDirectory && !isGroup) Children.Add(Placeholder());
    }

    private ExplorerNode()
    {
        Name = string.Empty;
        Path = string.Empty;
        Kind = "placeholder";
        IsPlaceholder = true;
    }

    public string Name { get; }
    public string Path { get; }
    public string Kind { get; }
    public string? DependencyId { get; }
    public bool IsRoot { get; }
    public bool IsDependency { get; }
    public bool IsGroup { get; }
    public bool IsPlaceholder { get; }
    public bool IsDirectory => Kind == "directory" || IsGroup;
    public bool IsFile => Kind == "file";
    public bool CanRemove => IsRoot && IsDependency;
    public bool IsLoading { get => _isLoading; set => SetProperty(ref _isLoading, value); }
    public bool IsLoaded { get => _isLoaded; set => SetProperty(ref _isLoaded, value); }
    public bool IsExpanded { get => _isExpanded; set => SetProperty(ref _isExpanded, value); }
    public ObservableCollection<ExplorerNode> Children { get; } = [];

    private static ExplorerNode Placeholder() => new();
}

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

public sealed class MessageItem : ObservableObject
{
    private bool _isVisible = true;
    private bool _isProcess;
    private bool _isFinalAssistant;
    private bool _showProcessToggle;
    private bool _processContentVisible = true;
    private bool _processExpanded;
    private int _processItemCount;

    public string MessageId { get; init; } = string.Empty;
    public required string Role { get; init; }
    public required string Text { get; set; }
    public required long ContentSequence { get; set; }
    public string TurnId { get; init; } = string.Empty;
    public string Kind { get; init; } = "message";
    public string ToolCallId { get; init; } = string.Empty;
    public string ToolName { get; init; } = string.Empty;
    public string ToolState { get; init; } = string.Empty;
    public string ToolDetail { get; init; } = string.Empty;
    public bool CanBeFinalAssistant { get; init; }
    public bool Streaming { get; set; }
    public bool IsUser => Role == "user";
    public bool IsAssistant => Role == "assistant";
    public bool IsTool => Kind == "tool";
    public string Author => IsUser ? "You" : "SunCode";
    public bool IsVisible { get => _isVisible; set => SetProperty(ref _isVisible, value); }
    public bool IsProcess { get => _isProcess; set => SetProperty(ref _isProcess, value); }
    public bool IsFinalAssistant
    {
        get => _isFinalAssistant;
        set
        {
            if (SetProperty(ref _isFinalAssistant, value)) OnPropertyChanged(nameof(ShowCopy));
        }
    }
    public bool ShowCopy => IsFinalAssistant;
    public bool ShowProcessToggle { get => _showProcessToggle; set => SetProperty(ref _showProcessToggle, value); }
    public bool ProcessContentVisible { get => _processContentVisible; set => SetProperty(ref _processContentVisible, value); }
    public bool ProcessExpanded
    {
        get => _processExpanded;
        set
        {
            if (!SetProperty(ref _processExpanded, value)) return;
            OnPropertyChanged(nameof(ProcessCollapsed));
            OnPropertyChanged(nameof(ProcessToggleText));
        }
    }
    public bool ProcessCollapsed => !ProcessExpanded;
    public int ProcessItemCount
    {
        get => _processItemCount;
        set
        {
            if (SetProperty(ref _processItemCount, value)) OnPropertyChanged(nameof(ProcessToggleText));
        }
    }
    public string ProcessToggleText => ProcessExpanded
        ? "Hide work"
        : $"Show work ({ProcessItemCount})";
    public string ToolStateText => ToolState switch
    {
        "requested" or "validating" or "policy_check" or "authorized" => "Preparing",
        "executing" => "Running",
        "awaiting_approval" => "Waiting for approval",
        "succeeded" => "Completed",
        "denied" => "Denied",
        "failed" => "Failed",
        "timed_out" => "Timed out",
        "unknown_completion" or "reconciling" => "Checking result",
        _ => ToolState
    };
    public string ToolDetailText
    {
        get
        {
            var compact = string.Join(" ", ToolDetail.Split((char[]?)null, StringSplitOptions.RemoveEmptyEntries));
            return compact.Length <= 240 ? compact : $"{compact[..240]}…";
        }
    }
    public bool HasToolDetail => !string.IsNullOrWhiteSpace(ToolDetail);
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

public sealed record ProviderTraceItem(
    string ExchangeId,
    string TurnId,
    string Provider,
    string ModelId,
    string WireModel,
    string ProviderRequestId,
    string ProviderResponseId,
    string State,
    int Iteration,
    string StartedAt,
    string CompletedAt,
    long? InputTokens,
    long? OutputTokens,
    long? CacheReadTokens,
    long? CacheWriteTokens,
    long? TotalTokens,
    string FinishReason,
    string InputText,
    string OutputText,
    string ToolCallsText,
    string ErrorText,
    IReadOnlyList<ProviderTraceMessageItem> Messages,
    IReadOnlyList<ProviderTraceToolItem> Tools)
{
    public ObservableCollection<ProviderTraceContentItem> Contents { get; } = [ProviderTraceContentItem.Placeholder()];
    public bool IsExpanded { get; set; }
    public bool ContentsLoaded { get; set; }
    public bool ContentsLoading { get; set; }
    public string Title => $"{Provider}  ·  {ModelId}";
    public string CallText => $"Call {Iteration}";
    public string TurnText => $"turn {Short(TurnId)}";
    public string IdentifierText => Short(ExchangeId);
    public string TokenSummary => TotalTokens is { } total
        ? $"{Compact(total)} tokens"
        : "usage not reported";
    public string UsageSummary => string.Join("  ",
        new[]
        {
            InputTokens is { } input ? $"in {Compact(input)}" : "in -",
            OutputTokens is { } output ? $"out {Compact(output)}" : "out -",
            CacheReadTokens is { } cacheRead ? $"cache read {Compact(cacheRead)}" : "cache read -",
            CacheWriteTokens is { } cacheWrite ? $"cache write {Compact(cacheWrite)}" : "cache write -",
        });
    public string InputTokenText => Metric(InputTokens);
    public string OutputTokenText => Metric(OutputTokens);
    public string CacheReadTokenText => Metric(CacheReadTokens);
    public string CacheWriteTokenText => Metric(CacheWriteTokens);
    public string TotalTokenText => Metric(TotalTokens);
    public string CacheHitRateText => InputTokens is > 0 && CacheReadTokens is { } cached
        ? $"{cached * 100d / InputTokens.Value:0.#}%"
        : "—";
    public string DurationText
    {
        get
        {
            if (!DateTimeOffset.TryParse(StartedAt, out var started)) return "—";
            var ended = DateTimeOffset.TryParse(CompletedAt, out var completed) ? completed : DateTimeOffset.Now;
            var elapsed = ended - started;
            return elapsed.TotalSeconds < 1 ? $"{elapsed.TotalMilliseconds:0} ms" : $"{elapsed.TotalSeconds:0.##} s";
        }
    }
    public string TimingText => DateTimeOffset.TryParse(StartedAt, out var timestamp)
        ? timestamp.ToLocalTime().ToString("HH:mm:ss.fff")
        : StartedAt;
    public string StatusText => State switch
    {
        "started" => "Running",
        "completed" => string.IsNullOrWhiteSpace(FinishReason) ? "Completed" : FinishReason,
        "failed" => "Failed",
        _ => State
    };
    public bool IsRunning => State == "started";
    public bool IsCompleted => State == "completed";
    public bool IsFailed => State == "failed";
    public bool HasOutput => !string.IsNullOrWhiteSpace(OutputText);
    public bool HasToolCalls => !string.IsNullOrWhiteSpace(ToolCallsText) && ToolCallsText != "[]";
    public bool HasError => !string.IsNullOrWhiteSpace(ErrorText);
    public bool HasMessages => Messages.Count > 0;
    public bool HasTools => Tools.Count > 0;
    public bool HasProviderRequestId => !string.IsNullOrWhiteSpace(ProviderRequestId);
    public bool HasProviderResponseId => !string.IsNullOrWhiteSpace(ProviderResponseId);

    private static string Short(string value) => value.Length <= 8 ? value : value[..8];
    private static string Metric(long? value) => value is { } number ? Compact(number) : "—";
    private static string Compact(long value) => value switch
    {
        >= 1_000_000 => $"{value / 1_000_000d:0.#}m",
        >= 1_000 => $"{value / 1_000d:0.#}k",
        _ => value.ToString()
    };
}

public sealed record ProviderTraceTurnItem(
    string TurnId,
    string State,
    string ModelId,
    string CreatedAt,
    string StartedAt,
    string CompletedAt,
    long InputTokens,
    long OutputTokens,
    long TotalTokens,
    int Sequence,
    IReadOnlyList<ProviderTraceItem> Calls)
{
    public bool IsExpanded { get; set; } = true;
    public string Title => $"Turn {Sequence}";
    public string IdentifierText => TurnId.Length <= 8 ? TurnId : TurnId[..8];
    public string CallCountText => $"{Calls.Count} {(Calls.Count == 1 ? "call" : "calls")}";
    public string TokenText => TotalTokens > 0 ? $"{Compact(TotalTokens)} tokens" : "no usage";
    public string MetricsText => $"{CallCountText}  ·  {TokenText}";
    public string DurationText
    {
        get
        {
            var startText = string.IsNullOrWhiteSpace(StartedAt) ? CreatedAt : StartedAt;
            if (!DateTimeOffset.TryParse(startText, out var started)) return "—";

            DateTimeOffset ended;
            if (DateTimeOffset.TryParse(CompletedAt, out var completed))
            {
                ended = completed;
            }
            else if (IsRunning)
            {
                ended = DateTimeOffset.Now;
            }
            else
            {
                return "—";
            }

            var elapsed = ended - started;
            if (elapsed < TimeSpan.Zero) elapsed = TimeSpan.Zero;
            return elapsed.TotalSeconds < 1
                ? $"{elapsed.TotalMilliseconds:0} ms"
                : $"{elapsed.TotalSeconds:0.##} s";
        }
    }
    public string StateText => State.Replace('_', ' ');
    public string TimeText => DateTimeOffset.TryParse(CreatedAt, out var timestamp)
        ? timestamp.ToLocalTime().ToString("HH:mm:ss")
        : CreatedAt;
    public bool IsRunning => State is "admitted" or "queued" or "preparing" or "calling_model" or "resolving_calls" or "compacting";
    public bool IsCompleted => State == "completed";
    public bool IsFailed => State is "failed" or "cancelled" or "interrupted";

    private static string Compact(long value) => value switch
    {
        >= 1_000_000 => $"{value / 1_000_000d:0.#}m",
        >= 1_000 => $"{value / 1_000d:0.#}k",
        _ => value.ToString()
    };
}

public sealed record ProviderTraceContentItem(
    string ExchangeId,
    string Kind,
    string Title,
    string Summary,
    string Content,
    string Request,
    string Result,
    string ErrorCode,
    string CreatedAt,
    bool IsPlaceholder = false)
{
    public bool IsExpanded { get; set; }
    public string TimeText => DateTimeOffset.TryParse(CreatedAt, out var timestamp)
        ? timestamp.ToLocalTime().ToString("HH:mm:ss.fff")
        : CreatedAt;
    public string KindText => Kind switch
    {
        "user" => "USER MESSAGE",
        "assistant" => "ASSISTANT MESSAGE",
        "thinking" => "THINKING MESSAGE",
        "tool" => "TOOL USE",
        _ => Kind.ToUpperInvariant()
    };
    public bool IsUser => Kind == "user";
    public bool IsAssistant => Kind == "assistant";
    public bool IsThinking => Kind == "thinking";
    public bool IsTool => Kind == "tool";
    public bool HasContent => !string.IsNullOrWhiteSpace(Content);
    public bool HasRequest => !string.IsNullOrWhiteSpace(Request);
    public bool HasResult => !string.IsNullOrWhiteSpace(Result);
    public bool HasError => !string.IsNullOrWhiteSpace(ErrorCode);

    public static ProviderTraceContentItem Placeholder(string title = "Load call contents") =>
        new(string.Empty, "placeholder", title, string.Empty, string.Empty, string.Empty, string.Empty, string.Empty, string.Empty, true);
}

public sealed record ProviderTraceMessageItem(string MessageId, string Role, string Content, string CreatedAt)
{
    public string RoleText => Role.ToUpperInvariant();
    public string TimeText => DateTimeOffset.TryParse(CreatedAt, out var timestamp)
        ? timestamp.ToLocalTime().ToString("HH:mm:ss.fff")
        : CreatedAt;
    public bool IsUser => Role == "user";
    public bool IsAssistant => Role == "assistant";
    public bool IsThinking => Role == "thinking";
    public bool IsTool => Role == "tool";
}

public sealed record ProviderTraceToolItem(
    string ToolCallId,
    string Name,
    string State,
    string Request,
    string Result,
    string ErrorCode,
    string CreatedAt)
{
    public string StateText => State.Replace('_', ' ');
    public string TimeText => DateTimeOffset.TryParse(CreatedAt, out var timestamp)
        ? timestamp.ToLocalTime().ToString("HH:mm:ss.fff")
        : CreatedAt;
    public bool IsSucceeded => State == "succeeded";
    public bool IsFailed => State is "failed" or "denied" or "timed_out" or "unknown_completion";
    public bool IsActive => !IsSucceeded && !IsFailed;
    public bool HasRequest => !string.IsNullOrWhiteSpace(Request);
    public bool HasResult => !string.IsNullOrWhiteSpace(Result);
    public bool HasError => !string.IsNullOrWhiteSpace(ErrorCode);
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

    public static JsonArray Array(this JsonObject value, params string[] names)
    {
        foreach (var name in names)
        {
            if (value[name] is JsonArray result) return result;
        }
        return [];
    }
}
