namespace SunCode.Desktop.Models;

/// <summary>Agent event-type vocabulary consumed by the desktop client.</summary>
public static class AgentEventTypes
{
    public const string TurnState = "turn.state";

    public const string MessageUser = "message.user";
    public const string MessageAssistant = "message.assistant";
    public const string MessageTool = "message.tool";
    public const string AssistantDelta = "assistant.delta";

    public const string ToolRequested = "tool.requested";
    public const string ToolState = "tool.state";
    public const string ToolResult = "tool.result";
    public const string ToolOutput = "tool.output";

    public const string TodoUpdated = "todo.updated";
    public const string ContextCompacted = "context.compacted";

    public const string ApprovalRequested = "approval.requested";
    public const string ApprovalResolved = "approval.resolved";

    public const string QuestionAsked = "question.asked";
    public const string QuestionReplied = "question.replied";
    public const string QuestionRejected = "question.rejected";

    public const string CheckpointPrefix = "checkpoint.";
    public const string CheckpointCaptured = "checkpoint.captured";
    public const string CheckpointRestoreFailed = "checkpoint.restore_failed";

    public const string ProviderExchangePrefix = "provider.exchange.";
    public const string ProviderExchangeStarted = "provider.exchange.started";
    public const string ProviderExchangeProgress = "provider.exchange.progress";
    public const string ProviderExchangeCompleted = "provider.exchange.completed";
    public const string ProviderExchangeFailed = "provider.exchange.failed";

    public const string ResyncRequired = "resync.required";

    public static bool IsMessage(string type) => type is MessageUser or MessageAssistant;

    public static bool IsTool(string type) => type is ToolRequested or ToolState or ToolResult or ToolOutput;

    public static bool IsPrompt(string type) =>
        type is ApprovalRequested or ApprovalResolved or QuestionAsked or QuestionReplied or QuestionRejected;

    public static bool IsCheckpoint(string type) => type.StartsWith(CheckpointPrefix, StringComparison.Ordinal);

    public static bool IsProviderExchange(string type) =>
        type.StartsWith(ProviderExchangePrefix, StringComparison.Ordinal);
}
