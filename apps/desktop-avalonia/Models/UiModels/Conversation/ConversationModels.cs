using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Text.Encodings.Web;
using System.Text.Json;
using System.Text.Json.Nodes;
using Avalonia.Media.Imaging;
using SunCode.Desktop.Infrastructure;

namespace SunCode.Desktop.Models;

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

public sealed class MessageItem : ObservableObject, IDisposable
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
    public IReadOnlyList<ComposerAttachment> Attachments { get; init; } = [];
    public bool HasAttachments => Attachments.Count > 0;
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

    public void Dispose()
    {
        foreach (var attachment in Attachments) attachment.Dispose();
    }
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
