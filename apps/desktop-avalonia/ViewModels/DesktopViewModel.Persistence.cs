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

    private void OnNativeEvent(string sessionId, AgentEvent value) => Dispatcher.UIThread.Post(() =>
    {
        if (_disposed || SelectedSession?.SessionId != sessionId)
        {
            return;
        }
        if (value.EventType == "resync.required")
        {
            LogSession("event", sessionId, "resync.required reload_begin");
            RunInBackground(() => ReloadCurrentSessionAsync(sessionId), "session.reload");
            return;
        }
        ApplyEvent(value, true);
    });

    private async Task ReloadCurrentSessionAsync(string sessionId)
    {
        if (SelectedSession?.SessionId != sessionId) return;
        _loadedSessionId = null;
        CloseSubscription();
        await SelectSessionAsync(SelectedSession);
    }

    internal void ApplyEvent(AgentEvent value, bool live)
    {
        var type = value.EventType;
        var payload = value.Payload;
        var text = EventText(type, payload);
        if (type.StartsWith("provider.exchange.", StringComparison.Ordinal)) ProviderTraffic.Apply(value);

        if (payload.Usage is { } usage && type.StartsWith("provider.exchange.", StringComparison.Ordinal))
            UpdateContextUsage(usage);

        if (type == "assistant.delta")
        {
            var turnId = payload.TurnId ?? string.Empty;
            var assistant = Messages.LastOrDefault(message =>
                message.TurnId == turnId && message.Role == "assistant" && message.Streaming);
            var delta = payload.Text ?? string.Empty;
            if (assistant is null)
            {
                if (delta.Length > 0)
                {
                    var activityTurn = EnsureToolActivityTurn(turnId);
                    Messages.Add(new MessageItem
                    {
                        Role = "assistant",
                        Text = delta,
                        ContentSequence = Messages.Count + 1,
                        TurnId = turnId,
                        TurnSequence = activityTurn.Sequence,
                        TurnPreview = activityTurn.Preview,
                        Streaming = true,
                        IsProcess = true,
                        CanBeFinalAssistant = false
                    });
                }
            }
            else if (delta.Length > 0)
            {
                assistant.Text += delta;
            }
            if (delta.Length > 0)
            {
                // The collection itself does not change when a streaming
                // assistant row receives another text delta. Notify the chat
                // surface so it can keep the viewport at the latest content.
                OnPropertyChanged(nameof(Messages));
                NotifyAssistantStreamingChanged();
            }
        }
        else if (type is "message.user" or "message.assistant")
        {
            var turnId = payload.TurnId ?? string.Empty;
            var messageId = payload.MessageId ?? string.Empty;
            var message = payload.Message ?? EmptyMessage;
            var canBeFinalAssistant = type == "message.assistant" && (message.ToolCalls?.Count ?? 0) == 0;
            var changed = false;
            var streaming = type == "message.assistant"
                ? Messages.LastOrDefault(message =>
                    message.TurnId == turnId && message.Role == "assistant" && message.Streaming)
                : null;
            if (messageId.Length > 0 && !_appliedMessageIds.Add(messageId))
            {
                DiagnosticLog.Debug("session.message", $"duplicate ignored type={type} message={messageId} turn={turnId}");
            }
            else if (type == "message.user")
            {
                EnsureToolActivityTurn(turnId, text);
                Messages.Add(new MessageItem
                {
                    MessageId = messageId,
                    Role = "user",
                    Text = text,
                    ContentSequence = Messages.Count + 1,
                    TurnId = turnId,
                    IsVisible = true,
                    Attachments = PendingMessageAttachments(message)
                });
                changed = true;
            }
            else if (!string.IsNullOrWhiteSpace(text))
            {
                if (streaming is not null)
                {
                    streaming.MessageId = messageId;
                    streaming.Text = text;
                    streaming.Streaming = false;
                    streaming.CanBeFinalAssistant = canBeFinalAssistant;
                }
                else
                {
                    var activityTurn = EnsureToolActivityTurn(turnId);
                    foreach (var previous in Messages.Where(item => item.TurnId == turnId && item.IsAssistant))
                        previous.IsFinalAssistant = false;
                    Messages.Add(new MessageItem
                    {
                        MessageId = messageId,
                        Role = "assistant",
                        Text = text,
                        ContentSequence = Messages.Count + 1,
                        TurnId = turnId,
                        CanBeFinalAssistant = canBeFinalAssistant,
                        IsFinalAssistant = false,
                        TurnSequence = activityTurn.Sequence,
                        TurnPreview = activityTurn.Preview
                    });
                }
                changed = true;
            }
            else if (streaming is not null)
            {
                streaming.MessageId = messageId;
                streaming.Streaming = false;
                streaming.CanBeFinalAssistant = canBeFinalAssistant;
                changed = true;
            }
            if (changed)
            {
                OnPropertyChanged(nameof(Messages));
                OnPropertyChanged(nameof(HasMessages));
                NotifyAssistantStreamingChanged();
            }
        }
        else if (type is "tool.requested" or "tool.state" or "tool.result" or "tool.output")
        {
            ApplyToolEvent(payload, type);
            Activities.Add(new ActivityItem(type, text, Activities.Count + 1, payload.State ?? string.Empty, payload.Name ?? string.Empty));
            OnPropertyChanged(nameof(HasActivities));
            OnPropertyChanged(nameof(LatestActivityText));
        }
        else if (type == "todo.updated")
        {
            CurrentTodos.ReplaceAll((payload.Todos ?? [])
                .Select(TodoItem.FromSdk)
                .Where(item => item is not null)
                .Select(item => item!)
                .ToArray());
            OnPropertyChanged(nameof(HasCurrentTodos));
            OnPropertyChanged(nameof(IsReviewTodosVisible));
            Activities.Add(new ActivityItem(type, text, Activities.Count + 1, payload.State ?? string.Empty, "todowrite"));
            OnPropertyChanged(nameof(HasActivities));
            OnPropertyChanged(nameof(LatestActivityText));
        }
        else if (type == "context.compacted")
        {
            Messages.Add(new MessageItem
            {
                Role = "assistant",
                Kind = "context.compacted",
                Text = EventText(type, payload),
                ContentSequence = Messages.Count + 1,
                TurnId = payload.TurnId ?? string.Empty,
                IsProcess = true
            });
            OnPropertyChanged(nameof(Messages));
            OnPropertyChanged(nameof(HasMessages));
            NotifyAssistantStreamingChanged();
        }
        else if (!type.StartsWith("provider.exchange.", StringComparison.Ordinal))
        {
            Activities.Add(new ActivityItem(type, text, Activities.Count + 1, payload.State ?? string.Empty, payload.Operation ?? string.Empty));
            OnPropertyChanged(nameof(HasActivities));
            OnPropertyChanged(nameof(LatestActivityText));
        }

        var pathAdded = false;
        foreach (var path in new[] { payload.Path ?? string.Empty })
        {
            if (path.Length > 0 && !ChangedPaths.Contains(path))
            {
                ChangedPaths.Add(path);
                OnPropertyChanged(nameof(HasChangedPaths));
                OnPropertyChanged(nameof(TurnChangeSummary));
                NotifyReviewPresentationChanged();
                pathAdded = true;
            }
        }
        if (type == "approval.requested") PendingApproval = ApprovalItem.FromSdk(payload);
        if (type == "approval.resolved")
        {
            if (payload.Decision == "allow_session") FullControlEnabled = true;
            PendingApproval = null;
        }
        if (type == "question.asked") PendingQuestion = PendingQuestionItem.FromSdk(payload);
        if (type is "question.replied" or "question.rejected") PendingQuestion = null;
        if (live && type is "approval.requested" or "approval.resolved" or "question.asked" or "question.replied"
                or "question.rejected")
            SyncSelectedSessionAgentStateFromReview();
        if (type == "turn.state")
        {
            var state = payload.State ?? string.Empty;
            var turnId = payload.TurnId ?? string.Empty;
            if (!string.IsNullOrWhiteSpace(turnId)) LastTurnId = turnId;
            if (state == "admitted")
            {
                CurrentTodos.Clear();
                OnPropertyChanged(nameof(HasCurrentTodos));
                OnPropertyChanged(nameof(IsReviewTodosVisible));
            }
            ActiveTurnId = IsTerminalTurnState(state) ? string.Empty : turnId;
            ActiveTurnState = state;
            var activityTurn = EnsureToolActivityTurn(turnId);
            var occurredAt = value.OccurredAt;
            activityTurn.SetTiming(
                payload.StartedAt,
                IsTerminalTurnState(state) ? (payload.CompletedAt is { Length: > 0 } completed ? completed : occurredAt) : null);
            if (string.IsNullOrWhiteSpace(activityTurn.StartedAt) && state == "admitted")
                activityTurn.SetTiming(occurredAt, null);
            activityTurn.Update(state);
            UpdateActiveTurnTiming(turnId, activityTurn.StartedAt);
            if (!IsTerminalTurnState(state)) activityTurn.IsExpanded = true;
            if (IsTerminalTurnState(state))
            {
                var finalAssistant = Messages.LastOrDefault(item => item.TurnId == turnId && item.IsAssistant && item.CanBeFinalAssistant);
                if (finalAssistant is not null)
                {
                    finalAssistant.DurationText = FormatDuration(activityTurn.StartedAt, activityTurn.CompletedAt, state);
                    finalAssistant.IsFinalAssistant = true;
                    finalAssistant.CompletionTimeText = FormatCompletionTime(activityTurn.CompletedAt);
                }
            }
            SyncActiveToolRow();
            ToolActivity.NotifyTurnsChanged();
        }

        if (live && type == "turn.state") SyncSelectedSessionAgentStateFromReview();
        if (live && type.StartsWith("checkpoint.", StringComparison.Ordinal)) RunInBackground(() => LoadCheckpointsAsync(), "checkpoints.load");
        if (live && type.StartsWith("provider.exchange.", StringComparison.Ordinal) && ProviderTraceVisible) RunInBackground(ProviderTrace.RefreshAsync, "provider_trace.refresh");
        if (live && (type == "tool.result" || (type == "turn.state" && IsTerminalTurnState(payload.State ?? string.Empty)))) RunInBackground(LoadChildSessionsAsync, "child_sessions.load");
        if (live && (type.StartsWith("checkpoint.", StringComparison.Ordinal) || pathAdded)) RunInBackground(Git.RefreshAsync, "git.refresh");
    }

    private void ApplyToolEvent(AgentEventPayload payload, string eventType)
    {
        var turnId = payload.TurnId ?? string.Empty;
        var toolCallId = payload.ToolCallId ?? string.Empty;
        if (turnId.Length == 0 || toolCallId.Length == 0) return;
        var turn = EnsureToolActivityTurn(turnId);
        var existing = turn.Tools.FirstOrDefault(tool => tool.ToolCallId == toolCallId);
        var state = eventType == "tool.state"
            ? payload.State ?? string.Empty
            : existing?.State ?? "requested";
        var name = payload.Name ?? string.Empty;
        if (name.Length == 0) name = existing?.Name ?? "tool";
        var request = eventType == "tool.requested"
            ? Pretty(payload.Arguments)
            : existing?.Request ?? string.Empty;
        var result = eventType == "tool.result"
            ? Pretty(payload.Result)
            : existing?.Result ?? string.Empty;
        var output = eventType == "tool.output"
            ? AppendBoundedOutput(existing?.Output ?? string.Empty, DecodeOutputChunk(payload))
            : existing?.Output ?? string.Empty;
        var error = eventType == "tool.state"
            ? payload.Reason ?? string.Empty
            : existing?.Error ?? string.Empty;
        ToolActivity.Upsert(turn, toolCallId, name, state, request, result, output, error);
        SyncActiveToolRow();
    }

    private static string DecodeOutputChunk(AgentEventPayload payload)
    {
        var encoded = payload.ChunkBase64 ?? string.Empty;
        if (encoded.Length == 0) return string.Empty;
        try { return System.Text.Encoding.UTF8.GetString(Convert.FromBase64String(encoded)); }
        catch (FormatException) { return string.Empty; }
    }

    private static string AppendBoundedOutput(string existing, string chunk)
    {
        const int maxCharacters = 256 * 1024;
        if (existing.Length >= maxCharacters || chunk.Length == 0) return existing;
        var remaining = maxCharacters - existing.Length;
        return existing + (chunk.Length <= remaining ? chunk : chunk[..remaining]);
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

    private static string EventText(string type, AgentEventPayload payload)
    {
        var messageText = payload.Message is { } message ? MessageText(message) : string.Empty;
        if (type is "message.user" or "message.assistant" or "message.tool" || !string.IsNullOrEmpty(messageText)) return messageText;
        return type switch
        {
            "approval.requested" => $"Approval required for {payload.Operation}",
            "question.asked" => "Waiting for an answer",
            "question.replied" => "Question answered",
            "question.rejected" => "Question skipped",
            "todo.updated" => "Todo list updated",
            "context.compacted" => $"Context compacted · retained {payload.RetainedTokens} tokens",
            "checkpoint.captured" => $"Checkpoint captured for {payload.Path}",
            "checkpoint.restore_failed" => "Undo stopped because a file changed outside SunCode",
            "turn.state" => $"Turn {payload.State}",
            "assistant.delta" => payload.Text ?? string.Empty,
            "tool.output" => $"Command output · {payload.Stream}",
            _ => type
        };
    }

    private static readonly AgentMessage EmptyMessage = new(string.Empty, [], [], null);

    private IReadOnlyList<ComposerAttachment> PendingMessageAttachments(AgentMessage message)
    {
        var imageIds = (message.Content ?? [])
            .Where(part => part.Kind == "image_ref")
            .Select(part => part.Text)
            .ToHashSet(StringComparer.Ordinal);
        if (imageIds.Count == 0) return [];
        var attachments = _submittedAttachments.Where(item => imageIds.Contains(item.ImageId)).ToArray();
        _submittedAttachments = _submittedAttachments.Where(item => !imageIds.Contains(item.ImageId)).ToArray();
        foreach (var attachment in attachments)
        {
            // Transfer ownership from the composer to the live user message without
            // disposing the shared preview bitmap.
            ComposerAttachments.Remove(attachment);
        }
        return attachments;
    }
}
