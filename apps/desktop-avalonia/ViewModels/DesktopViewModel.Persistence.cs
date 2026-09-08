using System.Collections.ObjectModel;
using System.Diagnostics;
using System.Globalization;
using System.Text.Json;
using System.Text.Json.Nodes;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Threading;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Sdk;

namespace SunCode.Desktop.ViewModels;

public sealed partial class DesktopViewModel : ObservableObject, IDisposable
{
    private async Task LoadProjectsAsync()
    {
        if (_sdk is null) return;
        var result = await _sdk.ListProjectsAsync();
        Projects.Clear();
        foreach (var item in result.Array("projects").OfType<JsonObject>())
        {
            Projects.Add(new ProjectItem(item.String("projectId"), item.String("displayName"), item.String("canonicalRoot")));
        }
        OnPropertyChanged(nameof(HasProjects));
    }

    private ProjectItem? MatchOrCreateProject(JsonObject opened)
    {
        var projectId = opened.String("projectId");
        if (projectId.Length == 0) return null;

        var project = Projects.FirstOrDefault(item => item.ProjectId == projectId);
        if (project is not null) return project;

        var fallback = new ProjectItem(
            projectId,
            opened.String("displayName"),
            opened.String("canonicalRoot"));

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
        foreach (var item in result.Array("dependencies").OfType<JsonObject>())
        {
            ProjectDependencies.Add(new ProjectDependencyItem(
                item.String("dependencyId"),
                item.String("displayName")));
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
        var sessionStates = result.Object("sessionStates");
        Sessions.Clear();
        foreach (var item in result.Array("sessions").OfType<JsonObject>())
        {
            var sessionId = item.String("sessionId");
            Sessions.Add(new SessionItem(sessionId, item.String("title"), item.String("lastActivityAt"), !string.IsNullOrWhiteSpace(item.String("pinAt", "pin_at")), sessionStates.String(sessionId)));
        }
        OnPropertyChanged(nameof(HasSessions));
        var session = Sessions.FirstOrDefault(item => item.SessionId == preferredSessionId)
            ?? Sessions.FirstOrDefault(item => item.SessionId == SelectedSession?.SessionId)
            ?? Sessions.FirstOrDefault();
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
        var result = await _sdk.ListModelsAsync();
        _selectedModel = null;
        Models.Clear();
        Providers.Clear();
        foreach (var item in result.Array("models").OfType<JsonObject>())
        {
            Models.Add(new ModelItem(
                item.String("id"),
                item.String("provider"),
                item.String("providerLabel", "provider_label"),
                item.String("availability"),
                item.Object("capabilities").Bool("reasoning_effort"),
                item.Object("capabilities").Bool("vision"),
                item.String("apiBase", "api_base"),
                item.String("defaultApiBase", "default_api_base"),
                item.Array("reasoningEfforts", "reasoning_efforts")
                    .Select(value => value?.GetValue<string>() ?? string.Empty)
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

    public IEnumerable<ModelItem> ModelsForProvider(string providerId) =>
        Models.Where(model => model.Provider == providerId);

    private async Task LoadCredentialsAsync()
    {
        if (_sdk is null) return;
        var result = await _sdk.ListCredentialsAsync();
        Credentials.Clear();
        foreach (var item in result.Array("credentials").OfType<JsonObject>())
        {
            Credentials.Add(new CredentialItem(item.String("provider"), item.Bool("configured")));
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
        var result = await _sdk.ListSettingsAsync();
        var settings = result.Array("settings").OfType<JsonObject>().ToArray();
        string StringSetting(string key, string fallback)
        {
            var node = settings.FirstOrDefault(item => item.String("key") == key)?["value"];
            return node is JsonValue value && value.TryGetValue<string>(out var parsed)
                ? parsed
                : fallback;
        }
        long LongSetting(string key, long fallback)
        {
            var node = settings.FirstOrDefault(item => item.String("key") == key)?["value"];
            return node is JsonValue value && value.TryGetValue<long>(out var parsed)
                ? parsed
                : fallback;
        }
        bool BoolSetting(string key, bool fallback)
        {
            var node = settings.FirstOrDefault(item => item.String("key") == key)?["value"];
            return node is JsonValue value && value.TryGetValue<bool>(out var parsed)
                ? parsed
                : fallback;
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
        DiagnosticLog.Configure(
            LogLevel,
            LogDirectory,
            LogMaxBytes,
            LogRetention);

        foreach (var item in settings)
        {
            var key = item.String("key");
            if (item["value"] is not JsonValue settingValue
                || !settingValue.TryGetValue<string>(out var value)) continue;
            if (key == "theme_mode" && value is "dark" or "light") SetTheme(value);
            if (key == "default_model") SelectedModel = Models.FirstOrDefault(model => model.Id == value) ?? SelectedModel;
        }
    }

    private async Task LoadSessionControlAsync(string sessionId, long loadVersion)
    {
        if (_sdk is null || SelectedProject is null) return;
        var result = await _sdk.ListSessionSettingsAsync(SelectedProject.ProjectId, sessionId);
        if (!IsCurrentSessionLoad(sessionId, loadVersion)) return;
        var setting = result.Array("settings")
            .OfType<JsonObject>()
            .FirstOrDefault(item => item.String("key") == "full_control");
        FullControlEnabled = setting?["value"] is JsonValue value
            && value.TryGetValue<bool>(out var enabled)
            && enabled;
    }

    private async Task LoadSessionUsageAsync(string? requestedSessionId = null, long? loadVersion = null)
    {
        if (_sdk is null || SelectedSession is null) return;
        var sessionId = requestedSessionId ?? SelectedSession.SessionId;
        var result = await _sdk.SessionUsageAsync(sessionId);
        if (!IsSessionContextCurrent(sessionId, loadVersion)) return;
        SessionTotalTokens = result.Long("total_tokens");
    }

    private async Task LoadCheckpointsAsync(string? requestedSessionId = null, long? loadVersion = null)
    {
        if (_sdk is null || SelectedSession is null) return;
        var sessionId = requestedSessionId ?? SelectedSession.SessionId;
        var result = await _sdk.ListCheckpointsAsync(sessionId);
        if (!IsSessionContextCurrent(sessionId, loadVersion)) return;
        var checkpoints = result.Array("checkpoints").OfType<JsonObject>().Select(item =>
        {
            var paths = item.Array("paths").Select(node => node?.GetValue<string>() ?? string.Empty).Where(path => path.Length > 0).ToArray();
            return new CheckpointItem(item.String("manifestId"), item.String("turnId"), item.String("status"), paths);
        });
        Checkpoints.ReplaceAll(checkpoints);
        OnPropertyChanged(nameof(HasCheckpoints));
        NotifyReviewPresentationChanged();
    }

    internal static SessionSnapshotProjection ProjectSnapshot(JsonObject snapshot)
    {
        var messages = new List<MessageItem>();
        var toolActivityTurns = new List<ToolActivityTurnItem>();
        var activities = new List<ActivityItem>();
        IReadOnlyList<TodoItem> currentTodos = [];
        var changedPaths = new List<string>();
        var changedPathSet = new HashSet<string>(StringComparer.Ordinal);
        ApprovalItem? pendingApproval = null;
        PendingQuestionItem? pendingQuestion = (snapshot["pendingQuestion"] as JsonObject ?? snapshot["pending_question"] as JsonObject) is { } pendingPayload
            ? PendingQuestionItem.FromPayload(pendingPayload)
            : null;
        var activeTurnId = string.Empty;
        var activeTurnState = string.Empty;
        var imagePayloads = snapshot.Array("images")
            .OfType<JsonObject>()
            .Where(image => image.String("imageId", "image_id").Length > 0)
            .ToDictionary(image => image.String("imageId", "image_id"), StringComparer.Ordinal);

        var conversationTurns = snapshot.Array("conversationTurns").OfType<JsonObject>().ToArray();
        var todoTurnId = conversationTurns
            .Where(turn => !IsTerminalTurnState(turn.String("state")))
            .Select(turn => turn.String("turnId", "turn_id"))
            .LastOrDefault(id => id.Length > 0)
            ?? conversationTurns
                .Select(turn => turn.String("turnId", "turn_id"))
                .LastOrDefault(id => id.Length > 0)
            ?? string.Empty;
        if (conversationTurns.Length > 0)
        {
            for (var turnIndex = 0; turnIndex < conversationTurns.Length; turnIndex++)
            {
                var turn = conversationTurns[turnIndex];
                var turnId = turn.String("turnId", "turn_id");
                var state = turn.String("state");
                if (!IsTerminalTurnState(state)) activeTurnId = turnId;
                activeTurnState = state;
                var toolUses = turn.Array("toolUses").OfType<JsonObject>().ToArray();
                if (turnId == todoTurnId)
                    currentTodos = ParseTodos(turn["todos"]);
                var turnMessages = turn.Array("messages").OfType<JsonObject>()
                    .OrderBy(item => item.String("createdAt", "created_at"), StringComparer.Ordinal)
                    .ToArray();
                var userPreview = turnMessages
                    .Where(item => item.String("role") == "user")
                    .Select(item => MessageText(item.Object("message")))
                    .FirstOrDefault(text => !string.IsNullOrWhiteSpace(text)) ?? string.Empty;
                var activityTurn = new ToolActivityTurnItem(
                    turnId,
                    turnIndex + 1,
                    state,
                    BoundedPreview(userPreview),
                    turn.String("createdAt", "created_at"))
                {
                    IsExpanded = !IsTerminalTurnState(state)
                };
                foreach (var toolUse in toolUses.OrderBy(item => item.String("createdAt", "created_at"), StringComparer.Ordinal).ThenBy(item => item.Int("ordinal")))
                {
                    activityTurn.Tools.Add(ToolActivityItemFromJson(toolUse, turnId));
                }
                toolActivityTurns.Add(activityTurn);
                foreach (var item in turnMessages)
                {
                    var role = item.String("role");
                    var message = item.Object("message");
                    var text = MessageText(message);
                    if (role is "user" or "assistant" && !string.IsNullOrWhiteSpace(text))
                    {
                        messages.Add(new MessageItem
                        {
                            MessageId = item.String("messageId", "message_id"),
                            Role = role,
                            Text = text,
                            ContentSequence = messages.Count + 1,
                            TurnId = turnId,
                            Attachments = role == "user" ? MessageAttachments(message, imagePayloads) : [],
                            CanBeFinalAssistant = role == "assistant" && message.Array("tool_calls").Count == 0,
                            IsFinalAssistant = role == "assistant" && message.Array("tool_calls").Count == 0,
                            IsVisible = role == "assistant",
                            TurnSequence = turnIndex + 1,
                            TurnPreview = BoundedPreview(userPreview)
                        });
                    }
                }
                var firstAssistant = messages.FirstOrDefault(item => item.TurnId == turnId && item.IsAssistant);
                if (firstAssistant is not null) firstAssistant.ShowTurnMarker = true;
                foreach (var toolUse in toolUses.OrderBy(item => item.String("createdAt", "created_at"), StringComparer.Ordinal).ThenBy(item => item.Int("ordinal")))
                {
                    var toolMessage = ToolMessageItem(toolUse, turnId, messages.Count + 1);
                    toolMessage.IsVisible = false;
                    messages.Add(toolMessage);
                }
            }
        }
        else
        {
            foreach (var item in snapshot.Array("messages").OfType<JsonObject>())
            {
                var role = item.String("role");
                if (role is not ("user" or "assistant")) continue;
                var text = MessageText(item);
                if (string.IsNullOrWhiteSpace(text)) continue;
                messages.Add(new MessageItem
                {
                    Role = role,
                    Text = text,
                    ContentSequence = messages.Count + 1,
                    IsVisible = role == "assistant",
                    Attachments = role == "user" ? MessageAttachments(item, imagePayloads) : [],
                    IsFinalAssistant = role == "assistant",
                    CanBeFinalAssistant = role == "assistant"
                });
            }
        }

        foreach (var item in snapshot.Array("events").OfType<JsonObject>())
        {
            var type = item.String("event_type", "eventType");
            if (type is "message.user" or "message.assistant" or "message.tool") continue;
            var payload = item.Object("payload");
            if (!type.StartsWith("provider.exchange.", StringComparison.Ordinal))
            {
                activities.Add(new ActivityItem(type, EventText(type, payload), activities.Count + 1, payload.String("state"), payload.String("operation")));
            }

            if (type == "context.compacted")
            {
                messages.Add(new MessageItem
                {
                    Role = "assistant",
                    Kind = "context.compacted",
                    Text = EventText(type, payload),
                    ContentSequence = messages.Count + 1,
                    TurnId = payload.String("turn_id"),
                    IsProcess = true
                });
            }

            foreach (var path in new[] { payload.String("path"), payload.String("from"), payload.String("to") })
            {
                if (path.Length > 0 && changedPathSet.Add(path)) changedPaths.Add(path);
            }

            if (type == "approval.requested") pendingApproval = ApprovalItem.FromPayload(payload);
            if (type == "approval.resolved") pendingApproval = null;
            if (type == "question.asked") pendingQuestion = PendingQuestionItem.FromPayload(payload);
            if (type is "question.replied" or "question.rejected") pendingQuestion = null;
            if (type == "turn.state")
            {
                var state = payload.String("state");
                activeTurnId = state is "completed" or "failed" or "cancelled" or "interrupted"
                    ? string.Empty
                    : payload.String("turn_id");
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
        OnPropertyChanged(nameof(HasMessages));
        OnPropertyChanged(nameof(HasActivities));
        OnPropertyChanged(nameof(HasToolActivityTurns));
        OnPropertyChanged(nameof(ToolActivitySummary));
        OnPropertyChanged(nameof(HasCurrentTodos));
        OnPropertyChanged(nameof(LatestActivityText));
    }

    private void OnNativeEvent(string sessionId, string json) => Dispatcher.UIThread.Post(() =>
    {
        if (_disposed || SelectedSession?.SessionId != sessionId)
        {
            return;
        }
        try
        {
            if (JsonNode.Parse(json) is JsonObject item)
            {
                if (item.String("event_type", "eventType") == "resync.required")
                {
                    LogSession("event", sessionId, "resync.required reload_begin");
                    _ = ReloadCurrentSessionAsync(sessionId);
                    return;
                }
                ApplyEvent(item, true);
            }
        }
        catch (JsonException exception)
        {
            StatusText = $"Ignored malformed agent event: {exception.Message}";
        }
    });

    private async Task ReloadCurrentSessionAsync(string sessionId)
    {
        if (SelectedSession?.SessionId != sessionId) return;
        _loadedSessionId = null;
        CloseSubscription();
        await SelectSessionAsync(SelectedSession);
    }

    internal void ApplyEvent(JsonObject value, bool live)
    {
        var type = value.String("event_type", "eventType");
        var payload = value.Object("payload");
        var text = EventText(type, payload);

        if (type == "assistant.delta")
        {
            var turnId = payload.String("turn_id");
            var assistant = Messages.LastOrDefault(message =>
                message.TurnId == turnId && message.Role == "assistant" && message.Streaming);
            var delta = payload.String("text");
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
                        ShowTurnMarker = !Messages.Any(message => message.TurnId == turnId && message.IsAssistant),
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
        }
        else if (type is "message.user" or "message.assistant")
        {
            var turnId = payload.String("turn_id");
            var messageId = payload.String("message_id", "messageId");
            var message = payload.Object("message");
            var canBeFinalAssistant = type == "message.assistant" && message.Array("tool_calls").Count == 0;
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
                    IsVisible = false
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
                    Messages.Add(new MessageItem
                    {
                        MessageId = messageId,
                        Role = "assistant",
                        Text = text,
                        ContentSequence = Messages.Count + 1,
                        TurnId = turnId,
                        CanBeFinalAssistant = canBeFinalAssistant,
                        IsFinalAssistant = canBeFinalAssistant,
                        ShowTurnMarker = !Messages.Any(item => item.TurnId == turnId && item.IsAssistant),
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
                OnPropertyChanged(nameof(HasMessages));
            }
        }
        else if (type is "tool.requested" or "tool.state" or "tool.result" or "tool.output")
        {
            ApplyToolEvent(payload, type);
            Activities.Add(new ActivityItem(type, text, Activities.Count + 1, payload.String("state"), payload.String("name")));
            OnPropertyChanged(nameof(HasActivities));
            OnPropertyChanged(nameof(LatestActivityText));
        }
        else if (type == "todo.updated")
        {
            CurrentTodos.ReplaceAll(ParseTodos(payload["todos"]));
            OnPropertyChanged(nameof(HasCurrentTodos));
            Activities.Add(new ActivityItem(type, text, Activities.Count + 1, payload.String("state"), "todowrite"));
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
                TurnId = payload.String("turn_id"),
                IsProcess = true
            });
            OnPropertyChanged(nameof(HasMessages));
        }
        else if (!type.StartsWith("provider.exchange.", StringComparison.Ordinal))
        {
            Activities.Add(new ActivityItem(type, text, Activities.Count + 1, payload.String("state"), payload.String("operation")));
            OnPropertyChanged(nameof(HasActivities));
            OnPropertyChanged(nameof(LatestActivityText));
        }

        var pathAdded = false;
        foreach (var path in new[] { payload.String("path"), payload.String("from"), payload.String("to") })
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
        if (type == "approval.requested") PendingApproval = ApprovalItem.FromPayload(payload);
        if (type == "approval.resolved")
        {
            if (payload.String("decision") == "allow_session") FullControlEnabled = true;
            PendingApproval = null;
        }
        if (type == "question.asked") PendingQuestion = PendingQuestionItem.FromPayload(payload);
        if (type is "question.replied" or "question.rejected") PendingQuestion = null;
        if (type == "turn.state")
        {
            var state = payload.String("state");
            var turnId = payload.String("turn_id");
            if (!string.IsNullOrWhiteSpace(turnId)) LastTurnId = turnId;
            if (state == "admitted")
            {
                CurrentTodos.Clear();
                OnPropertyChanged(nameof(HasCurrentTodos));
            }
            ActiveTurnId = IsTerminalTurnState(state) ? string.Empty : turnId;
            ActiveTurnState = state;
            var activityTurn = EnsureToolActivityTurn(turnId);
            activityTurn.Update(state);
            if (!IsTerminalTurnState(state)) activityTurn.IsExpanded = true;
            if (IsTerminalTurnState(state))
            {
                var finalAssistant = Messages.LastOrDefault(item => item.TurnId == turnId && item.IsAssistant && item.CanBeFinalAssistant);
                if (finalAssistant is not null) finalAssistant.IsFinalAssistant = true;
            }
            SyncActiveToolRow();
            OnPropertyChanged(nameof(ToolActivitySummary));
        }
        if (live && type.StartsWith("checkpoint.", StringComparison.Ordinal)) _ = LoadCheckpointsAsync();
        if (live && type == "usage.updated") _ = LoadSessionUsageAsync();
        if (live && type.StartsWith("provider.exchange.", StringComparison.Ordinal) && ProviderTraceVisible) _ = RefreshProviderTracesAsync();
        if (live && (type.StartsWith("checkpoint.", StringComparison.Ordinal) || pathAdded)) _ = RefreshGitAsync();
    }

    private void ApplyToolEvent(JsonObject payload, string eventType)
    {
        var turnId = payload.String("turn_id");
        var toolCallId = payload.String("tool_call_id");
        if (turnId.Length == 0 || toolCallId.Length == 0) return;
        var turn = EnsureToolActivityTurn(turnId);
        var existing = turn.Tools.FirstOrDefault(tool => tool.ToolCallId == toolCallId);
        var state = eventType == "tool.state"
            ? payload.String("state")
            : existing?.State ?? "requested";
        var name = payload.String("name");
        if (name.Length == 0) name = existing?.Name ?? "tool";
        var request = eventType == "tool.requested"
            ? Pretty(payload["arguments"])
            : existing?.Request ?? string.Empty;
        var result = eventType == "tool.result"
            ? Pretty(payload["result"])
            : existing?.Result ?? string.Empty;
        var output = eventType == "tool.output"
            ? AppendBoundedOutput(existing?.Output ?? string.Empty, DecodeOutputChunk(payload))
            : existing?.Output ?? string.Empty;
        var error = eventType == "tool.state"
            ? payload.String("reason")
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

    private static string DecodeOutputChunk(JsonObject payload)
    {
        var encoded = payload.String("chunk_base64", "chunkBase64");
        if (encoded.Length == 0) return payload.String("chunk");
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
            ToolError = activeTool.Error
        });
    }

    private static ToolActivityItem ToolActivityItemFromJson(JsonObject item, string turnId) => new(
        turnId,
        item.String("toolCallId", "tool_call_id"),
        item.String("name"),
        item.String("state"),
        Pretty(item["request"]),
        Pretty(item["result"]),
        string.Empty,
        item.String("errorCode", "error_code"),
        item.String("createdAt", "created_at"));

    private static MessageItem ToolMessageItem(JsonObject item, string turnId, long sequence) => new()
    {
        Role = "tool",
        Kind = "tool",
        Text = item.String("name"),
        ContentSequence = sequence,
        TurnId = turnId,
        ToolCallId = item.String("toolCallId", "tool_call_id"),
        ToolName = item.String("name"),
        ToolState = item.String("state"),
        ToolDetail = Pretty(item["result"] ?? item["request"]),
        ToolRequest = Pretty(item["request"]),
        ToolResult = Pretty(item["result"]),
        ToolError = item.String("errorCode", "error_code"),
        IsProcess = true,
        IsVisible = false
    };

    private static string BoundedPreview(string text)
    {
        const int previewLength = 72;
        var compact = string.Join(" ", text.Split((char[]?)null, StringSplitOptions.RemoveEmptyEntries));
        return compact.Length <= previewLength ? compact : compact[..previewLength].TrimEnd() + "...";
    }

    private static IReadOnlyList<TodoItem> ParseTodos(JsonNode? value) =>
        (value as JsonArray)?.OfType<JsonObject>()
            .Select(TodoItem.FromPayload)
            .Where(item => item is not null)
            .Select(item => item!)
            .ToArray() ?? [];

    private static bool IsTerminalTurnState(string state) =>
        state is "completed" or "failed" or "cancelled" or "interrupted";

    private static string EventText(string type, JsonObject payload)
    {
        var message = payload.Object("message");
        var messageText = MessageText(message);
        if (type is "message.user" or "message.assistant" or "message.tool" || !string.IsNullOrEmpty(messageText)) return messageText;
        return type switch
        {
            "approval.requested" => $"Approval required for {payload.String("operation")}",
            "question.asked" => "Waiting for an answer",
            "question.replied" => "Question answered",
            "question.rejected" => "Question skipped",
            "todo.updated" => "Todo list updated",
            "context.compacted" => $"Context compacted · retained {payload.String("retained_tokens")} tokens",
            "checkpoint.captured" => $"Checkpoint captured for {payload.String("path")}",
            "checkpoint.restore_failed" => "Undo stopped because a file changed outside SunCode",
            "turn.state" => $"Turn {payload.String("state")}",
            "assistant.delta" => payload.String("text"),
            "tool.output" => $"Command output · {payload.String("stream")}",
            _ => type
        };
    }

    private static string MessageText(JsonObject message)
    {
        return string.Join("\n", message.Array("content")
            .OfType<JsonObject>()
            .Where(part => part.String("type") == "text")
            .Select(part => part.String("text")));
    }

    private static IReadOnlyList<ComposerAttachment> MessageAttachments(
        JsonObject message,
        IReadOnlyDictionary<string, JsonObject> images)
    {
        var attachments = new List<ComposerAttachment>();
        foreach (var imageId in MessageImageIds(message))
        {
            if (images.TryGetValue(imageId, out var payload))
                attachments.Add(ComposerAttachment.FromPayload(payload));
        }
        return attachments;
    }

    internal static IReadOnlyList<string> MessageImageIds(JsonObject message) =>
        message.Array("content")
            .OfType<JsonObject>()
            .Where(part => part.String("type") == "image_ref")
            .Select(part => part.String("text"))
            .Where(imageId => imageId.Length > 0)
            .ToArray();

    private IReadOnlyList<ComposerAttachment> PendingMessageAttachments(JsonObject message)
    {
        var imageIds = message.Array("content")
            .OfType<JsonObject>()
            .Where(part => part.String("type") == "image_ref")
            .Select(part => part.String("text"))
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
