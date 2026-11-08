using System.Collections.ObjectModel;
using System.ComponentModel;
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
    private int _number;
    private QuestionOptionItem[] _observedOptions = [];
    private readonly string _groupName = Guid.NewGuid().ToString("N");

    public QuestionPromptItem(string header, string question, bool multiple, bool allowCustom)
    {
        Header = header;
        Question = question;
        Multiple = multiple;
        AllowCustom = allowCustom;
        Options.CollectionChanged += (_, _) => ObserveOptions();
    }

    public string Header { get; }
    public string NumberedHeader => $"{_number} · {Header}";
    public string Question { get; }
    public bool Multiple { get; }
    public bool AllowCustom { get; }
    public ObservableCollection<QuestionOptionItem> Options { get; } = [];
    public string CustomAnswer
    {
        get => _customAnswer;
        set
        {
            if (!SetProperty(ref _customAnswer, value)) return;
            if (!Multiple && !string.IsNullOrWhiteSpace(value))
                foreach (var option in Options) option.IsSelected = false;
            NotifyAnswersChanged();
        }
    }

    internal void SetNumber(int number)
    {
        _number = number;
        OnPropertyChanged(nameof(NumberedHeader));
    }

    private void ObserveOptions()
    {
        foreach (var option in _observedOptions) option.PropertyChanged -= OptionChanged;
        _observedOptions = Options.ToArray();
        foreach (var option in _observedOptions)
        {
            option.Multiple = Multiple;
            option.GroupName = _groupName;
            option.PropertyChanged += OptionChanged;
        }
        NotifyAnswersChanged();
    }

    private void OptionChanged(object? sender, PropertyChangedEventArgs e)
    {
        if (e.PropertyName != nameof(QuestionOptionItem.IsSelected)) return;
        if (!Multiple && sender is QuestionOptionItem { IsSelected: true } selected)
        {
            foreach (var option in Options)
                if (option != selected) option.IsSelected = false;
            CustomAnswer = string.Empty;
        }
        NotifyAnswersChanged();
    }

    private void NotifyAnswersChanged()
    {
        OnPropertyChanged(nameof(Answers));
        OnPropertyChanged(nameof(HasAnswer));
    }

    public void Select(QuestionOptionItem option)
    {
        if (Options.Contains(option)) option.IsSelected = !option.IsSelected;
    }

    public bool HasAnswer => Answers.Count > 0;
    public IReadOnlyList<string> Answers => Options.Where(option => option.IsSelected).Select(option => option.Label)
        .Concat(!AllowCustom || string.IsNullOrWhiteSpace(CustomAnswer) ? [] : [CustomAnswer.Trim()]).ToArray();
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
    public bool Multiple { get; internal set; }
    public string GroupName { get; internal set; } = string.Empty;
    public bool IsSelected
    {
        get => _isSelected;
        set
        {
            if (SetProperty(ref _isSelected, value)) OnPropertyChanged(nameof(IsSingleSelected));
        }
    }

    // Invisible radio controls must stay unchecked for multi-select prompts.
    public bool IsSingleSelected
    {
        get => !Multiple && IsSelected;
        set { if (!Multiple) IsSelected = value; }
    }
}

public sealed class PendingQuestionItem : ObservableObject
{
    private QuestionPromptItem[] _observedQuestions = [];
    private bool _isResolving;
    private bool _isSkipping;
    private string? _errorKey;

    public PendingQuestionItem(string requestId, string turnId, string toolCallId)
    {
        RequestId = requestId;
        TurnId = turnId;
        ToolCallId = toolCallId;
        Questions.CollectionChanged += (_, _) => ObserveQuestions();
    }

    public string RequestId { get; }
    public string TurnId { get; }
    public string ToolCallId { get; }
    public ObservableCollection<QuestionPromptItem> Questions { get; } = [];
    public bool IsResolving => _isResolving;
    public bool IsEditable => !IsResolving;
    public bool IsSubmitting => IsResolving && !_isSkipping;
    public bool IsSkipping => IsResolving && _isSkipping;
    public int AnsweredCount => Questions.Count(question => question.HasAnswer);
    public bool CanSubmit => IsEditable && AnsweredCount > 0;
    public bool HasError => _errorKey is not null;
    public string ErrorText => _errorKey is null ? string.Empty : LocalizationService.GetString(_errorKey,
        "Couldn't resolve the questions. Your answers are saved. Try again.");
    public string AnsweredText => string.Format(LocalizationService.GetString("LocQuestionsAnsweredFormat", "{0} of {1} answered"), AnsweredCount, Questions.Count);
    public string ContextText => string.Format(LocalizationService.GetString(
        Questions.Count == 1 ? "LocQuestionContextFormat" : "LocQuestionsContextFormat",
        Questions.Count == 1 ? "{0} question · Current session" : "{0} questions · Current session"), Questions.Count);

    private void ObserveQuestions()
    {
        foreach (var question in _observedQuestions) question.PropertyChanged -= QuestionChanged;
        _observedQuestions = Questions.ToArray();
        for (var index = 0; index < _observedQuestions.Length; index++)
        {
            _observedQuestions[index].SetNumber(index + 1);
            _observedQuestions[index].PropertyChanged += QuestionChanged;
        }
        NotifyAnswerCountChanged();
        OnPropertyChanged(nameof(ContextText));
    }

    private void QuestionChanged(object? sender, PropertyChangedEventArgs e)
    {
        if (e.PropertyName == nameof(QuestionPromptItem.HasAnswer)) NotifyAnswerCountChanged();
    }

    private void NotifyAnswerCountChanged()
    {
        OnPropertyChanged(nameof(AnsweredCount));
        OnPropertyChanged(nameof(AnsweredText));
        OnPropertyChanged(nameof(CanSubmit));
    }

    internal bool TryBeginResolution(bool skip)
    {
        if (IsResolving || (!skip && !CanSubmit)) return false;
        _isSkipping = skip;
        _isResolving = true;
        _errorKey = null;
        NotifyResolutionChanged();
        return true;
    }

    internal void FinishResolution(bool failed)
    {
        _errorKey = null;
        if (failed) _errorKey = _isSkipping ? "LocQuestionSkipFailed" : "LocQuestionSubmitFailed";
        _isResolving = false;
        NotifyResolutionChanged();
    }

    private void NotifyResolutionChanged()
    {
        OnPropertyChanged(nameof(IsResolving));
        OnPropertyChanged(nameof(IsEditable));
        OnPropertyChanged(nameof(IsSubmitting));
        OnPropertyChanged(nameof(IsSkipping));
        OnPropertyChanged(nameof(CanSubmit));
        OnPropertyChanged(nameof(HasError));
        OnPropertyChanged(nameof(ErrorText));
    }

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
