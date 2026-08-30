using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Text.Encodings.Web;
using System.Text.Json;
using System.Text.Json.Nodes;
using Avalonia.Media.Imaging;
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
    public bool IsDependencyRoot => IsRoot && IsDependency;
    public bool HasPathSubtitle => !string.IsNullOrWhiteSpace(Path) && Path != ".";
    public double ExpansionRotation => IsExpanded ? 90 : 0;
    public bool IsLoading { get => _isLoading; set => SetProperty(ref _isLoading, value); }
    public bool IsLoaded { get => _isLoaded; set => SetProperty(ref _isLoaded, value); }
    public bool IsExpanded
    {
        get => _isExpanded;
        set
        {
            if (!SetProperty(ref _isExpanded, value)) return;
            OnPropertyChanged(nameof(ExpansionRotation));
        }
    }
    public ObservableCollection<ExplorerNode> Children { get; } = [];

    private static ExplorerNode Placeholder() => new();
}

public sealed record SessionItem(string SessionId, string Title, string LastActivityAt, bool IsPinned)
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

public sealed record ProviderItem(string Id, string DisplayName, bool Configured)
{
    public string Display => Configured ? DisplayName : $"{DisplayName} (needs key)";
}

public sealed record ModelItem(string Id, string Provider, string ProviderLabel, string Availability, bool SupportsReasoningEffort)
{
    public bool Configured => Availability == "configured";
    public string Display => Configured ? Id : $"{Id} (needs key)";
}

public sealed record CredentialItem(string Provider, bool Configured);

public sealed class ComposerAttachment : IDisposable
{
    public ComposerAttachment(
        string imageId,
        string name,
        Bitmap preview,
        string storagePath,
        string sourceKind,
        string? originalPath)
    {
        ImageId = imageId;
        Name = name;
        Preview = preview;
        StoragePath = storagePath;
        SourceKind = sourceKind;
        OriginalPath = originalPath;
    }

    public string ImageId { get; }
    public string Name { get; }
    public Bitmap Preview { get; }
    public string StoragePath { get; }
    public string SourceKind { get; }
    public string? OriginalPath { get; }

    public void Dispose() => Preview.Dispose();

    public static ComposerAttachment FromPayload(JsonObject value)
    {
        var thumbnail = value.String("thumbnailBase64", "thumbnail_base64");
        var bytes = Convert.FromBase64String(thumbnail);
        using var stream = new MemoryStream(bytes, writable: false);
        return new ComposerAttachment(
            value.String("imageId", "image_id"),
            value.String("displayName", "display_name"),
            new Bitmap(stream),
            value.String("storagePath", "storage_path"),
            value.String("sourceKind", "source_kind"),
            value["originalPath"]?.GetValue<string>());
    }
}

public sealed class MessageItem : ObservableObject
{
    private string _messageId = string.Empty;
    private string _text = string.Empty;
    private bool _canBeFinalAssistant;
    private bool _streaming;
    private bool _isVisible = true;
    private bool _isProcess;
    private bool _isFinalAssistant;
    private bool _showProcessToggle;
    private bool _processContentVisible = true;
    private bool _processExpanded;
    private int _processItemCount;

    public string MessageId { get => _messageId; set => SetProperty(ref _messageId, value); }
    public required string Role { get; init; }
    public required string Text { get => _text; set => SetProperty(ref _text, value); }
    public required long ContentSequence { get; set; }
    public string TurnId { get; init; } = string.Empty;
    public string Kind { get; init; } = "message";
    public string ToolCallId { get; init; } = string.Empty;
    public string ToolName { get; init; } = string.Empty;
    public string ToolState { get; init; } = string.Empty;
    public string ToolDetail { get; init; } = string.Empty;
    public string ToolRequest { get; init; } = string.Empty;
    public string ToolResult { get; init; } = string.Empty;
    public string ToolError { get; init; } = string.Empty;
    public bool CanBeFinalAssistant { get => _canBeFinalAssistant; set => SetProperty(ref _canBeFinalAssistant, value); }
    public bool Streaming { get => _streaming; set => SetProperty(ref _streaming, value); }
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
    public string ToolSummaryText => ToolName switch
    {
        "bash" => "Run shell command",
        "webfetch" => "Fetch web content",
        "read" => "Read file",
        "glob" => "Find files",
        "grep" => "Search files",
        "question" => "Ask a question",
        "todowrite" => "Update turn todos",
        "write" => "Write file",
        "edit" => "Edit file",
        _ => string.IsNullOrWhiteSpace(ToolName) ? "Run operation" : ToolName
    };
    public bool IsToolFailed => ToolState is "failed" or "denied" or "timed_out" or "unknown_completion";
    public bool IsToolSucceeded => ToolState == "succeeded";
    public bool IsToolActive => !IsToolFailed && !IsToolSucceeded;
    public string ToolStateText => ToolState switch
    {
        "requested" or "validating" or "policy_check" or "authorized" => "Preparing",
        "executing" => "Running",
        "awaiting_approval" => "Waiting for approval",
        "awaiting_question" => "Waiting for an answer",
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
    public bool HasToolRequest => !string.IsNullOrWhiteSpace(ToolRequest);
    public bool HasToolResult => !string.IsNullOrWhiteSpace(ToolResult);
    public bool HasToolError => !string.IsNullOrWhiteSpace(ToolError);
    public string ToolErrorText => ToolError switch
    {
        "invalid_arguments" => "The operation arguments were invalid.",
        "authorization_denied" => "The operation was not authorized.",
        "scope_denied" => "The operation was outside the project scope.",
        "process_executable_not_found" => "The executable could not be found.",
        "process_start_failed" => "The process could not be started.",
        "webfetch_failed" => "The web request could not be completed.",
        _ => ToolError.Replace('_', ' ')
    };
}

public sealed record ActivityItem(string EventType, string Text, long ContentSequence, string State, string Operation);

public sealed record TodoItem(string Content, string Status, string Priority)
{
    public string StatusMarker => Status switch
    {
        "in_progress" => ">",
        "completed" => "x",
        "cancelled" => "-",
        _ => " "
    };

    public double Opacity => IsCompleted ? 0.58 : 1.0;

    public string StatusText => Status switch
    {
        "in_progress" => "In progress",
        "completed" => "Completed",
        "cancelled" => "Cancelled",
        _ => "Pending"
    };

    public string PriorityText => Priority switch
    {
        "high" => "High",
        "low" => "Low",
        _ => "Medium"
    };

    public bool IsCompleted => Status is "completed" or "cancelled";

    public static TodoItem? FromPayload(JsonObject payload)
    {
        var content = payload.String("content");
        return string.IsNullOrWhiteSpace(content)
            ? null
            : new TodoItem(content, payload.String("status"), payload.String("priority"));
    }
}

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

public sealed class QuestionPromptItem : ObservableObject
{
    private string _customAnswer = string.Empty;

    public QuestionPromptItem(string header, string question, bool multiple, bool allowCustom)
    {
        Header = header;
        Question = question;
        Multiple = multiple;
        AllowCustom = allowCustom;
    }

    public string Header { get; }
    public string Question { get; }
    public bool Multiple { get; }
    public bool AllowCustom { get; }
    public ObservableCollection<QuestionOptionItem> Options { get; } = [];
    public string CustomAnswer
    {
        get => _customAnswer;
        set => SetProperty(ref _customAnswer, value);
    }

    public void Select(QuestionOptionItem option)
    {
        if (!Multiple)
        {
            var wasSelected = option.IsSelected;
            foreach (var item in Options) item.IsSelected = !wasSelected && item == option;
        }
        else
        {
            option.IsSelected = !option.IsSelected;
        }
    }

    public IReadOnlyList<string> Answers => Options.Where(option => option.IsSelected).Select(option => option.Label)
        .Concat(string.IsNullOrWhiteSpace(CustomAnswer) ? [] : [CustomAnswer.Trim()]).ToArray();
}

public sealed class QuestionOptionItem : ObservableObject
{
    private bool _isSelected;

    public QuestionOptionItem(string label, string description)
    {
        Label = label;
        Description = description;
    }

    public string Label { get; }
    public string Description { get; }
    public bool IsSelected
    {
        get => _isSelected;
        set => SetProperty(ref _isSelected, value);
    }
}

public sealed class PendingQuestionItem
{
    public PendingQuestionItem(string requestId, string turnId, string toolCallId)
    {
        RequestId = requestId;
        TurnId = turnId;
        ToolCallId = toolCallId;
    }

    public string RequestId { get; }
    public string TurnId { get; }
    public string ToolCallId { get; }
    public ObservableCollection<QuestionPromptItem> Questions { get; } = [];

    public static PendingQuestionItem? FromPayload(JsonObject payload)
    {
        var requestId = payload.String("request_id", "requestId");
        if (string.IsNullOrWhiteSpace(requestId)) return null;
        var result = new PendingQuestionItem(requestId, payload.String("turn_id", "turnId"), payload.String("tool_call_id", "toolCallId"));
        foreach (var value in payload.Array("questions").OfType<JsonObject>())
        {
            var prompt = new QuestionPromptItem(
                value.String("header"), value.String("question"), value.Bool("multiple"),
                !value.TryGetPropertyValue("custom", out var custom) || custom?.GetValue<bool>() != false);
            foreach (var option in value.Array("options").OfType<JsonObject>())
                prompt.Options.Add(new QuestionOptionItem(option.String("label"), option.String("description")));
            result.Questions.Add(prompt);
        }
        return result.Questions.Count == 0 ? null : result;
    }
}

public sealed record ApprovalItem(string ApprovalId, string Operation, string Arguments)
{
    private JsonObject ArgumentObject
    {
        get
        {
            if (string.IsNullOrWhiteSpace(Arguments)) return [];
            try
            {
                return JsonNode.Parse(Arguments) as JsonObject ?? [];
            }
            catch (JsonException)
            {
                return [];
            }
        }
    }

    public string ActionText => Operation switch
    {
        "bash" => "Run a shell command",
        "webfetch" => "Fetch web content",
        "write" => "Write to a project file",
        "edit" => "Edit a project file",
        _ => "Perform a project action"
    };

    public string OperationText => Operation switch
    {
        "bash" => "Shell command",
        "webfetch" => "Web request",
        "write" => "File write",
        "edit" => "File edit",
        _ => string.IsNullOrWhiteSpace(Operation) ? "Project action" : Operation
    };

    public string DetailLabel => IsCommand ? "Command" : IsWebFetch ? "URL" : "Target";

    public string DetailText => IsCommand
        ? CommandText
        : IsWebFetch
            ? Value("url")
            : Value("path", "file", "target");

    public bool HasDetail => !string.IsNullOrWhiteSpace(DetailText);

    public string WorkingDirectoryText => Value("workdir", "cwd");

    public bool HasWorkingDirectory => IsCommand && !string.IsNullOrWhiteSpace(WorkingDirectoryText);

    public bool IsCommand => Operation == "bash";

    public string ProgramText => Value("program", "command");

    private bool IsWebFetch => Operation == "webfetch";

    private string CommandText
    {
        get
        {
            var script = Value("script");
            if (!string.IsNullOrWhiteSpace(script)) return script;

            var program = ProgramText;
            if (string.IsNullOrWhiteSpace(program)) return string.Empty;

            var args = ArgumentObject["args"] as JsonArray;
            return args is null || args.Count == 0
                ? program
                : string.Join(" ", new[] { program }.Concat(args.Select(FormatArgument)));
        }
    }

    private string Value(params string[] names)
    {
        foreach (var name in names)
        {
            if (ArgumentObject[name] is JsonValue value && value.TryGetValue<string>(out var text))
                return text ?? string.Empty;
        }
        return string.Empty;
    }

    private static string FormatArgument(JsonNode? value)
    {
        if (value is not JsonValue jsonValue || !jsonValue.TryGetValue<string>(out var text))
            return value?.ToJsonString() ?? string.Empty;

        if (string.IsNullOrEmpty(text) || text.Any(char.IsWhiteSpace) || text.Contains('"'))
            return $"\"{text.Replace("\\", "\\\\").Replace("\"", "\\\"")}\"";
        return text;
    }

    public static ApprovalItem? FromPayload(JsonObject payload)
    {
        var id = payload.String("approval_id", "approvalId");
        return string.IsNullOrWhiteSpace(id)
            ? null
            : new ApprovalItem(
                id,
                payload.String("operation"),
                payload["arguments"]?.ToJsonString(DisplayJson.Options) ?? "{}");
    }
}

internal static class DisplayJson
{
    // This text is rendered in a read-only desktop code view, not emitted to HTML or a script.
    public static JsonSerializerOptions Options { get; } = new()
    {
        WriteIndented = true,
        Encoder = JavaScriptEncoder.UnsafeRelaxedJsonEscaping
    };
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
