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

    private async Task LoadProjectDependenciesAsync()
    {
        ProjectDependencies.Clear();
        if (_sdk is null || SelectedProject is null)
        {
            OnPropertyChanged(nameof(HasProjectDependencies));
            return;
        }
        var result = await _sdk.ListProjectDependenciesAsync(SelectedProject.ProjectId);
        foreach (var item in result.Dependencies)
        {
            ProjectDependencies.Add(new ProjectDependencyItem(
                item.DependencyId,
                item.DisplayName));
        }
        OnPropertyChanged(nameof(HasProjectDependencies));
    }

    private void ResetExplorerRoots()
    {
        ExplorerRoots.Clear();
        if (SelectedProject is null) return;
        ExplorerRoots.Add(new ExplorerNode(
            SelectedProject.DisplayName,
            ".",
            "directory",
            isRoot: true));
        var dependencyGroup = new ExplorerNode(
            "Dependencies",
            ".",
            "group",
            isRoot: true,
            isGroup: true);
        foreach (var dependency in ProjectDependencies)
        {
            dependencyGroup.Children.Add(new ExplorerNode(
                dependency.DisplayName,
                ".",
                "directory",
                dependency.DependencyId,
                isRoot: true,
                isDependency: true));
        }
        dependencyGroup.IsLoaded = true;
        ExplorerRoots.Add(dependencyGroup);
    }

    private async Task LoadSessionsAsync(string? preferredSessionId = null)
    {
        if (_sdk is null || SelectedProject is null) return;
        var result = await _sdk.ListSessionsAsync(SelectedProject.ProjectId);
        var sessionStates = result.SessionStates;
        Sessions.Clear();
        foreach (var item in result.Sessions)
        {
            var sessionId = item.SessionId;
            Sessions.Add(new SessionItem(
                sessionId,
                item.Title ?? string.Empty,
                item.LastActivityAt,
                !string.IsNullOrWhiteSpace(item.PinAt),
                sessionStates.TryGetValue(sessionId, out var state) ? state : string.Empty,
                item.ModelId ?? string.Empty,
                item.ReasoningEffort ?? string.Empty));
        }
        RefreshRecentSessionReferences();
        OnPropertyChanged(nameof(HasSessions));
        var savedState = RestoreRecentContentState();
        await RestoreSavedChildRecentContentsAsync(savedState);
        var savedSessionId = preferredSessionId
            ?? (savedState.CurrentContentKind == "session" ? savedState.CurrentSessionId : savedState.LastSessionId);
        var session = Sessions.FirstOrDefault(item => item.SessionId == savedSessionId)
            ?? Sessions.FirstOrDefault(item => item.SessionId == SelectedSession?.SessionId)
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
                item.ApiBase,
                item.DefaultApiBase,
                item.ReasoningEfforts
                    .Where(value => !string.IsNullOrWhiteSpace(value))
                    .ToArray()));
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
        string StringSetting(string key, string fallback)
        {
            var setting = settings.FirstOrDefault(item => item.Key == key);
            return setting is not null && setting.Value.ValueKind == JsonValueKind.String
                ? setting.Value.GetString() ?? fallback
                : fallback;
        }
        long LongSetting(string key, long fallback)
        {
            var setting = settings.FirstOrDefault(item => item.Key == key);
            return setting is not null && setting.Value.TryGetInt64(out var parsed)
                ? parsed
                : fallback;
        }
        bool BoolSetting(string key, bool fallback)
        {
            var setting = settings.FirstOrDefault(item => item.Key == key);
            return setting is not null && setting.Value.ValueKind is JsonValueKind.True or JsonValueKind.False
                ? setting.Value.GetBoolean()
                : fallback;
        }
        string[] StringArraySetting(string key)
        {
            var setting = settings.FirstOrDefault(item => item.Key == key);
            return setting is not null && setting.Value.ValueKind == JsonValueKind.Array
                ? setting.Value.EnumerateArray()
                    .Where(value => value.ValueKind == JsonValueKind.String)
                    .Select(value => value.GetString() ?? string.Empty)
                    .Where(value => value.Length > 0)
                    .ToArray()
                : Array.Empty<string>();
        }
        var retention = LongSetting("log_retention", 5);
        var configuredLevel = StringSetting("log_level", "INFO").Trim().ToUpperInvariant();
        LogLevel = configuredLevel is "TRACE" or "DEBUG" or "INFO" or "WARN" or "ERROR" or "OFF"
            ? configuredLevel
            : "INFO";
        LogDirectory = StringSetting("log_directory", string.Empty);
        ImageDirectory = StringSetting("image_directory", string.Empty);
        var maxBytes = LongSetting("log_max_bytes", 10 * 1024 * 1024);
        LogMaxBytes = maxBytes >= 1024 ? maxBytes : 10 * 1024 * 1024;
        LogRetention = retention is >= 0 and <= 100 ? (int)retention : 5;
        VerifyHttpsCertificates = BoolSetting("verify_https_certificates", true);
        UseSystemCertificates = BoolSetting("use_system_certificates", true);
        CertificatePath = StringSetting("certificate_path", string.Empty);
        var proxyMode = StringSetting("proxy_mode", "system");
        ProxyMode = proxyMode is "no_proxy" or "system" or "custom" ? proxyMode : "system";
        ProxyUrl = StringSetting("proxy_url", string.Empty);
        ProxyUsername = StringSetting("proxy_username", string.Empty);
        ProxyPasswordConfigured = BoolSetting("proxy_password_configured", false);
        ProxyBypassRules = string.Join(Environment.NewLine, StringArraySetting("proxy_bypass"));
        DiagnosticLog.Configure(
            LogLevel,
            LogDirectory,
            LogMaxBytes,
            LogRetention);

        foreach (var item in settings)
        {
            var key = item.Key;
            if (item.Value.ValueKind != JsonValueKind.String) continue;
            var value = item.Value.GetString() ?? string.Empty;
            if (key == "theme_mode" && value is "dark" or "light") SetTheme(value);
            if (key == "default_model") SelectedModel = Models.FirstOrDefault(model => model.Id == value) ?? SelectedModel;
        }
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

    private async Task LoadSessionUsageAsync(string? requestedSessionId = null, long? loadVersion = null)
    {
        if (_sdk is null || SelectedSession is null) return;
        var sessionId = requestedSessionId ?? SelectedSession.SessionId;
        var result = await _sdk.GetSessionUsageAsync(sessionId);
        if (!IsSessionContextCurrent(sessionId, loadVersion)) return;
        SessionTotalTokens = (long)result.TotalTokens;
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

    internal static SessionSnapshotProjection ProjectSnapshot(SessionSnapshot snapshot)
    {
        var messages = new List<MessageItem>();
        var toolActivityTurns = new List<ToolActivityTurnItem>();
        var activities = new List<ActivityItem>();
        IReadOnlyList<TodoItem> currentTodos = [];
        var changedPaths = new List<string>();
        var changedPathSet = new HashSet<string>(StringComparer.Ordinal);
        ApprovalItem? pendingApproval = null;
        PendingQuestionItem? pendingQuestion = snapshot.PendingQuestion is { } pending
            ? PendingQuestionItem.FromSdk(pending)
            : null;
        var activeTurnId = string.Empty;
        var activeTurnState = string.Empty;
        var imagePayloads = snapshot.Images
            .Where(image => image.ImageId.Length > 0)
            .ToDictionary(image => image.ImageId, StringComparer.Ordinal);

        var conversationTurns = snapshot.ConversationTurns;
        var todoTurnId = conversationTurns
            .Where(turn => !IsTerminalTurnState(turn.State))
            .Select(turn => turn.TurnId)
            .LastOrDefault(id => id.Length > 0)
            ?? conversationTurns
                .Select(turn => turn.TurnId)
                .LastOrDefault(id => id.Length > 0)
            ?? string.Empty;
        if (conversationTurns.Count > 0)
        {
            for (var turnIndex = 0; turnIndex < conversationTurns.Count; turnIndex++)
            {
                var turn = conversationTurns[turnIndex];
                var turnId = turn.TurnId;
                var state = turn.State;
                var startedAt = turn.StartedAt ?? string.Empty;
                var completedAt = turn.CompletedAt ?? string.Empty;
                if (!IsTerminalTurnState(state)) activeTurnId = turnId;
                activeTurnState = state;
                var toolUses = turn.ToolUses;
                if (turnId == todoTurnId)
                    currentTodos = turn.Todos
                        .Select(TodoItem.FromSdk)
                        .Where(item => item is not null)
                        .Select(item => item!)
                        .ToArray();
                var turnMessages = turn.Messages
                    .OrderBy(item => item.CreatedAt, StringComparer.Ordinal)
                    .ToArray();
                var userPreview = turnMessages
                    .Where(item => item.Role == "user")
                    .Select(item => MessageText(item.Message))
                    .FirstOrDefault(text => !string.IsNullOrWhiteSpace(text)) ?? string.Empty;
                var activityTurn = new ToolActivityTurnItem(
                    turnId,
                    turnIndex + 1,
                    state,
                    BoundedPreview(userPreview),
                    turn.CreatedAt,
                    startedAt,
                    completedAt)
                {
                    IsExpanded = !IsTerminalTurnState(state)
                };
                foreach (var toolUse in toolUses.OrderBy(item => item.CreatedAt, StringComparer.Ordinal).ThenBy(item => item.Ordinal))
                {
                    activityTurn.Tools.Add(ToolActivityItemFromSdk(toolUse, turnId));
                }
                toolActivityTurns.Add(activityTurn);
                foreach (var item in turnMessages)
                {
                    var role = item.Role;
                    var message = item.Message;
                    var text = MessageText(message);
                    if (role is "user" or "assistant" && !string.IsNullOrWhiteSpace(text))
                    {
                        messages.Add(new MessageItem
                        {
                            MessageId = item.MessageId,
                            Role = role,
                            Text = text,
                            ContentSequence = messages.Count + 1,
                            TurnId = turnId,
                            Attachments = role == "user" ? MessageAttachments(message, imagePayloads) : [],
                            CanBeFinalAssistant = role == "assistant" && (message.ToolCalls?.Count ?? 0) == 0,
                            IsFinalAssistant = role == "assistant" && (message.ToolCalls?.Count ?? 0) == 0,
                            IsVisible = true,
                            TurnSequence = turnIndex + 1,
                            TurnPreview = BoundedPreview(userPreview)
                        });
                    }
                }
                var finalAssistant = messages.LastOrDefault(item => item.TurnId == turnId && item.IsAssistant && item.CanBeFinalAssistant);
                foreach (var assistant in messages.Where(item => item.TurnId == turnId && item.IsAssistant))
                    assistant.IsFinalAssistant = false;
                if (finalAssistant is not null)
                {
                    finalAssistant.IsFinalAssistant = true;
                    finalAssistant.DurationText = FormatDuration(activityTurn.StartedAt, activityTurn.CompletedAt, state);
                    finalAssistant.CompletionTimeText = FormatCompletionTime(activityTurn.CompletedAt);
                }
                foreach (var toolUse in toolUses.OrderBy(item => item.CreatedAt, StringComparer.Ordinal).ThenBy(item => item.Ordinal))
                {
                    var toolMessage = ToolMessageItem(toolUse, turnId, messages.Count + 1);
                    toolMessage.IsVisible = false;
                    messages.Add(toolMessage);
                }
            }
        }
        else
        {
            foreach (var item in snapshot.Messages)
            {
                var role = item.Role;
                if (role is not ("user" or "assistant")) continue;
                var text = MessageText(item);
                if (string.IsNullOrWhiteSpace(text)) continue;
                messages.Add(new MessageItem
                {
                    Role = role,
                    Text = text,
                    ContentSequence = messages.Count + 1,
                    IsVisible = true,
                    Attachments = role == "user" ? MessageAttachments(item, imagePayloads) : [],
                    IsFinalAssistant = role == "assistant" && (item.ToolCalls?.Count ?? 0) == 0,
                    CanBeFinalAssistant = role == "assistant" && (item.ToolCalls?.Count ?? 0) == 0
                });
            }
        }

        return new SessionSnapshotProjection(messages, toolActivityTurns, activities, changedPaths, currentTodos, pendingApproval, pendingQuestion, activeTurnId, activeTurnState);
    }

    internal void ApplySnapshot(SessionSnapshotProjection projection)
    {
        _appliedMessageIds.Clear();
        foreach (var message in projection.Messages)
        {
            if (message.MessageId.Length > 0) _appliedMessageIds.Add(message.MessageId);
        }
        DisposeMessages();
        Messages = new BulkObservableCollection<MessageItem>(projection.Messages);
        ToolActivityTurns.Clear();
        foreach (var turn in projection.ToolActivityTurns) ToolActivityTurns.Add(turn);
        SelectDefaultToolActivity();
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
        var activeActivityTurn = ToolActivityTurns.LastOrDefault(item => item.TurnId == projection.ActiveTurnId);
        UpdateActiveTurnTiming(projection.ActiveTurnId, activeActivityTurn?.StartedAt);
        OnPropertyChanged(nameof(HasMessages));
        NotifyAssistantStreamingChanged();
        OnPropertyChanged(nameof(HasActivities));
        OnPropertyChanged(nameof(HasToolActivityTurns));
        OnPropertyChanged(nameof(ToolActivitySummary));
        OnPropertyChanged(nameof(HasCurrentTodos));
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
            _ = ReloadCurrentSessionAsync(sessionId);
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
        if (type == "turn.state")
        {
            var state = payload.State ?? string.Empty;
            var turnId = payload.TurnId ?? string.Empty;
            if (!string.IsNullOrWhiteSpace(turnId)) LastTurnId = turnId;
            if (state == "admitted")
            {
                CurrentTodos.Clear();
                OnPropertyChanged(nameof(HasCurrentTodos));
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
            OnPropertyChanged(nameof(ToolActivitySummary));
        }
        if (live && type.StartsWith("checkpoint.", StringComparison.Ordinal)) _ = LoadCheckpointsAsync();
        if (live && type == "usage.updated") _ = LoadSessionUsageAsync();
        if (live && type.StartsWith("provider.exchange.", StringComparison.Ordinal) && ProviderTraceVisible) _ = RefreshProviderTracesAsync();
        if (live && (type == "tool.result" || (type == "turn.state" && IsTerminalTurnState(payload.State ?? string.Empty)))) _ = LoadChildSessionsAsync();
        if (live && (type.StartsWith("checkpoint.", StringComparison.Ordinal) || pathAdded)) _ = RefreshGitAsync();
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
        if (existing is null)
        {
            existing = new ToolActivityItem(turnId, toolCallId, name, state, request, result, output, error, string.Empty);
            turn.Tools.Add(existing);
            OnPropertyChanged(nameof(ToolActivitySummary));
            OnPropertyChanged(nameof(HasToolActivityTurns));
        }
        else
        {
            existing.Update(name, state, request, result, output, error);
        }
        if (SelectedToolActivity is null || (SelectedToolActivityTurn?.IsActive == true && existing.IsActive))
            SelectToolActivity(turn, existing);
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

    public void SelectToolActivity(ToolActivityTurnItem turn, ToolActivityItem tool)
    {
        turn.IsExpanded = true;
        SelectedToolActivityTurn = turn;
        SelectedToolActivity = tool;
    }

    public void SelectToolActivityTurn(ToolActivityTurnItem turn)
    {
        turn.IsExpanded = !turn.IsExpanded;
        SelectedToolActivityTurn = turn;
        if (SelectedToolActivity is null || SelectedToolActivity.TurnId != turn.TurnId)
            SelectedToolActivity = turn.Tools.FirstOrDefault();
    }

    public void ShowToolActivity(string turnId, string toolCallId)
    {
        var turn = ToolActivityTurns.FirstOrDefault(item => item.TurnId == turnId);
        var tool = turn?.Tools.FirstOrDefault(item => item.ToolCallId == toolCallId);
        if (turn is null || tool is null) return;
        SelectToolActivity(turn, tool);
        ToolActivityVisible = true;
        GitVisible = false;
        ProviderTraceVisible = false;
    }

    private void SelectDefaultToolActivity()
    {
        var turn = ToolActivityTurns.LastOrDefault(item => item.IsActive && item.Tools.Any(tool => tool.IsActive))
            ?? ToolActivityTurns.LastOrDefault(item => item.Tools.Count > 0);
        var tool = turn?.Tools.LastOrDefault(item => item.IsActive) ?? turn?.Tools.FirstOrDefault();
        if (turn is null || tool is null)
        {
            SelectedToolActivityTurn = null;
            SelectedToolActivity = null;
            return;
        }
        SelectToolActivity(turn, tool);
    }

    private ToolActivityTurnItem EnsureToolActivityTurn(string turnId, string preview = "")
    {
        var existing = ToolActivityTurns.FirstOrDefault(item => item.TurnId == turnId);
        if (existing is not null)
        {
            if (!string.IsNullOrWhiteSpace(preview)) existing.Update(existing.State, BoundedPreview(preview));
            return existing;
        }
        var created = new ToolActivityTurnItem(turnId, ToolActivityTurns.Count + 1, "admitted", BoundedPreview(preview), string.Empty)
        {
            IsExpanded = true
        };
        ToolActivityTurns.Add(created);
        OnPropertyChanged(nameof(HasToolActivityTurns));
        OnPropertyChanged(nameof(ToolActivitySummary));
        return created;
    }

    private void SyncActiveToolRow()
    {
        foreach (var row in Messages.Where(item => item.IsTool).ToArray()) Messages.Remove(row);
        var activeTurn = ToolActivityTurns.LastOrDefault(turn => turn.IsActive);
        var activeTool = activeTurn?.Tools.LastOrDefault(tool => tool.IsActive);
        if (activeTurn is null || activeTool is null) return;
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
            ToolResult = activeTool.Result,
            ToolOutput = activeTool.Output,
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

    private static ToolActivityItem ToolActivityItemFromSdk(SessionCallToolUse item, string turnId) => new(
        turnId,
        item.ToolCallId,
        item.Name,
        item.State,
        Pretty(item.Request),
        Pretty(item.Result),
        string.Empty,
        item.ErrorCode ?? string.Empty,
        item.CreatedAt);

    private static MessageItem ToolMessageItem(SessionCallToolUse item, string turnId, long sequence) => new()
    {
        Role = "tool",
        Kind = "tool",
        Text = item.Name,
        ContentSequence = sequence,
        TurnId = turnId,
        ToolCallId = item.ToolCallId,
        ToolName = item.Name,
        ToolState = item.State,
        ToolDetail = Pretty(item.Result ?? item.Request),
        ToolRequest = Pretty(item.Request),
        ToolResult = Pretty(item.Result),
        ToolError = item.ErrorCode ?? string.Empty,
        IsProcess = true,
        IsVisible = false
    };

    private static string BoundedPreview(string text)
    {
        const int previewLength = 72;
        var compact = string.Join(" ", text.Split((char[]?)null, StringSplitOptions.RemoveEmptyEntries));
        return compact.Length <= previewLength ? compact : compact[..previewLength].TrimEnd() + "...";
    }

    private static bool IsTerminalTurnState(string state) =>
        state is "completed" or "failed" or "cancelled" or "interrupted";

    internal static string FormatDuration(string startedAt, string completedAt, string state = "")
    {
        var startText = string.IsNullOrWhiteSpace(startedAt) ? string.Empty : startedAt;
        if (!DateTimeOffset.TryParse(startText, out var started)) return string.Empty;
        var hasCompleted = DateTimeOffset.TryParse(completedAt, out var completed);
        if (IsTerminalTurnState(state) && !hasCompleted) return string.Empty;
        var ended = hasCompleted ? completed : DateTimeOffset.Now;
        if (ended < started) ended = started;
        var elapsed = ended - started;
        if (elapsed.TotalSeconds < 1) return $"{elapsed.TotalMilliseconds:0} ms";
        if (elapsed.TotalMinutes < 1) return $"{elapsed.TotalSeconds:0.#} s";
        return $"{elapsed.TotalMinutes:0.#} m";
    }

    internal static string FormatCompletionTime(string completedAt)
    {
        return DateTimeOffset.TryParse(completedAt, out var completed)
            ? completed.ToLocalTime().ToString("HH:mm")
            : string.Empty;
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

    private static string MessageText(AgentMessage message) => message.Text;

    private static IReadOnlyList<ComposerAttachment> MessageAttachments(
        AgentMessage message,
        IReadOnlyDictionary<string, SessionImage> images)
    {
        var attachments = new List<ComposerAttachment>();
        foreach (var imageId in MessageImageIds(message))
        {
            if (images.TryGetValue(imageId, out var payload))
                attachments.Add(ComposerAttachment.FromSdk(payload));
        }
        return attachments;
    }

    internal static IReadOnlyList<string> MessageImageIds(AgentMessage message) =>
        (message.Content ?? [])
            .Where(part => part.Kind == "image_ref")
            .Select(part => part.Text)
            .Where(imageId => imageId.Length > 0)
            .ToArray();

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
