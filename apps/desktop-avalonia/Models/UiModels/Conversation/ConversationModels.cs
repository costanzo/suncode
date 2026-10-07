using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Text.Encodings.Web;
using System.Text.Json;
using Avalonia.Media.Imaging;
using SunCode.Desktop.Infrastructure;
using SunCode.Sdk.Models;

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

    public static ComposerAttachment FromSdk(SessionImage value)
    {
        var bytes = Convert.FromBase64String(value.ThumbnailBase64);
        using var stream = new MemoryStream(bytes, writable: false);
        return new ComposerAttachment(
            value.ImageId,
            value.DisplayName,
            new Bitmap(stream),
            value.StoragePath,
            value.SourceKind,
            value.OriginalPath);
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
    private bool _processExpanded;
    private int _processItemCount;
    private string _durationText = string.Empty;
    private bool _isWorkingDuration;
    private string _completionTimeText = string.Empty;

    public string MessageId { get => _messageId; set => SetProperty(ref _messageId, value); }
    public required string Role { get; init; }
    public required string Text
    {
        get => _text;
        set
        {
            if (!SetProperty(ref _text, value)) return;
            OnPropertyChanged(nameof(IsLongUserMessage));
            OnPropertyChanged(nameof(UserPreviewText));
        }
    }
    public required long ContentSequence { get; set; }
    public string TurnId { get; init; } = string.Empty;
    public string Kind { get; init; } = "message";
    public int TurnSequence { get; init; }
    public string TurnPreview { get; init; } = string.Empty;
    public string ToolCallId { get; init; } = string.Empty;
    public string ToolName { get; init; } = string.Empty;
    public string ToolState { get; init; } = string.Empty;
    public string ToolRequest { get; init; } = string.Empty;
    public string ToolError { get; init; } = string.Empty;
    public IReadOnlyList<ComposerAttachment> Attachments { get; init; } = [];
    public bool HasAttachments => Attachments.Count > 0;
    public string DurationText
    {
        get => _durationText;
        set
        {
            if (SetProperty(ref _durationText, value)) OnPropertyChanged(nameof(DurationLabel));
        }
    }
    public bool IsWorkingDuration
    {
        get => _isWorkingDuration;
        init => _isWorkingDuration = value;
    }
    public bool ShowDuration => (IsWorkingDuration || IsFinalAssistant) && !string.IsNullOrWhiteSpace(DurationText);
    public string DurationLabel => IsWorkingDuration ? $"Working for {DurationText}" : $"Worked for {DurationText}";
    public string CompletionTimeText
    {
        get => _completionTimeText;
        set
        {
            if (SetProperty(ref _completionTimeText, value)) OnPropertyChanged(nameof(HasCompletionTime));
        }
    }
    public bool HasCompletionTime => !string.IsNullOrWhiteSpace(CompletionTimeText);
    public bool CanBeFinalAssistant { get => _canBeFinalAssistant; set => SetProperty(ref _canBeFinalAssistant, value); }
    public bool Streaming { get => _streaming; set => SetProperty(ref _streaming, value); }
    public bool IsUser => Role == "user";
    public bool IsAssistant => Role == "assistant";
    public bool IsTool => Kind == "tool";
    public bool IsTurnMarker => Kind == "turn_marker";
    public bool IsConversationAssistant => IsAssistant && !IsTurnMarker && !IsCompaction;
    public bool IsIntermediateAssistant => IsConversationAssistant && !IsFinalAssistant;
    public bool IsCompaction => Kind == "context.compacted";
    // Keep the timeline compact for unusually large submitted prompts while
    // retaining the canonical text for the read-only detail dialog.
    public bool IsLongUserMessage => IsUser && (Text.Length > 340 || Text.Count(c => c == '\n') >= 5);
    public string UserPreviewText
    {
        get
        {
            if (!IsLongUserMessage) return Text;

            var lines = Text.Replace("\r\n", "\n").Split('\n');
            var preview = string.Join("\n", lines.Take(5));
            if (preview.Length > 340) preview = preview[..340].TrimEnd();
            return preview.TrimEnd() + "...";
        }
    }
    public bool IsVisible { get => _isVisible; set => SetProperty(ref _isVisible, value); }
    public bool IsProcess { get => _isProcess; set => SetProperty(ref _isProcess, value); }
    public bool IsFinalAssistant
    {
        get => _isFinalAssistant;
        set
        {
            if (SetProperty(ref _isFinalAssistant, value))
            {
                OnPropertyChanged(nameof(ShowCopy));
                OnPropertyChanged(nameof(ShowDuration));
                OnPropertyChanged(nameof(DurationLabel));
                OnPropertyChanged(nameof(IsIntermediateAssistant));
            }
        }
    }
    public bool ShowCopy => IsFinalAssistant;
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
    public string ToolSummaryText => ToolStates.Summary(ToolName);
    public string ToolStateText => ToolStates.DisplayText(ToolState);
    public bool HasToolRequest => !string.IsNullOrWhiteSpace(ToolRequest);
    public bool HasToolError => !string.IsNullOrWhiteSpace(ToolError);
    public string ToolErrorText => ToolStates.ErrorText(ToolError);
}

public sealed class ToolActivityTurnItem : ObservableObject
{
    private bool _isExpanded;

    public ToolActivityTurnItem(string turnId, int sequence, string state, string preview, string createdAt, string? startedAt = null, string? completedAt = null)
    {
        TurnId = turnId;
        Sequence = sequence;
        State = state;
        Preview = preview;
        CreatedAt = createdAt;
        StartedAt = string.IsNullOrWhiteSpace(startedAt) ? createdAt : startedAt;
        CompletedAt = completedAt ?? string.Empty;
    }

    public string TurnId { get; }
    public int Sequence { get; }
    public string State { get; private set; }
    public string Preview { get; private set; }
    public string CreatedAt { get; }
    public string StartedAt { get; private set; }
    public string CompletedAt { get; private set; }
    public ObservableCollection<ToolActivityItem> Tools { get; } = [];
    public bool IsExpanded { get => _isExpanded; set => SetProperty(ref _isExpanded, value); }
    public string Title => $"Turn {Sequence}";
    public string IdentifierText => TurnId.Length <= 8 ? TurnId : TurnId[..8];
    public string ToolCountText => $"{Tools.Count}";
    public string StateText => State.Replace('_', ' ');
    public bool IsActive => State is "admitted" or "queued" or "preparing" or "calling_model" or "resolving_calls" or "compacting";

    public void Update(string state, string? preview = null)
    {
        State = state;
        if (!string.IsNullOrWhiteSpace(preview)) Preview = preview;
        OnPropertyChanged(nameof(State));
        OnPropertyChanged(nameof(IsActive));
    }

    public void SetTiming(string? startedAt, string? completedAt)
    {
        if (!string.IsNullOrWhiteSpace(startedAt)) StartedAt = startedAt;
        if (!string.IsNullOrWhiteSpace(completedAt)) CompletedAt = completedAt;
    }
}

public sealed class ToolActivityItem : ObservableObject
{
    private string _name;
    private string _state;
    private string _request;
    private string _result;
    private string _output;
    private string _error;

    public ToolActivityItem(
        string turnId,
        string toolCallId,
        string name,
        string state,
        string request,
        string result,
        string output,
        string error,
        string createdAt)
    {
        TurnId = turnId;
        ToolCallId = toolCallId;
        _name = name;
        _state = state;
        _request = request;
        _result = result;
        _output = output;
        _error = error;
        CreatedAt = createdAt;
    }

    public string TurnId { get; }
    public string ToolCallId { get; }
    public string CreatedAt { get; }
    public string Name { get => _name; private set => SetProperty(ref _name, value); }
    public string State { get => _state; private set => SetProperty(ref _state, value); }
    public string Request { get => _request; private set => SetProperty(ref _request, value); }
    public string Result { get => _result; private set => SetProperty(ref _result, value); }
    public string Output { get => _output; private set => SetProperty(ref _output, value); }
    public string Error { get => _error; private set => SetProperty(ref _error, value); }
    public string DisplayName => ToolStates.Summary(Name);
    public string StateText => ToolStates.DisplayText(State);
    public bool IsSucceeded => ToolStates.IsSucceeded(State);
    public bool IsFailed => ToolStates.IsFailed(State);
    public bool IsActive => ToolStates.IsActive(State);
    public bool HasRequest => !string.IsNullOrWhiteSpace(Request);
    public bool HasResult => !string.IsNullOrWhiteSpace(Result);
    public bool HasOutput => !string.IsNullOrWhiteSpace(Output);
    public bool ShowOutput => HasOutput || IsActive;
    public bool HasError => !string.IsNullOrWhiteSpace(Error);
    public string ErrorText => ToolStates.ErrorText(Error);

    public void Update(string? name = null, string? state = null, string? request = null, string? result = null, string? output = null, string? error = null)
    {
        if (!string.IsNullOrWhiteSpace(name)) Name = name;
        if (!string.IsNullOrWhiteSpace(state)) State = state;
        if (request is not null) Request = request;
        if (result is not null) Result = result;
        if (output is not null) Output = output;
        if (error is not null) Error = error;
        OnPropertyChanged(nameof(DisplayName));
        OnPropertyChanged(nameof(StateText));
        OnPropertyChanged(nameof(IsSucceeded));
        OnPropertyChanged(nameof(IsFailed));
        OnPropertyChanged(nameof(IsActive));
        OnPropertyChanged(nameof(HasRequest));
        OnPropertyChanged(nameof(HasResult));
        OnPropertyChanged(nameof(HasOutput));
        OnPropertyChanged(nameof(ShowOutput));
        OnPropertyChanged(nameof(HasError));
        OnPropertyChanged(nameof(ErrorText));
    }
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


    public bool IsCompleted => Status is "completed" or "cancelled";

    public static TodoItem? FromSdk(SessionTurnTodo todo)
    {
        return string.IsNullOrWhiteSpace(todo.Content)
            ? null
            : new TodoItem(todo.Content, todo.Status, todo.Priority);
    }

    public static TodoItem? FromSdk(AgentTodoEventItem todo)
    {
        return string.IsNullOrWhiteSpace(todo.Content)
            ? null
            : new TodoItem(todo.Content, todo.Status, todo.Priority);
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
    public bool IsContext => Kind == "context";
    public string DisplayText => IsHunk ? Text : $"{(IsAddition ? "+" : IsDeletion ? "-" : " ")}{Text}";
}
