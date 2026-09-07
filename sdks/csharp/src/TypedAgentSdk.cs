using System.Text.Json;
using System.Text.Json.Nodes;
using SunCode.Sdk.Models;

namespace SunCode.Sdk;

public sealed partial class AgentSdk
{
    private static readonly JsonSerializerOptions TypedJsonOptions = new()
    {
        PropertyNameCaseInsensitive = true,
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        NumberHandling = System.Text.Json.Serialization.JsonNumberHandling.AllowReadingFromString
    };

    private static T Deserialize<T>(JsonObject body)
        => body.Deserialize<T>(TypedJsonOptions)
           ?? throw new SdkException("invalid_response", $"Agent returned an empty {typeof(T).Name} response");

    private static async Task<T> Typed<T>(Task<JsonObject> operation)
        => Deserialize<T>(await operation.ConfigureAwait(false));

    public static Task<VersionResult> GetVersionAsync() => Typed<VersionResult>(VersionAsync());
    public Task<HealthResult> GetHealthAsync() => Typed<HealthResult>(HealthAsync());
    public Task<DiagnosticsResult> GetDiagnosticsAsync() => Typed<DiagnosticsResult>(DiagnosticsAsync());
    public Task<ModelsResult> GetModelsAsync() => Typed<ModelsResult>(ListModelsAsync());
    public Task<CredentialsResult> GetCredentialsAsync() => Typed<CredentialsResult>(ListCredentialsAsync());
    public Task<ProjectsResult> GetProjectsAsync() => Typed<ProjectsResult>(ListProjectsAsync());

    public Task<SettingsResult> GetSettingsAsync(SettingScope scope) => Typed<SettingsResult>(
        scope.ProjectId is null && scope.SessionId is null
            ? ListSettingsAsync()
            : scope.SessionId is null
                ? ListProjectSettingsAsync(scope.ProjectId!)
                : ListSessionSettingsAsync(scope.ProjectId ?? throw new ArgumentException("ProjectId is required when SessionId is set", nameof(scope)), scope.SessionId));

    public Task<SettingUpdate> SetSettingAsync(SetSettingRequest request) => Typed<SettingUpdate>(
        request.Scope switch
        {
            "global" => SetSettingAsync(request.Key, request.Value),
            "project" when request.ProjectId is not null => SetProjectSettingAsync(request.ProjectId, request.Key, request.Value),
            "session" when request.SessionId is not null && request.Key == "full_control" && request.Value is bool enabled => SetSessionFullControlAsync(request.SessionId, enabled),
            _ => throw new ArgumentException("Scope and identifiers do not form a supported setting request", nameof(request))
        });

    public Task<CredentialUpdate> SetCredentialAsync(SetCredentialRequest request) => Typed<CredentialUpdate>(SetCredentialAsync(request.Provider, request.ApiKey));
    public Task<CredentialUpdate> RemoveCredentialTypedAsync(string provider) => Typed<CredentialUpdate>(RemoveCredentialAsync(provider));
    public Task<ProviderEndpointUpdate> SetProviderEndpointAsync(ProviderEndpointRequest request) => Typed<ProviderEndpointUpdate>(SetProviderEndpointAsync(request.Provider, request.Endpoint));
    public Task<ProjectRecord> OpenProjectAsync(OpenProjectRequest request) => Typed<ProjectRecord>(WithNullableUtf8Async(
        [request.Path, request.DisplayName], values => NativeMethods.suncode_agent_sdk_open_project(_handle, values[0], values[1])));
    public Task<ProjectRecord> SelectProjectTypedAsync(string projectId) => Typed<ProjectRecord>(SelectProjectAsync(projectId));
    public Task<ProjectDependenciesResult> ListProjectDependenciesTypedAsync(string projectId) => Typed<ProjectDependenciesResult>(ListProjectDependenciesAsync(projectId));
    public Task<DependencyRemoval> RemoveProjectDependencyTypedAsync(string projectId, string dependencyId) => Typed<DependencyRemoval>(RemoveProjectDependencyAsync(projectId, dependencyId));
    public Task<ProjectDirectoryResult> ListProjectDirectoryTypedAsync(string projectId, string? dependencyId, string path) => Typed<ProjectDirectoryResult>(ListProjectDirectoryAsync(projectId, dependencyId, path));
    public Task<SessionsResult> ListSessionsTypedAsync(string projectId) => Typed<SessionsResult>(ListSessionsAsync(projectId));
    public Task<GitStatusResult> GitStatusTypedAsync(string projectId) => Typed<GitStatusResult>(GitStatusAsync(projectId));
    public Task<GitDiffFileResult> GitDiffTypedAsync(string projectId, string scope, string path) => Typed<GitDiffFileResult>(GitDiffAsync(projectId, scope, path));
    public Task<SessionRecord> CreateSessionAsync(CreateSessionRequest request) => Typed<SessionRecord>(CreateSessionAsync(request.ProjectId, request.Title, request.Model));
    public Task<SessionRecord> RenameSessionTypedAsync(string sessionId, string title) => Typed<SessionRecord>(RenameSessionAsync(sessionId, title));
    public Task<SessionRecord> ArchiveSessionTypedAsync(string sessionId) => Typed<SessionRecord>(ArchiveSessionAsync(sessionId));
    public Task<SessionRecord> SetSessionPinnedTypedAsync(string sessionId, bool pinned) => Typed<SessionRecord>(SetSessionPinnedAsync(sessionId, pinned));
    public Task<SessionRecord> ReopenSessionAsync(string sessionId) => Typed<SessionRecord>(WithUtf8Async([sessionId], values => NativeMethods.suncode_agent_sdk_reopen_session(_handle, values[0])));
    public Task<SessionImagesResult> ListSessionImagesTypedAsync(string sessionId) => Typed<SessionImagesResult>(ListSessionImagesAsync(sessionId));
    public Task<SessionImage> AddSessionImageAsync(AddSessionImageRequest request) => Typed<SessionImage>(WithUtf8Async(
        [request.SessionId, JsonSerializer.Serialize(request, TypedJsonOptions)],
        values => NativeMethods.suncode_agent_sdk_add_session_image(_handle, values[0], values[1])));
    public Task<SessionImageRemoval> RemoveSessionImageTypedAsync(string sessionId, string imageId) => Typed<SessionImageRemoval>(RemoveSessionImageAsync(sessionId, imageId));
    public Task<SessionSnapshot> GetSessionSnapshotAsync(string sessionId) => Typed<SessionSnapshot>(SessionSnapshotAsync(sessionId));
    public Task<SessionUsageResult> GetSessionUsageAsync(string sessionId) => Typed<SessionUsageResult>(SessionUsageAsync(sessionId));
    public Task<ProviderExchangesResult> GetProviderExchangesAsync(string sessionId) => Typed<ProviderExchangesResult>(ListProviderExchangesAsync(sessionId));
    public Task<ProviderExchangeDetails> GetProviderExchangeAsync(string sessionId, string exchangeId) => Typed<ProviderExchangeDetails>(ProviderExchangeAsync(sessionId, exchangeId));
    public Task<CheckpointsResult> GetCheckpointsAsync(string sessionId) => Typed<CheckpointsResult>(ListCheckpointsAsync(sessionId));
    public Task<CheckpointDetails> GetCheckpointManifestAsync(string manifestId) => Typed<CheckpointDetails>(WithUtf8Async([manifestId], values => NativeMethods.suncode_agent_sdk_checkpoint_manifest(_handle, values[0])));
    public Task<RestoreOutcome> RestoreCheckpointTypedAsync(string manifestId, string sessionId) => Typed<RestoreOutcome>(RestoreCheckpointAsync(manifestId, sessionId));
    public Task<TurnResponse> SubmitTurnAsync(SubmitTurnRequest request) => Typed<TurnResponse>(
        request.ImageIds.Count == 0
            ? WithNullableUtf8Async(
                [request.SessionId, request.Input, request.IdempotencyKey, request.Model, request.ReasoningEffort],
                values => NativeMethods.suncode_agent_sdk_submit_turn(_handle, values[0], values[1], values[2], values[3], values[4]))
            : WithNullableUtf8Async(
                [request.SessionId, request.Input, request.IdempotencyKey, request.Model, request.ReasoningEffort, JsonSerializer.Serialize(request.ImageIds)],
                values => NativeMethods.suncode_agent_sdk_submit_turn_with_attachments(_handle, values[0], values[1], values[2], values[3], values[4], values[5])));
    public Task<CancellationOutcome> CancelTurnTypedAsync(string sessionId, string turnId) => Typed<CancellationOutcome>(CancelTurnAsync(sessionId, turnId));
    public Task<TurnResponse> RetryLastTurnTypedAsync(string sessionId) => Typed<TurnResponse>(RetryLastTurnAsync(sessionId));
    public Task<ApprovalRecord> GetApprovalAsync(string approvalId) => Typed<ApprovalRecord>(WithUtf8Async([approvalId], values => NativeMethods.suncode_agent_sdk_get_approval(_handle, values[0])));
    public Task<ApprovalOutcome> ResolveApprovalAsync(ApprovalDecisionRequest request) => Typed<ApprovalOutcome>(ResolveApprovalAsync(request.ApprovalId, request.Decision switch
    {
        ApprovalDecision.Deny => "deny",
        ApprovalDecision.AllowOnce => "allow_once",
        ApprovalDecision.AllowSession => "allow_session",
        _ => throw new ArgumentOutOfRangeException(nameof(request))
    }));
    public Task<QuestionOutcome> ReplyQuestionAsync(ReplyQuestionRequest request) => Typed<QuestionOutcome>(ReplyQuestionAsync(request.RequestId, JsonSerializer.SerializeToNode(request.Answers) as JsonArray ?? throw new ArgumentException("Answers cannot be serialized", nameof(request))));
    public Task<QuestionOutcome> RejectQuestionTypedAsync(string requestId) => Typed<QuestionOutcome>(RejectQuestionAsync(requestId));

    public IDisposable SubscribeTyped(string sessionId, long after, Action<JsonElement> onEvent)
    {
        ArgumentNullException.ThrowIfNull(onEvent);
        return Subscribe(sessionId, after, json =>
        {
            using var document = JsonDocument.Parse(json);
            onEvent(document.RootElement.Clone());
        });
    }
}
