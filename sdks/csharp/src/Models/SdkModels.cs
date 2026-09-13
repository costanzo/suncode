using System.Text.Json;
using System.Text.Json.Serialization;

namespace SunCode.Sdk.Models;

public sealed record VersionResult(string Version);

public sealed record DatabaseHealth(
    bool Ok,
    [property: JsonPropertyName("journal_mode")] string JournalMode);

public sealed record HealthResult(bool Ok, string Agent, DatabaseHealth Database);

public sealed record RecoveryStatus(
    string Status,
    [property: JsonPropertyName("pending_operations")] int PendingOperations);

public sealed record DiagnosticsResult(
    HealthResult Health,
    RecoveryStatus Recovery,
    IReadOnlyList<CredentialState> Credentials,
    [property: JsonPropertyName("active_project_id")] string? ActiveProjectId);

public sealed record CredentialState(string Provider, bool Configured);

public sealed record CredentialsResult(IReadOnlyList<CredentialState> Credentials);

public sealed record CredentialUpdate(string Provider, bool Configured);

public sealed record ProviderEndpointUpdate(string ProviderId, string Endpoint);

public sealed record ModelCapabilities(
    bool Streaming,
    [property: JsonPropertyName("tool_use")] bool ToolUse,
    bool Vision,
    [property: JsonPropertyName("structured_output")] bool StructuredOutput,
    bool Cancellation,
    [property: JsonPropertyName("reasoning_effort")] bool ReasoningEffort);

public sealed record ModelLimits(
    [property: JsonPropertyName("max_input_tokens")] ulong? MaxInputTokens,
    [property: JsonPropertyName("auto_compact_tokens")] ulong? AutoCompactTokens,
    [property: JsonPropertyName("max_output_tokens")] ulong? MaxOutputTokens);

public sealed record ModelDescriptor(
    string Provider,
    [property: JsonPropertyName("provider_label")] string ProviderLabel,
    string Id,
    [property: JsonPropertyName("wire_model")] string WireModel,
    [property: JsonPropertyName("api_base")] string ApiBase,
    [property: JsonPropertyName("default_api_base")] string DefaultApiBase,
    ModelCapabilities Capabilities,
    [property: JsonPropertyName("reasoning_efforts")] IReadOnlyList<string> ReasoningEfforts,
    ModelLimits Limits,
    string Availability);

public sealed record ModelsResult(IReadOnlyList<ModelDescriptor> Models);

public sealed record SettingRecord(
    string Key,
    JsonElement Value,
    string Scope,
    [property: JsonPropertyName("scope_id")] string ScopeId);

public sealed record SettingsResult(IReadOnlyList<SettingRecord> Settings);

public sealed record SettingUpdate(
    bool Saved,
    string Key,
    string Scope,
    [property: JsonPropertyName("scope_id")] string ScopeId);

public sealed record ProjectRecord(
    string ProjectId,
    string CanonicalRoot,
    string DisplayName,
    string CreatedAt,
    string UpdatedAt,
    string LastOpenedAt,
    string? ArchivedAt);

public sealed record ProjectsResult(IReadOnlyList<ProjectRecord> Projects);

public sealed record McpServer(
    string McpServerId,
    string DisplayName,
    string ToolPrefix,
    string TransportType,
    string? Command,
    IReadOnlyList<string> Arguments,
    string? WorkingDirectory,
    string? Url,
    IReadOnlyList<string> EnvironmentKeys,
    IReadOnlyList<string> HeaderKeys,
    ulong StartupTimeoutSeconds,
    ulong RequestTimeoutSeconds,
    bool Enabled,
    long SortOrder,
    ulong Revision,
    string RuntimeStatus,
    int ToolCount,
    string? Error,
    string CreatedAt,
    string UpdatedAt);

public sealed record McpServersResult(IReadOnlyList<McpServer> Servers);

public sealed record McpLoadProgress(
    int Total,
    int Settled,
    int Connected,
    int Failed,
    bool Loading);

public sealed record McpServerDeleteResult(string McpServerId, bool Removed);

public sealed record McpSecretChanges(
    IReadOnlyDictionary<string, string> Set,
    IReadOnlyList<string> Remove)
{
    public static McpSecretChanges Empty { get; } = new(
        new Dictionary<string, string>(),
        Array.Empty<string>());
}

[JsonPolymorphic(TypeDiscriminatorPropertyName = "kind")]
[JsonDerivedType(typeof(McpStdioTransportRequest), "stdio")]
[JsonDerivedType(typeof(McpStreamableHttpTransportRequest), "streamable_http")]
public abstract record McpTransportRequest(
    ulong StartupTimeoutSeconds,
    ulong RequestTimeoutSeconds);

public sealed record McpStdioTransportRequest(
    string Command,
    IReadOnlyList<string> Arguments,
    string WorkingDirectory,
    McpSecretChanges? Environment,
    ulong StartupTimeoutSeconds = 15,
    ulong RequestTimeoutSeconds = 60)
    : McpTransportRequest(StartupTimeoutSeconds, RequestTimeoutSeconds);

public sealed record McpStreamableHttpTransportRequest(
    string Url,
    McpSecretChanges? Headers,
    ulong StartupTimeoutSeconds = 15,
    ulong RequestTimeoutSeconds = 60)
    : McpTransportRequest(StartupTimeoutSeconds, RequestTimeoutSeconds);

public sealed record McpServerWriteRequest(
    string DisplayName,
    McpTransportRequest Transport,
    bool Enabled = true,
    long SortOrder = 0);

public sealed record CreateMcpServerRequest(
    string? ProjectId,
    string IdempotencyKey,
    McpServerWriteRequest Server);

public sealed record UpdateMcpServerRequest(
    string? ProjectId,
    string ServerId,
    ulong ExpectedRevision,
    string IdempotencyKey,
    McpServerWriteRequest Server);

public sealed record SetMcpServerEnabledRequest(
    string? ProjectId,
    string ServerId,
    ulong ExpectedRevision,
    string IdempotencyKey,
    bool Enabled);

public sealed record DeleteMcpServerRequest(
    string ServerId,
    ulong ExpectedRevision,
    string IdempotencyKey);

public sealed record ProjectDependency(
    string DependencyId,
    string ProjectId,
    string DisplayName,
    string CreatedAt);

public sealed record ProjectDependenciesResult(
    [property: JsonPropertyName("project_id")] string ProjectId,
    IReadOnlyList<ProjectDependency> Dependencies);

public sealed record DependencyRemoval(
    [property: JsonPropertyName("dependency_id")] string DependencyId,
    bool Removed);

public sealed record DirectoryEntry(string Name, string Path, string Kind, bool Expandable);

public sealed record ProjectDirectoryResult(
    string ProjectId,
    string? DependencyId,
    string Path,
    IReadOnlyList<DirectoryEntry> Entries,
    bool Truncated);

public sealed record ProjectFileResult(
    string ProjectId,
    string? DependencyId,
    string Path,
    string Content,
    ulong Bytes);

public sealed record SessionRecord(
    string SessionId,
    string? ProjectId,
    string? Title,
    string? ModelId,
    string? ReasoningEffort,
    string Status,
    string CreatedAt,
    string UpdatedAt,
    string LastActivityAt,
    string? ArchivedAt,
    string? PinAt);

public sealed record SessionsResult(
    [property: JsonPropertyName("project_id")] string ProjectId,
    IReadOnlyList<SessionRecord> Sessions,
    [property: JsonPropertyName("sessionStates")] IReadOnlyDictionary<string, string> SessionStates);

public sealed record SessionImage(
    string ImageId,
    string SessionId,
    string DisplayName,
    string SourceKind,
    string? OriginalPath,
    string StoragePath,
    string ThumbnailBase64,
    string CreatedAt);

public sealed record SessionImagesResult(
    [property: JsonPropertyName("session_id")] string SessionId,
    IReadOnlyList<SessionImage> Images);

public sealed record SessionImageRemoval(
    [property: JsonPropertyName("session_id")] string SessionId,
    [property: JsonPropertyName("image_id")] string ImageId,
    bool Removed);

public sealed record AgentContentPart(
    [property: JsonPropertyName("type")] string Kind,
    string Text);

public sealed record AgentToolCall(
    [property: JsonPropertyName("call_id")] string CallId,
    string Name,
    JsonElement Arguments);

public sealed record AgentMessage(
    string Role,
    IReadOnlyList<AgentContentPart>? Content,
    [property: JsonPropertyName("tool_calls")] IReadOnlyList<AgentToolCall>? ToolCalls,
    [property: JsonPropertyName("tool_call_id")] string? ToolCallId)
{
    public string Text => string.Join(
        "\n",
        (Content ?? []).Where(part => part.Kind == "text").Select(part => part.Text));
}

public sealed record AgentUsage(
    [property: JsonPropertyName("input_tokens")] ulong InputTokens,
    [property: JsonPropertyName("output_tokens")] ulong OutputTokens,
    [property: JsonPropertyName("total_tokens")] ulong TotalTokens,
    [property: JsonPropertyName("cache_read_tokens")] ulong? CacheReadTokens = null,
    [property: JsonPropertyName("cache_miss_tokens")] ulong? CacheMissTokens = null,
    [property: JsonPropertyName("cache_write_tokens")] ulong? CacheWriteTokens = null,
    [property: JsonPropertyName("reasoning_tokens")] ulong? ReasoningTokens = null);

public sealed record SessionTurnTodo(
    string TurnId,
    long Ordinal,
    string Content,
    string Status,
    string Priority,
    string CreatedAt,
    string UpdatedAt,
    string? CompletedAt);

public sealed record SessionCallMessage(
    string MessageId,
    string SessionId,
    string? TurnId,
    string? SessionCallId,
    string Role,
    AgentMessage Message,
    string CreatedAt);

public sealed record SessionCallToolUse(
    string TurnId,
    string ToolCallId,
    string? SessionCallId,
    string Name,
    JsonElement? Request,
    JsonElement? Result,
    string State,
    long? Ordinal,
    string CreatedAt,
    string UpdatedAt,
    string? CompletedAt,
    string? ErrorCode);

public sealed record SessionConversationTurn(
    string TurnId,
    string State,
    string CreatedAt,
    string? StartedAt,
    string? CompletedAt,
    IReadOnlyList<SessionCallMessage> Messages,
    IReadOnlyList<SessionCallToolUse> ToolUses,
    IReadOnlyList<SessionTurnTodo> Todos);

public sealed record QuestionOption(
    string Label,
    string Description);

public sealed record QuestionPrompt(
    string Question,
    string Header,
    IReadOnlyList<QuestionOption> Options,
    bool Multiple,
    bool Custom);

public sealed record PendingQuestion(
    [property: JsonPropertyName("request_id")] string RequestId,
    [property: JsonPropertyName("session_id")] string SessionId,
    [property: JsonPropertyName("turn_id")] string TurnId,
    [property: JsonPropertyName("tool_call_id")] string ToolCallId,
    IReadOnlyList<QuestionPrompt> Questions);

public sealed record SessionSnapshot(
    SessionRecord Session,
    IReadOnlyList<AgentMessage> Messages,
    [property: JsonPropertyName("conversationTurns")] IReadOnlyList<SessionConversationTurn> ConversationTurns,
    IReadOnlyList<SessionImage> Images,
    [property: JsonPropertyName("pendingQuestion")] PendingQuestion? PendingQuestion);

public sealed record SessionUsageResult(
    [property: JsonPropertyName("session_id")] string SessionId,
    [property: JsonPropertyName("input_tokens")] ulong InputTokens,
    [property: JsonPropertyName("output_tokens")] ulong OutputTokens,
    [property: JsonPropertyName("total_tokens")] ulong TotalTokens);

public sealed record ProviderError(
    string Code,
    string Message,
    bool Retryable);

public sealed record ProviderExchange(
    [property: JsonPropertyName("exchange_id")] string ExchangeId,
    [property: JsonPropertyName("session_id")] string SessionId,
    [property: JsonPropertyName("turn_id")] string TurnId,
    string Provider,
    [property: JsonPropertyName("model_id")] string ModelId,
    [property: JsonPropertyName("wire_model")] string WireModel,
    [property: JsonPropertyName("provider_request_id")] string? ProviderRequestId,
    [property: JsonPropertyName("provider_response_id")] string? ProviderResponseId,
    string State,
    int Iteration,
    [property: JsonPropertyName("started_at")] string StartedAt,
    [property: JsonPropertyName("completed_at")] string? CompletedAt,
    [property: JsonPropertyName("input_messages")] IReadOnlyList<AgentMessage> InputMessages,
    [property: JsonPropertyName("output_message")] AgentMessage? OutputMessage,
    [property: JsonPropertyName("tool_calls")] IReadOnlyList<AgentToolCall> ToolCalls,
    AgentUsage? Usage,
    [property: JsonPropertyName("finish_reason")] string? FinishReason,
    JsonElement? Error);

public sealed record SessionTraceTurn(
    [property: JsonPropertyName("turn_id")] string TurnId,
    [property: JsonPropertyName("session_id")] string SessionId,
    string State,
    [property: JsonPropertyName("model_id")] string? ModelId,
    [property: JsonPropertyName("created_at")] string CreatedAt,
    [property: JsonPropertyName("updated_at")] string UpdatedAt,
    [property: JsonPropertyName("started_at")] string? StartedAt,
    [property: JsonPropertyName("completed_at")] string? CompletedAt,
    [property: JsonPropertyName("error_code")] string? ErrorCode,
    [property: JsonPropertyName("input_tokens")] ulong InputTokens,
    [property: JsonPropertyName("output_tokens")] ulong OutputTokens,
    [property: JsonPropertyName("total_tokens")] ulong TotalTokens);

public sealed record ProviderExchangesResult(
    [property: JsonPropertyName("session_id")] string SessionId,
    IReadOnlyList<SessionTraceTurn> Turns,
    IReadOnlyList<ProviderExchange> Exchanges);

public sealed record ProviderExchangeDetails(
    [property: JsonPropertyName("exchange_id")] string ExchangeId,
    [property: JsonPropertyName("session_id")] string SessionId,
    [property: JsonPropertyName("turn_id")] string TurnId,
    string Provider,
    [property: JsonPropertyName("model_id")] string ModelId,
    [property: JsonPropertyName("wire_model")] string WireModel,
    [property: JsonPropertyName("provider_request_id")] string? ProviderRequestId,
    [property: JsonPropertyName("provider_response_id")] string? ProviderResponseId,
    string State,
    int Iteration,
    [property: JsonPropertyName("started_at")] string StartedAt,
    [property: JsonPropertyName("completed_at")] string? CompletedAt,
    [property: JsonPropertyName("input_messages")] IReadOnlyList<AgentMessage> InputMessages,
    [property: JsonPropertyName("output_message")] AgentMessage? OutputMessage,
    [property: JsonPropertyName("tool_calls")] IReadOnlyList<AgentToolCall> ToolCalls,
    AgentUsage? Usage,
    [property: JsonPropertyName("finish_reason")] string? FinishReason,
    JsonElement? Error,
    IReadOnlyList<SessionCallMessage> Messages,
    [property: JsonPropertyName("tool_uses")] IReadOnlyList<SessionCallToolUse> ToolUses);

public sealed record CheckpointManifest(
    [property: JsonPropertyName("manifest_id")] string ManifestId,
    [property: JsonPropertyName("session_id")] string SessionId,
    [property: JsonPropertyName("turn_id")] string? TurnId,
    string Status,
    [property: JsonPropertyName("created_at")] string CreatedAt,
    [property: JsonPropertyName("updated_at")] string UpdatedAt,
    [property: JsonPropertyName("expires_at")] string ExpiresAt,
    [property: JsonPropertyName("restored_at")] string? RestoredAt);

public sealed record CheckpointItem(
    [property: JsonPropertyName("checkpoint_id")] string CheckpointId,
    [property: JsonPropertyName("manifest_id")] string? ManifestId,
    [property: JsonPropertyName("session_id")] string SessionId,
    [property: JsonPropertyName("turn_id")] string? TurnId,
    [property: JsonPropertyName("tool_call_id")] string? ToolCallId,
    [property: JsonPropertyName("relative_path")] string? RelativePath,
    string Status,
    [property: JsonPropertyName("created_at")] string CreatedAt,
    [property: JsonPropertyName("restored_at")] string? RestoredAt,
    [property: JsonPropertyName("invalidated_at")] string? InvalidatedAt,
    long? Ordinal);

public sealed record CheckpointsResult(
    [property: JsonPropertyName("session_id")] string SessionId,
    IReadOnlyList<CheckpointManifest> Checkpoints);

public sealed record CheckpointDetails(
    CheckpointManifest Manifest,
    IReadOnlyList<CheckpointItem> Items);

public sealed record RestoreOutcome(
    [property: JsonPropertyName("manifest_id")] string ManifestId,
    string Status,
    [property: JsonPropertyName("restored_items")] int RestoredItems);

public sealed record GitFileStatus(
    string Path,
    [property: JsonPropertyName("old_path")] string? OldPath,
    string Status,
    [property: JsonPropertyName("index_status")] string? IndexStatus,
    [property: JsonPropertyName("worktree_status")] string? WorktreeStatus,
    bool Staged,
    bool Unstaged,
    bool Conflicted,
    bool Binary,
    ulong Additions,
    ulong Deletions);

public sealed record GitStatusResult(
    bool Repository,
    string? Branch,
    bool Detached,
    [property: JsonPropertyName("head_oid")] string? HeadOid,
    [property: JsonPropertyName("changed_files")] int ChangedFiles,
    ulong Additions,
    ulong Deletions,
    int Conflicts,
    IReadOnlyList<GitFileStatus> Files,
    bool Truncated,
    [property: JsonPropertyName("unsupported_paths")] int UnsupportedPaths);

public sealed record GitDiffLine(
    string Kind,
    [property: JsonPropertyName("old_line")] uint? OldLine,
    [property: JsonPropertyName("new_line")] uint? NewLine,
    string Text);

public sealed record GitDiffHunk(
    string Header,
    [property: JsonPropertyName("old_start")] uint OldStart,
    [property: JsonPropertyName("old_lines")] uint OldLines,
    [property: JsonPropertyName("new_start")] uint NewStart,
    [property: JsonPropertyName("new_lines")] uint NewLines,
    IReadOnlyList<GitDiffLine> Lines);

public sealed record GitDiffFileResult(
    string Scope,
    string Path,
    [property: JsonPropertyName("old_path")] string? OldPath,
    string Status,
    bool Binary,
    int Additions,
    int Deletions,
    IReadOnlyList<GitDiffHunk> Hunks,
    string Patch,
    bool Truncated);

public sealed record TurnResponse(
    string Status,
    [property: JsonPropertyName("turn_id")] string? TurnId,
    [property: JsonPropertyName("tool_call_id")] string? ToolCallId,
    [property: JsonPropertyName("approval_id")] string? ApprovalId,
    [property: JsonPropertyName("request_id")] string? RequestId,
    [property: JsonPropertyName("queued_id")] string? QueuedId,
    [property: JsonPropertyName("active_turn_id")] string? ActiveTurnId,
    int? Position,
    AgentMessage? Message,
    AgentUsage? Usage,
    uint? Iterations,
    [property: JsonPropertyName("tool_calls")] uint? ToolCalls);

public sealed record ApprovalRecord(
    [property: JsonPropertyName("approval_id")] string ApprovalId,
    [property: JsonPropertyName("project_id")] string? ProjectId,
    [property: JsonPropertyName("session_id")] string SessionId,
    [property: JsonPropertyName("turn_id")] string TurnId,
    [property: JsonPropertyName("tool_call_id")] string ToolCallId,
    string Operation,
    JsonElement Arguments,
    string Status,
    string? Decision,
    [property: JsonPropertyName("decision_source")] string? DecisionSource,
    [property: JsonPropertyName("created_at")] string CreatedAt,
    [property: JsonPropertyName("updated_at")] string UpdatedAt);

public sealed record ApprovalOutcome(
    [property: JsonPropertyName("approval_id")] string ApprovalId,
    string Decision);

public sealed record QuestionOutcome(
    [property: JsonPropertyName("request_id")] string RequestId,
    string Status);

public sealed record CancellationOutcome(
    [property: JsonPropertyName("turn_id")] string TurnId,
    string Status);

public sealed record SettingScope(string? ProjectId = null, string? SessionId = null);

public sealed record SetSettingRequest(
    string Scope,
    string? ProjectId,
    string? SessionId,
    string Key,
    JsonElement Value);

public sealed record SetCredentialRequest(string Provider, string ApiKey);

public sealed record ProviderEndpointRequest(string Provider, string Endpoint);

public sealed record OpenProjectRequest(string Path, string? DisplayName = null);

public sealed record CreateSessionRequest(string ProjectId, string? Title, string? Model);

public sealed record AddSessionImageRequest(
    string SessionId,
    string DisplayName,
    string SourceKind,
    string Extension,
    string BytesBase64,
    string ThumbnailBase64,
    string? OriginalPath = null);

public sealed record SubmitTurnRequest(
    string SessionId,
    string Input,
    string IdempotencyKey,
    string? Model,
    string? ReasoningEffort,
    IReadOnlyList<string> ImageIds);

public sealed record ApprovalDecisionRequest(string ApprovalId, ApprovalDecision Decision);

public enum ApprovalDecision
{
    Deny,
    AllowOnce,
    AllowSession
}

public sealed record AgentEvent(
    [property: JsonPropertyName("session_id")] string SessionId,
    [property: JsonPropertyName("occurred_at")] string OccurredAt,
    [property: JsonPropertyName("event_type")] string EventType,
    AgentEventPayload Payload);

public sealed record AgentEventPayload(
    [property: JsonPropertyName("turn_id")] string? TurnId = null,
    string? State = null,
    [property: JsonPropertyName("model_id")] string? ModelId = null,
    [property: JsonPropertyName("submission_idempotency_key")] string? SubmissionIdempotencyKey = null,
    string? Reason = null,
    [property: JsonPropertyName("call_id")] string? CallId = null,
    [property: JsonPropertyName("tool_call_id")] string? ToolCallId = null,
    string? Name = null,
    JsonElement? Arguments = null,
    JsonElement? Result = null,
    long? Ordinal = null,
    string? Stream = null,
    [property: JsonPropertyName("chunk_base64")] string? ChunkBase64 = null,
    [property: JsonPropertyName("message_id")] string? MessageId = null,
    [property: JsonPropertyName("queued_id")] string? QueuedId = null,
    [property: JsonPropertyName("queued_idempotency_key")] string? QueuedIdempotencyKey = null,
    AgentMessage? Message = null,
    AgentUsage? Usage = null,
    string? Text = null,
    [property: JsonPropertyName("input_messages")] IReadOnlyList<AgentMessage>? InputMessages = null,
    [property: JsonPropertyName("active_turn_id")] string? ActiveTurnId = null,
    int? Position = null,
    uint? Iterations = null,
    [property: JsonPropertyName("exchange_id")] string? ExchangeId = null,
    string? Provider = null,
    [property: JsonPropertyName("wire_model")] string? WireModel = null,
    [property: JsonPropertyName("started_at")] string? StartedAt = null,
    [property: JsonPropertyName("completed_at")] string? CompletedAt = null,
    int? OriginalCharacters = null,
    int? RetainedCharacters = null,
    int? OriginalTokens = null,
    int? RetainedTokens = null,
    int? DroppedMessages = null,
    AgentContextSummary? Summary = null,
    [property: JsonPropertyName("output_message")] AgentMessage? OutputMessage = null,
    ProviderError? Error = null,
    [property: JsonPropertyName("provider_request_id")] string? ProviderRequestId = null,
    [property: JsonPropertyName("provider_response_id")] string? ProviderResponseId = null,
    [property: JsonPropertyName("finish_reason")] string? FinishReason = null,
    [property: JsonPropertyName("approval_id")] string? ApprovalId = null,
    string? Operation = null,
    string? Decision = null,
    [property: JsonPropertyName("request_id")] string? RequestId = null,
    IReadOnlyList<QuestionPrompt>? Questions = null,
    IReadOnlyList<IReadOnlyList<string>>? Answers = null,
    IReadOnlyList<AgentTodoEventItem>? Todos = null,
    [property: JsonPropertyName("manifest_id")] string? ManifestId = null,
    [property: JsonPropertyName("checkpoint_id")] string? CheckpointId = null,
    string? Path = null);

public sealed record AgentTodoEventItem(
    string Content,
    string Status,
    string Priority);

public sealed record AgentContextSummary(
    string Objective,
    [property: JsonPropertyName("important_constraints")] IReadOnlyList<string> ImportantConstraints,
    [property: JsonPropertyName("completed_work")] IReadOnlyList<string> CompletedWork,
    [property: JsonPropertyName("active_work")] IReadOnlyList<string> ActiveWork,
    IReadOnlyList<string> Blockers,
    [property: JsonPropertyName("next_action")] string NextAction);

public sealed record ReplyQuestionRequest(
    string RequestId,
    IReadOnlyList<IReadOnlyList<string>> Answers);
