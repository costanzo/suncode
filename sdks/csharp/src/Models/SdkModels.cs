using System.Text.Json;
using System.Text.Json.Serialization;

namespace SunCode.Sdk.Models;

public sealed record VersionResult(string Version);

public sealed record HealthResult(bool Ok, string Agent, JsonElement Database);

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
    string ProviderLabel,
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

public sealed record SessionSnapshot(
    SessionRecord Session,
    JsonElement Messages,
    JsonElement ConversationTurns,
    IReadOnlyList<SessionImage> Images,
    [property: JsonPropertyName("pendingQuestion")] JsonElement? PendingQuestion);

public sealed record SessionUsageResult(
    [property: JsonPropertyName("session_id")] string SessionId,
    [property: JsonPropertyName("input_tokens")] ulong InputTokens,
    [property: JsonPropertyName("output_tokens")] ulong OutputTokens,
    [property: JsonPropertyName("total_tokens")] ulong TotalTokens);

public sealed record ProviderExchangesResult(
    [property: JsonPropertyName("session_id")] string SessionId,
    JsonElement Turns,
    JsonElement Exchanges);

public sealed record ProviderExchangeDetails(
    JsonElement Exchange,
    JsonElement Messages,
    JsonElement ToolUses);

public sealed record CheckpointsResult(string SessionId, JsonElement Checkpoints);

public sealed record CheckpointDetails(JsonElement Manifest, JsonElement Items);

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

public sealed record GitDiffFileResult(
    string Scope,
    string Path,
    [property: JsonPropertyName("old_path")] string? OldPath,
    string Status,
    bool Binary,
    int Additions,
    int Deletions,
    JsonElement Hunks,
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
    JsonElement? Message,
    JsonElement? Usage,
    uint? Iterations,
    [property: JsonPropertyName("tool_calls")] uint? ToolCalls);

public sealed record ApprovalRecord(
    [property: JsonPropertyName("approvalId")] string ApprovalId,
    [property: JsonPropertyName("projectId")] string? ProjectId,
    [property: JsonPropertyName("sessionId")] string SessionId,
    [property: JsonPropertyName("turnId")] string TurnId,
    [property: JsonPropertyName("toolCallId")] string ToolCallId,
    string Operation,
    JsonElement Arguments,
    string Status,
    string? Decision,
    [property: JsonPropertyName("decisionSource")] string? DecisionSource,
    [property: JsonPropertyName("createdAt")] string CreatedAt,
    [property: JsonPropertyName("updatedAt")] string UpdatedAt);

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

public sealed record SetSettingRequest(string Scope, string? ProjectId, string? SessionId, string Key, object Value);

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

public sealed record ReplyQuestionRequest(string RequestId, IReadOnlyList<IReadOnlyList<string>> Answers);
