using System.Text.Json;
using SunCode.Desktop.Models;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

// Converts SDK provider-exchange records into trace presentation items and
// matches them against the drawer filter.
internal static class ProviderTraceMapper
{
    public static bool Matches(ProviderTraceItem trace, string filter)
    {
        if (string.IsNullOrWhiteSpace(filter)) return true;
        return trace.ExchangeId.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || trace.TurnId.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || trace.Provider.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || trace.ModelId.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || trace.WireModel.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || trace.ProviderRequestId.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || trace.ProviderResponseId.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || trace.OutputText.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || trace.ToolCallsText.Contains(filter, StringComparison.OrdinalIgnoreCase);
    }

    public static bool TurnMatches(ProviderTraceTurnItem turn, string filter)
    {
        if (string.IsNullOrWhiteSpace(filter)) return true;
        return turn.TurnId.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || turn.ModelId.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || turn.State.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || turn.Sequence.ToString().Contains(filter, StringComparison.OrdinalIgnoreCase);
    }

    public static ProviderTraceTurnItem TurnFromSdk(
        SessionTraceTurn item,
        int sequence,
        IReadOnlyList<ProviderTraceItem> calls) =>
        new(
            item.TurnId,
            item.State,
            item.ModelId ?? string.Empty,
            item.CreatedAt,
            item.StartedAt ?? string.Empty,
            item.CompletedAt ?? string.Empty,
            (long)item.InputTokens,
            (long)item.OutputTokens,
            (long)item.TotalTokens,
            sequence,
            calls);

    public static ProviderTraceItem FromSdk(ProviderExchange item)
    {
        var usage = item.Usage;
        var result = new ProviderTraceItem(
            item.ExchangeId,
            item.TurnId,
            item.Provider,
            item.ModelId,
            item.WireModel,
            item.ProviderRequestId ?? string.Empty,
            item.ProviderResponseId ?? string.Empty,
            item.State,
            item.Iteration,
            item.StartedAt,
            item.CompletedAt ?? string.Empty,
            usage is null ? null : (long)usage.InputTokens,
            usage is null ? null : (long)usage.OutputTokens,
            usage?.CacheReadTokens is { } cacheRead ? (long)cacheRead : null,
            usage?.CacheWriteTokens is { } cacheWrite ? (long)cacheWrite : null,
            usage is null ? null : (long)usage.TotalTokens,
            item.FinishReason ?? string.Empty,
            MessageDisplayText(item.OutputMessage),
            JsonSerializer.Serialize(item.ToolCalls, DisplayJson.Options),
            Pretty(item.Error),
            [],
            []);
        return result;
    }

    public static ProviderTraceItem FromSdk(ProviderExchangeDetails item)
    {
        var usage = item.Usage;
        var messages = item.Messages.Select(message => new ProviderTraceMessageItem(
            message.MessageId,
            message.Role,
            message.Message.Text,
            message.CreatedAt)).ToList();
        var tools = item.ToolUses.Select(tool => new ProviderTraceToolItem(
            tool.ToolCallId,
            tool.Name,
            tool.State,
            Pretty(tool.Request),
            Pretty(tool.Result),
            tool.ErrorCode ?? string.Empty,
            tool.CreatedAt)).ToList();
        var result = new ProviderTraceItem(
            item.ExchangeId,
            item.TurnId,
            item.Provider,
            item.ModelId,
            item.WireModel,
            item.ProviderRequestId ?? string.Empty,
            item.ProviderResponseId ?? string.Empty,
            item.State,
            item.Iteration,
            item.StartedAt,
            item.CompletedAt ?? string.Empty,
            usage is null ? null : (long)usage.InputTokens,
            usage is null ? null : (long)usage.OutputTokens,
            usage?.CacheReadTokens is { } cacheRead ? (long)cacheRead : null,
            usage?.CacheWriteTokens is { } cacheWrite ? (long)cacheWrite : null,
            usage is null ? null : (long)usage.TotalTokens,
            item.FinishReason ?? string.Empty,
            MessageDisplayText(item.OutputMessage),
            JsonSerializer.Serialize(item.ToolCalls, DisplayJson.Options),
            Pretty(item.Error),
            messages,
            tools);
        return result;
    }

    private static string MessageDisplayText(AgentMessage? message)
    {
        if (message is null) return string.Empty;
        var text = message.Text;
        return string.IsNullOrWhiteSpace(text)
            ? JsonSerializer.Serialize(message, DisplayJson.Options)
            : text;
    }

    private static string Pretty(JsonElement? node) =>
        node is null || node.Value.ValueKind is JsonValueKind.Undefined or JsonValueKind.Null
            ? string.Empty
            : node.Value.GetRawText();
}
