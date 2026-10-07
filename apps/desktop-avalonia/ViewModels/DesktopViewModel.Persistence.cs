using System.Collections.ObjectModel;
using System.Diagnostics;
using System.Globalization;
using System.Text.Json;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Threading;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Sdk;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

public sealed partial class DesktopViewModel : ObservableObject, IDisposable
{
    private async Task LoadProjectsAsync()
    {
        if (_sdk is null) return;
        var result = await _sdk.GetProjectsAsync();
        Projects.Clear();
        foreach (var item in result.Projects)
        {
            Projects.Add(new ProjectItem(item.ProjectId, item.DisplayName, item.CanonicalRoot));
        }
        OnPropertyChanged(nameof(HasProjects));
    }

    private ProjectItem? MatchOrCreateProject(ProjectRecord opened)
    {
        var projectId = opened.ProjectId;
        if (projectId.Length == 0) return null;

        var project = Projects.FirstOrDefault(item => item.ProjectId == projectId);
        if (project is not null) return project;

        var fallback = new ProjectItem(
            projectId,
            opened.DisplayName,
            opened.CanonicalRoot);

        if (fallback.CanonicalRoot.Length == 0) return null;

        Projects.Add(fallback);
        OnPropertyChanged(nameof(HasProjects));
        return fallback;
    }

    private async Task LoadSessionsAsync(string? preferredSessionId = null)
    {
        if (_sdk is null || SelectedProject is null) return;
        var result = await _sdk.ListSessionsAsync(SelectedProject.ProjectId);
        var sessionStates = result.SessionStates;
        Sessions.Clear();
        ArchivedSessions.Clear();
        foreach (var item in result.Sessions)
        {
            var sessionId = item.SessionId;
            var projected = new SessionItem(
                sessionId,
                item.Title ?? string.Empty,
                item.LastActivityAt,
                !string.IsNullOrWhiteSpace(item.PinAt),
                sessionStates.TryGetValue(sessionId, out var state) ? state : string.Empty,
                item.ModelId ?? string.Empty,
                item.ReasoningEffort ?? string.Empty,
                string.Equals(item.Status, "archived", StringComparison.Ordinal));
            if (projected.IsArchived) ArchivedSessions.Add(projected);
            else Sessions.Add(projected);
        }
        RefreshRecentSessionReferences();
        OnPropertyChanged(nameof(HasSessions));
        OnPropertyChanged(nameof(HasArchivedSessions));
        NotifyRunningSessionsChanged();
        var savedState = RestoreRecentContentState();
        await RestoreSavedChildRecentContentsAsync(savedState);
        var savedSessionId = preferredSessionId
            ?? (savedState.CurrentContentKind == "session" ? savedState.CurrentSessionId : savedState.LastSessionId);
        var session = Sessions.FirstOrDefault(item => item.SessionId == savedSessionId)
            ?? ArchivedSessions.FirstOrDefault(item => item.SessionId == savedSessionId)
            ?? Sessions.FirstOrDefault(item => item.SessionId == SelectedSession?.SessionId)
            ?? ArchivedSessions.FirstOrDefault(item => item.SessionId == SelectedSession?.SessionId)
            ?? Sessions.FirstOrDefault();
        if (preferredSessionId is null && savedState.CurrentContentKind == "file" && savedState.CurrentFilePath is { Length: > 0 } filePath && IsSafeRelativePath(filePath))
        {
            var file = new ExplorerNode(Path.GetFileName(filePath), filePath, "file", savedState.CurrentDependencyId);
            RestoringSavedFile = true;
            await SelectExplorerFileAsync(file);
            RestoringSavedFile = false;
            return;
        }
        if (session is not null && session.SessionId != SelectedSession?.SessionId)
        {
            await SelectSessionAsync(session);
        }
        else if (session is not null)
        {
            SelectedSession = session;
        }
        if (session is null) ClearSession();
    }

    private async Task LoadModelsAsync()
    {
        if (_sdk is null) return;
        var selectedId = SelectedModel?.Id;
        var result = await _sdk.GetModelsAsync();
        _selectedModel = null;
        Models.Clear();
        Providers.Clear();
        foreach (var item in result.Models)
        {
            Models.Add(new ModelItem(
                item.Id,
                item.Provider,
                item.ProviderLabel,
                item.Availability,
                item.Capabilities.ReasoningEffort,
                item.Capabilities.Vision,
                item.Capabilities.ComputerUse,
                item.ApiBase,
                item.DefaultApiBase,
                item.ReasoningEfforts
                    .Where(value => !string.IsNullOrWhiteSpace(value))
                    .ToArray(),
                item.Limits.MaxInputTokens,
                item.Limits.AutoCompactTokens,
                item.Limits.MaxOutputTokens));
        }
        foreach (var group in Models.GroupBy(model => model.Provider, StringComparer.Ordinal))
        {
            var first = group.First();
            var configuredByCredential = Credentials.Any(item => item.Provider == group.Key && item.Configured);
            Providers.Add(new ProviderItem(
                group.Key,
                string.IsNullOrWhiteSpace(first.ProviderLabel) ? group.Key : first.ProviderLabel,
                configuredByCredential || group.Any(model => model.Configured),
                first.ApiBase,
                first.DefaultApiBase));
        }
        SelectedModel = Models.FirstOrDefault(item => item.Id == selectedId) ?? Models.FirstOrDefault();
        if (SelectedModel is null)
        {
            SelectedReasoningEffort = null;
            OnPropertyChanged(nameof(SelectedModel));
            OnPropertyChanged(nameof(SelectedModelName));
            OnPropertyChanged(nameof(CanSubmit));
            OnPropertyChanged(nameof(CanCompose));
            OnPropertyChanged(nameof(CanChooseReasoningEffort));
            OnPropertyChanged(nameof(ComposerPlaceholder));
        }
    }

    private async Task LoadAgentsAsync()
    {
        if (_sdk is null) return;
        var result = await _sdk.ListAgentsAsync();
        Agents.Clear();
        foreach (var item in result.Agents)
        {
            Agents.Add(new AgentItem(
                item.Id,
                item.Name,
                item.DisplayName,
                item.Description,
                item.Version,
                item.AllowedTools,
                item.ModelPolicy,
                item.McpPolicy,
                item.CanDelegate,
                item.ToolCallLimit));
        }
    }

    public IEnumerable<ModelItem> ModelsForProvider(string providerId) =>
        Models.Where(model => model.Provider == providerId);

    private async Task LoadCredentialsAsync()
    {
        if (_sdk is null) return;
        var result = await _sdk.GetCredentialsAsync();
        Credentials.Clear();
        foreach (var item in result.Credentials)
        {
            Credentials.Add(new CredentialItem(item.Provider, item.Configured));
        }
        RefreshProviderConfigurationStates();
    }

    private void RefreshProviderConfigurationStates()
    {
        foreach (var provider in Providers.ToArray())
        {
            var configured = Credentials.Any(item => item.Provider == provider.Id && item.Configured);
            if (configured == provider.Configured) continue;
            var index = Providers.IndexOf(provider);
            if (index >= 0) Providers[index] = provider with { Configured = configured };
        }
    }

    private async Task LoadSettingsAsync()
    {
        if (_sdk is null) return;
        var settings = (await _sdk.GetSettingsAsync(new())).Settings;
        var snapshot = new SettingsSnapshot(settings);
        AppSettings.Apply(snapshot);
        DiagnosticLog.Configure(AppSettings.LogLevel, AppSettings.LogDirectory, AppSettings.LogMaxBytes, AppSettings.LogRetention);
        Network.Apply(snapshot);

        foreach (var item in settings)
        {
            var key = item.Key;
            if (item.Value.ValueKind != JsonValueKind.String) continue;
            var value = item.Value.GetString() ?? string.Empty;
            if (key == "default_model") SelectedModel = Models.FirstOrDefault(model => model.Id == value) ?? SelectedModel;
        }
    }

    // Localized item text is computed on read; re-raise it after the shared locale changes.
    private void OnAppLanguageChanged(string locale)
    {
        foreach (var item in LanguageServers.Servers) item.OnPropertyChanged(string.Empty);
        foreach (var item in Mcp.Servers) item.OnPropertyChanged(string.Empty);
        foreach (var item in Messages) item.OnPropertyChanged(string.Empty);
        Remote.RefreshLocalizedText();
        OnPropertyChanged(nameof(ReviewStatusText));
    }

    private async Task LoadSessionControlAsync(string sessionId, long loadVersion)
    {
        if (_sdk is null || SelectedProject is null) return;
        var result = await _sdk.GetSettingsAsync(new(SelectedProject.ProjectId, sessionId));
        if (!IsCurrentSessionLoad(sessionId, loadVersion)) return;
        var setting = result.Settings.FirstOrDefault(item => item.Key == "full_control");
        FullControlEnabled = setting is not null
            && setting.Value.ValueKind is JsonValueKind.True or JsonValueKind.False
            && setting.Value.GetBoolean();
    }

    private async Task LoadCheckpointsAsync(string? requestedSessionId = null, long? loadVersion = null)
    {
        if (_sdk is null || SelectedSession is null) return;
        var sessionId = requestedSessionId ?? SelectedSession.SessionId;
        var result = await _sdk.GetCheckpointsAsync(sessionId);
        if (!IsSessionContextCurrent(sessionId, loadVersion)) return;
        var checkpoints = result.Checkpoints.Select(item =>
        {
            return new SunCode.Desktop.Models.CheckpointItem(
                item.ManifestId,
                item.TurnId ?? string.Empty,
                item.Status,
                Array.Empty<string>());
        });
        Checkpoints.ReplaceAll(checkpoints);
        OnPropertyChanged(nameof(HasCheckpoints));
        NotifyReviewPresentationChanged();
    }

    internal static SessionSnapshotProjection ProjectSnapshot(SessionSnapshot snapshot) =>
        SessionSnapshotProjector.ProjectSnapshot(snapshot);
    internal static string FormatDuration(string startedAt, string completedAt, string state = "") =>
        SessionSnapshotProjector.FormatDuration(startedAt, completedAt, state);
    internal static string FormatCompletionTime(string completedAt) =>
        SessionSnapshotProjector.FormatCompletionTime(completedAt);
    internal static IReadOnlyList<string> MessageImageIds(AgentMessage message) =>
        SessionSnapshotProjector.MessageImageIds(message);
    private static string BoundedPreview(string text) => SessionSnapshotProjector.BoundedPreview(text);
    private static string MessageText(AgentMessage message) => message.Text;
    private static bool IsTerminalTurnState(string state) => TurnStates.IsTerminal(state);

    internal void ApplySnapshot(SessionSnapshotProjection projection)
    {
        _appliedMessageIds.Clear();
        foreach (var message in projection.Messages)
        {
            if (message.MessageId.Length > 0) _appliedMessageIds.Add(message.MessageId);
        }
        DisposeMessages();
        Messages = new BulkObservableCollection<MessageItem>(projection.Messages);
        ToolActivity.ReplaceAll(projection.ToolActivityTurns);
        SyncActiveToolRow();
        Activities.ReplaceAll(projection.Activities);
        ChangedPaths.ReplaceAll(projection.ChangedPaths);
        OnPropertyChanged(nameof(HasChangedPaths));
        OnPropertyChanged(nameof(TurnChangeSummary));
        NotifyReviewPresentationChanged();
        CurrentTodos.ReplaceAll(projection.CurrentTodos);
        PendingApproval = projection.PendingApproval;
        PendingQuestion = projection.PendingQuestion;
        ActiveTurnId = projection.ActiveTurnId;
        ActiveTurnState = projection.ActiveTurnState;
        var activeActivityTurn = ToolActivity.Turns.LastOrDefault(item => item.TurnId == projection.ActiveTurnId);
        UpdateActiveTurnTiming(projection.ActiveTurnId, activeActivityTurn?.StartedAt);
        OnPropertyChanged(nameof(HasMessages));
        NotifyAssistantStreamingChanged();
        OnPropertyChanged(nameof(HasActivities));
        OnPropertyChanged(nameof(HasCurrentTodos));
        OnPropertyChanged(nameof(IsReviewTodosVisible));
        OnPropertyChanged(nameof(LatestActivityText));
    }

    public void ShowToolActivity(string turnId, string toolCallId)
    {
        if (!ToolActivity.TrySelect(turnId, toolCallId)) return;
        ToolActivityVisible = true;
        GitVisible = false;
        ProviderTraceVisible = false;
    }

    private ToolActivityTurnItem EnsureToolActivityTurn(string turnId, string preview = "") =>
        ToolActivity.EnsureTurn(turnId, string.IsNullOrWhiteSpace(preview) ? string.Empty : BoundedPreview(preview));

    private void SyncActiveToolRow()
    {
        foreach (var row in Messages.Where(item => item.IsTool).ToArray()) Messages.Remove(row);
        if (ToolActivity.ActiveTool is not { } active) return;
        var (activeTurn, activeTool) = active;
        Messages.Add(new MessageItem
        {
            Role = "tool",
            Kind = "tool",
            Text = activeTool.DisplayName,
            ContentSequence = Messages.Count + 1,
            TurnId = activeTurn.TurnId,
            ToolCallId = activeTool.ToolCallId,
            ToolName = activeTool.Name,
            ToolState = activeTool.State,
            ToolRequest = activeTool.Request,
            ToolError = activeTool.Error,
            IsWorkingDuration = true,
            DurationText = FormatDuration(activeTurn.StartedAt, string.Empty, activeTurn.State)
        });
    }

    private void UpdateActiveTurnTiming(string turnId, string? startedAt)
    {
        if (string.IsNullOrWhiteSpace(turnId) || IsTerminalTurnState(ActiveTurnState))
        {
            _activeTurnStartedAt = null;
            _activeTurnTimingTurnId = string.Empty;
            _conversationDurationTimer.Stop();
        }
        else if (_activeTurnStartedAt is null || !string.Equals(_activeTurnTimingTurnId, turnId, StringComparison.Ordinal))
        {
            _activeTurnStartedAt = DateTimeOffset.TryParse(startedAt, out var parsed) ? parsed : DateTimeOffset.Now;
            _activeTurnTimingTurnId = turnId;
            _conversationDurationTimer.Start();
        }
        else if (DateTimeOffset.TryParse(startedAt, out var updated))
        {
            _activeTurnStartedAt = updated;
        }
        OnPropertyChanged(nameof(ActiveTurnDurationText));
        RefreshConversationDuration();
    }

    private void ConversationDurationTick(object? sender, EventArgs e)
    {
        OnPropertyChanged(nameof(ActiveTurnDurationText));
        RefreshConversationDuration();
    }

    private void RefreshConversationDuration()
    {
        var row = Messages.FirstOrDefault(item => item.IsWorkingDuration);
        if (row is not null) row.DurationText = ActiveTurnDurationText;
    }
}
