using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Text.Encodings.Web;
using System.Text.Json;
using System.Text.Json.Nodes;
using Avalonia.Media.Imaging;
using SunCode.Desktop.Infrastructure;

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
