using System.Text.Json;
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

    private static T Deserialize<T>(JsonElement body)
        => body.Deserialize<T>(TypedJsonOptions)
           ?? throw new SdkException("invalid_response", $"Agent returned an empty {typeof(T).Name} response");

    private static async Task<T> Typed<T>(Task<JsonElement> operation)
        => Deserialize<T>(await operation.ConfigureAwait(false));

    public static Task<VersionResult> GetVersionAsync() => Typed<VersionResult>(RawVersionAsync());
    public Task<HealthResult> GetHealthAsync() => Typed<HealthResult>(RawHealthAsync());
    public Task<DiagnosticsResult> GetDiagnosticsAsync() => Typed<DiagnosticsResult>(RawDiagnosticsAsync());
    public Task<ModelsResult> GetModelsAsync() => Typed<ModelsResult>(RawListModelsAsync());
    public Task<CredentialsResult> GetCredentialsAsync() => Typed<CredentialsResult>(RawListCredentialsAsync());
    public Task<ProjectsResult> GetProjectsAsync() => Typed<ProjectsResult>(RawListProjectsAsync());

    public Task<BrowserRuntimeInfo> GetBrowserRuntimeInfoAsync(string? projectId = null) =>
        Typed<BrowserRuntimeInfo>(RawBrowserRuntimeInfoAsync(projectId));

    public Task<BrowserRuntimeInfo> SetBrowserUseEnabledAsync(bool enabled) =>
        Typed<BrowserRuntimeInfo>(RawSetBrowserUseEnabledAsync(enabled));

    public Task<BrowserRuntimeInfo> VerifyBrowserRuntimeAsync(string? projectId = null) =>
        Typed<BrowserRuntimeInfo>(RawVerifyBrowserRuntimeAsync(projectId));

    public Task<BrowserRuntimeInfo> StartBrowserProjectAsync(string projectId) =>
        Typed<BrowserRuntimeInfo>(RawStartBrowserProjectAsync(projectId));

    public Task<BrowserRuntimeInfo> TakeBrowserControlAsync(string projectId) =>
        Typed<BrowserRuntimeInfo>(RawTakeBrowserControlAsync(projectId));

    public Task<BrowserRuntimeInfo> ReturnBrowserControlAsync(string projectId) =>
        Typed<BrowserRuntimeInfo>(RawReturnBrowserControlAsync(projectId));

    public Task<BrowserRuntimeInfo> RestartBrowserRuntimeAsync(string projectId) =>
        Typed<BrowserRuntimeInfo>(RawRestartBrowserRuntimeAsync(projectId));

    public Task<BrowserRuntimeInfo> StopBrowserRuntimeAsync(string projectId) =>
        Typed<BrowserRuntimeInfo>(RawStopBrowserRuntimeAsync(projectId));

    public Task<BrowserProfileClearResult> ClearBrowserProfileAsync(string projectId) =>
        Typed<BrowserProfileClearResult>(RawClearBrowserProfileAsync(projectId));

    public Task<McpServersResult> GetMcpServersAsync(string? projectId = null) =>
        Typed<McpServersResult>(RawListMcpServersAsync(projectId));

    public Task<McpServer> CreateMcpServerAsync(CreateMcpServerRequest request) =>
        Typed<McpServer>(RawCreateMcpServerAsync(
            request.ProjectId,
            request.IdempotencyKey,
            JsonSerializer.Serialize(request.Server, TypedJsonOptions)));

    public Task<McpServer> UpdateMcpServerAsync(UpdateMcpServerRequest request) =>
        Typed<McpServer>(RawUpdateMcpServerAsync(
            request.ProjectId,
            request.ServerId,
            request.ExpectedRevision,
            request.IdempotencyKey,
            JsonSerializer.Serialize(request.Server, TypedJsonOptions)));

    public Task<McpServer> SetMcpServerEnabledAsync(SetMcpServerEnabledRequest request) =>
        Typed<McpServer>(RawSetMcpServerEnabledAsync(
            request.ProjectId,
            request.ServerId,
            request.ExpectedRevision,
            request.IdempotencyKey,
            request.Enabled));

    public Task<McpServerDeleteResult> DeleteMcpServerAsync(DeleteMcpServerRequest request) =>
        Typed<McpServerDeleteResult>(RawDeleteMcpServerAsync(
            request.ServerId,
            request.ExpectedRevision,
            request.IdempotencyKey));

    public Task<McpServer> RetryMcpServerAsync(string projectId, string serverId) =>
        Typed<McpServer>(RawRetryMcpServerAsync(projectId, serverId));

    public Task<McpLoadProgress> StartMcpProjectAsync(string projectId) =>
        Typed<McpLoadProgress>(RawStartMcpProjectAsync(projectId));

    public Task<McpLoadProgress> GetMcpLoadProgressAsync(string projectId) =>
        Typed<McpLoadProgress>(RawMcpLoadProgressAsync(projectId));

    public Task<LanguageServersResult> GetLanguageServersAsync(string? projectId = null) =>
        Typed<LanguageServersResult>(RawListLanguageServersAsync(projectId));

    public Task<LanguageServer> CreateLanguageServerAsync(CreateLanguageServerRequest request) =>
        Typed<LanguageServer>(RawCreateLanguageServerAsync(
            request.ProjectId,
            request.IdempotencyKey,
            JsonSerializer.Serialize(request.Server, TypedJsonOptions)));

    public Task<LanguageServer> UpdateLanguageServerAsync(UpdateLanguageServerRequest request) =>
        Typed<LanguageServer>(RawUpdateLanguageServerAsync(
            request.ProjectId,
            request.LanguageServerId,
            request.ExpectedRevision,
            request.IdempotencyKey,
            JsonSerializer.Serialize(request.Server, TypedJsonOptions)));

    public Task<LanguageServer> SetLanguageServerEnabledAsync(SetLanguageServerEnabledRequest request) =>
        Typed<LanguageServer>(RawSetLanguageServerEnabledAsync(
            request.ProjectId,
            request.LanguageServerId,
            request.ExpectedRevision,
            request.IdempotencyKey,
            request.Enabled));

    public Task<LanguageServerDeleteResult> DeleteLanguageServerAsync(DeleteLanguageServerRequest request) =>
        Typed<LanguageServerDeleteResult>(RawDeleteLanguageServerAsync(
            request.LanguageServerId,
            request.ExpectedRevision,
            request.IdempotencyKey));

    public Task<LanguageServer> RetryLanguageServerAsync(string projectId, string languageServerId) =>
        Typed<LanguageServer>(RawRetryLanguageServerAsync(projectId, languageServerId));

    public Task<LanguageServerProjectResult> StartLanguageServerProjectAsync(string projectId) =>
        Typed<LanguageServerProjectResult>(RawStartLanguageServerProjectAsync(projectId));

    public Task<SettingsResult> GetSettingsAsync(SettingScope scope) =>
        Typed<SettingsResult>(RawListSettingsAsync(scope.ProjectId, scope.SessionId));

    public Task<SettingUpdate> SetSettingAsync(SetSettingRequest request) =>
        Typed<SettingUpdate>(RawSetSettingAsync(
            request.Scope,
            request.ProjectId,
            request.SessionId,
            request.Key,
            request.Value));

    public Task<ProxyConfigurationResult> SetProxyConfigurationAsync(ProxyConfigurationRequest request) =>
        Typed<ProxyConfigurationResult>(RawSetProxyConfigurationAsync(
            JsonSerializer.Serialize(request, TypedJsonOptions)));

    public Task<CredentialUpdate> SetCredentialAsync(SetCredentialRequest request) =>
        Typed<CredentialUpdate>(RawSetCredentialAsync(request.Provider, request.ApiKey));

    public Task<CredentialUpdate> RemoveCredentialAsync(string provider) =>
        Typed<CredentialUpdate>(RawRemoveCredentialAsync(provider));

    public Task<ProviderEndpointUpdate> SetProviderEndpointAsync(ProviderEndpointRequest request) =>
        Typed<ProviderEndpointUpdate>(RawSetProviderEndpointAsync(request.Provider, request.Endpoint));

    public Task<ProjectRecord> OpenProjectAsync(OpenProjectRequest request) =>
        Typed<ProjectRecord>(RawOpenProjectAsync(request.Path, request.DisplayName));

    public Task<ProjectRecord> SelectProjectAsync(string projectId) =>
        Typed<ProjectRecord>(RawSelectProjectAsync(projectId));

    public Task<ProjectDependenciesResult> ListProjectDependenciesAsync(string projectId) =>
        Typed<ProjectDependenciesResult>(RawListProjectDependenciesAsync(projectId));

    public Task<ProjectDependency> AddProjectDependencyAsync(string projectId, string path) =>
        Typed<ProjectDependency>(RawAddProjectDependencyAsync(projectId, path));

    public Task<DependencyRemoval> RemoveProjectDependencyAsync(string projectId, string dependencyId) =>
        Typed<DependencyRemoval>(RawRemoveProjectDependencyAsync(projectId, dependencyId));

    public Task<ProjectDirectoryResult> ListProjectDirectoryAsync(
        string projectId,
        string? dependencyId,
        string path) =>
        Typed<ProjectDirectoryResult>(RawListProjectDirectoryAsync(projectId, dependencyId, path));

    public Task<ProjectFileResult> ReadProjectFileAsync(
        string projectId,
        string? dependencyId,
        string path) =>
        Typed<ProjectFileResult>(RawReadProjectFileAsync(projectId, dependencyId, path));

    public Task<GitStatusResult> GitStatusAsync(string projectId) =>
        Typed<GitStatusResult>(RawGitStatusAsync(projectId));

    public Task<GitDiffFileResult> GitDiffAsync(string projectId, string scope, string path) =>
        Typed<GitDiffFileResult>(RawGitDiffAsync(projectId, scope, path));

    public Task<SessionsResult> ListSessionsAsync(string projectId) =>
        Typed<SessionsResult>(RawListSessionsAsync(projectId));

    public Task<AgentsResult> ListAgentsAsync() =>
        Typed<AgentsResult>(RawListAgentsAsync());

    public Task<ChildSessionsResult> ListChildSessionsAsync(string parentSessionId) =>
        Typed<ChildSessionsResult>(RawListChildSessionsAsync(parentSessionId));

    public Task<SessionRecord> CreateSessionAsync(CreateSessionRequest request) =>
        Typed<SessionRecord>(RawCreateSessionAsync(request.ProjectId, request.Title, request.Model));

    public Task<SessionRecord> RenameSessionAsync(string sessionId, string title) =>
        Typed<SessionRecord>(RawRenameSessionAsync(sessionId, title));

    public Task<SessionRecord> ArchiveSessionAsync(string sessionId) =>
        Typed<SessionRecord>(RawArchiveSessionAsync(sessionId));

    public Task<SessionRecord> SetSessionPinnedAsync(string sessionId, bool pinned) =>
        Typed<SessionRecord>(RawSetSessionPinnedAsync(sessionId, pinned));

    public Task<SessionRecord> ReopenSessionAsync(string sessionId) =>
        Typed<SessionRecord>(RawReopenSessionAsync(sessionId));

    public Task<SessionImagesResult> ListSessionImagesAsync(string sessionId) =>
        Typed<SessionImagesResult>(RawListSessionImagesAsync(sessionId));

    public Task<SessionImage> AddSessionImageAsync(AddSessionImageRequest request) =>
        Typed<SessionImage>(RawAddSessionImageAsync(
            request.SessionId,
            JsonSerializer.Serialize(request, TypedJsonOptions)));

    public Task<SessionImageRemoval> RemoveSessionImageAsync(string sessionId, string imageId) =>
        Typed<SessionImageRemoval>(RawRemoveSessionImageAsync(sessionId, imageId));

    public Task<SessionSnapshot> GetSessionSnapshotAsync(string sessionId) =>
        Typed<SessionSnapshot>(RawSessionSnapshotAsync(sessionId));

    public Task<SessionUsageResult> GetSessionUsageAsync(string sessionId) =>
        Typed<SessionUsageResult>(RawSessionUsageAsync(sessionId));

    public Task<ProviderExchangesResult> GetProviderExchangesAsync(string sessionId) =>
        Typed<ProviderExchangesResult>(RawListProviderExchangesAsync(sessionId));

    public Task<ProviderExchangeDetails> GetProviderExchangeAsync(string sessionId, string exchangeId) =>
        Typed<ProviderExchangeDetails>(RawProviderExchangeAsync(sessionId, exchangeId));

    public Task<CheckpointsResult> GetCheckpointsAsync(string sessionId) =>
        Typed<CheckpointsResult>(RawListCheckpointsAsync(sessionId));

    public Task<CheckpointDetails> GetCheckpointManifestAsync(string manifestId) =>
        Typed<CheckpointDetails>(RawCheckpointManifestAsync(manifestId));

    public Task<RestoreOutcome> RestoreCheckpointAsync(string manifestId, string sessionId) =>
        Typed<RestoreOutcome>(RawRestoreCheckpointAsync(manifestId, sessionId));

    public Task<TurnResponse> SubmitTurnAsync(SubmitTurnRequest request) =>
        Typed<TurnResponse>(
            request.ImageIds.Count == 0
                ? RawSubmitTurnAsync(request)
                : RawSubmitTurnWithAttachmentsAsync(request));

    public Task<CancellationOutcome> CancelTurnAsync(string sessionId, string turnId) =>
        Typed<CancellationOutcome>(RawCancelTurnAsync(sessionId, turnId));

    public Task<TurnResponse> RetryLastTurnAsync(string sessionId) =>
        Typed<TurnResponse>(RawRetryLastTurnAsync(sessionId));

    public Task<ApprovalRecord> GetApprovalAsync(string approvalId) =>
        Typed<ApprovalRecord>(RawGetApprovalAsync(approvalId));

    public Task<ApprovalOutcome> ResolveApprovalAsync(ApprovalDecisionRequest request) =>
        Typed<ApprovalOutcome>(RawResolveApprovalAsync(
            request.ApprovalId,
            request.Decision switch
            {
                ApprovalDecision.Deny => "deny",
                ApprovalDecision.AllowOnce => "allow_once",
                ApprovalDecision.AllowSession => "allow_session",
                _ => throw new ArgumentOutOfRangeException(nameof(request))
            }));

    public Task<QuestionOutcome> ReplyQuestionAsync(ReplyQuestionRequest request) =>
        Typed<QuestionOutcome>(RawReplyQuestionAsync(
            request.RequestId,
            JsonSerializer.Serialize(request.Answers, TypedJsonOptions)));

    public Task<QuestionOutcome> RejectQuestionAsync(string requestId) =>
        Typed<QuestionOutcome>(RawRejectQuestionAsync(requestId));

    public IDisposable SubscribeTyped(string sessionId, long after, Action<AgentEvent> onEvent)
    {
        ArgumentNullException.ThrowIfNull(onEvent);
        return RawSubscribe(sessionId, after, json =>
        {
            using var document = JsonDocument.Parse(json);
            onEvent(document.RootElement.Deserialize<AgentEvent>(TypedJsonOptions)
                ?? throw new SdkException("invalid_event", "Agent returned an empty event"));
        });
    }
}
