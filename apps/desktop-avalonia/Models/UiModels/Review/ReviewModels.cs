using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Text.Encodings.Web;
using System.Text.Json;
using System.Text.Json.Nodes;
using Avalonia.Media.Imaging;
using SunCode.Desktop.Infrastructure;

namespace SunCode.Desktop.Models;

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
