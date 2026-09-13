using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Text.Encodings.Web;
using System.Text.Json;
using Avalonia.Media.Imaging;
using SunCode.Desktop.Infrastructure;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Models;

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

    public static PendingQuestionItem? FromSdk(PendingQuestion payload)
    {
        var requestId = payload.RequestId;
        if (string.IsNullOrWhiteSpace(requestId)) return null;
        var result = new PendingQuestionItem(requestId, payload.TurnId, payload.ToolCallId);
        foreach (var value in payload.Questions)
        {
            var prompt = new QuestionPromptItem(
                value.Header, value.Question, value.Multiple, value.Custom);
            foreach (var option in value.Options)
                prompt.Options.Add(new QuestionOptionItem(option.Label, option.Description));
            result.Questions.Add(prompt);
        }
        return result.Questions.Count == 0 ? null : result;
    }

    public static PendingQuestionItem? FromSdk(AgentEventPayload payload)
    {
        if (string.IsNullOrWhiteSpace(payload.RequestId) || payload.Questions is null || payload.Questions.Count == 0)
            return null;
        var result = new PendingQuestionItem(
            payload.RequestId,
            payload.TurnId ?? string.Empty,
            payload.ToolCallId ?? string.Empty);
        foreach (var value in payload.Questions)
        {
            var prompt = new QuestionPromptItem(value.Header, value.Question, value.Multiple, value.Custom);
            foreach (var option in value.Options)
                prompt.Options.Add(new QuestionOptionItem(option.Label, option.Description));
            result.Questions.Add(prompt);
        }
        return result;
    }
}

public sealed record ApprovalItem(string ApprovalId, string Operation, string Arguments)
{
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

            return !TryGetArguments(out var argumentObject)
                || !argumentObject.TryGetProperty("args", out var args)
                || args.ValueKind != JsonValueKind.Array
                || args.GetArrayLength() == 0
                ? program
                : string.Join(" ", new[] { program }.Concat(args.EnumerateArray().Select(FormatArgument)));
        }
    }

    private string Value(params string[] names)
    {
        foreach (var name in names)
        {
            if (TryGetArguments(out var argumentObject)
                && argumentObject.TryGetProperty(name, out var value)
                && value.ValueKind == JsonValueKind.String)
                return value.GetString() ?? string.Empty;
        }
        return string.Empty;
    }

    private bool TryGetArguments(out JsonElement value)
    {
        try
        {
            using var document = JsonDocument.Parse(Arguments);
            value = document.RootElement.Clone();
            return value.ValueKind == JsonValueKind.Object;
        }
        catch (JsonException)
        {
            value = default;
            return false;
        }
    }

    private static string FormatArgument(JsonElement value)
    {
        if (value.ValueKind != JsonValueKind.String)
            return value.GetRawText();
        var text = value.GetString() ?? string.Empty;

        if (string.IsNullOrEmpty(text) || text.Any(char.IsWhiteSpace) || text.Contains('"'))
            return $"\"{text.Replace("\\", "\\\\").Replace("\"", "\\\"")}\"";
        return text;
    }

    public static ApprovalItem? FromSdk(ApprovalRecord payload)
    {
        var id = payload.ApprovalId;
        return string.IsNullOrWhiteSpace(id)
            ? null
            : new ApprovalItem(
                id,
                payload.Operation,
                payload.Arguments.ValueKind == JsonValueKind.Undefined
                    ? "{}"
                    : JsonSerializer.Serialize(payload.Arguments, DisplayJson.Options));
    }

    public static ApprovalItem? FromSdk(AgentEventPayload payload)
    {
        return string.IsNullOrWhiteSpace(payload.ApprovalId)
            ? null
            : new ApprovalItem(
                payload.ApprovalId,
                payload.Operation ?? string.Empty,
                payload.Arguments is { } arguments && arguments.ValueKind != JsonValueKind.Undefined
                    ? JsonSerializer.Serialize(arguments, DisplayJson.Options)
                    : "{}");
    }
}
