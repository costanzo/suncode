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
    public async Task InitializeAsync()
    {
        Task initialization;
        lock (_initializationGate)
        {
            if (_sdk is not null || _disposed) return;
            _initializationTask ??= InitializeCoreAsync();
            initialization = _initializationTask;
        }

        await initialization;
    }

    public async Task OpenProjectAsync(string path)
    {
        if (string.IsNullOrWhiteSpace(path) || !await EnsureSdkReadyAsync()) return;
        await RunAsync(async () =>
        {
            var opened = await _sdk!.OpenProjectAsync(new OpenProjectRequest(path));
            await LoadProjectsAsync();
            var project = MatchOrCreateProject(opened);
            if (project is not null) await SelectProjectAsync(project);
        }, "Project opened");
    }

    public async Task<ProjectItem?> RegisterProjectAsync(string path)
    {
        if (string.IsNullOrWhiteSpace(path) || !await EnsureSdkReadyAsync()) return null;
        IsBusy = true;
        try
        {
            var opened = await _sdk!.OpenProjectAsync(new OpenProjectRequest(path));
            await LoadProjectsAsync();
            var project = MatchOrCreateProject(opened);
            StatusText = "Project opened";
            ConnectionState = "connected";
            return project;
        }
        catch (Exception exception)
        {
            ReportError(exception);
            return null;
        }
        finally
        {
            IsBusy = false;
        }
    }

    public async Task RefreshProjectsAsync()
    {
        if (!await EnsureSdkReadyAsync()) return;
        await RunAsync(LoadProjectsAsync);
    }
    
    internal void BeginProjectOpening(ProjectItem project)
    {
        _openingProject = project;
        NotifyProjectOpeningChanged();
        _ = RevealProjectOpeningAsync(project);
    }
    
    internal void EndProjectOpening()
    {
        if (_openingProject is null) return;
        _openingProject = null;
        _isProjectOpeningVisible = false;
        NotifyProjectOpeningChanged();
    }
    
    
    private async Task RevealProjectOpeningAsync(ProjectItem project)
    {
        await Task.Delay(120);
        if (_disposed || !ReferenceEquals(_openingProject, project)) return;
        _isProjectOpeningVisible = true;
        OnPropertyChanged(nameof(IsWorkspaceLoadingVisible));
    }
    
    private void NotifyProjectOpeningChanged()
    {
        OnPropertyChanged(nameof(IsProjectOpening));
        OnPropertyChanged(nameof(IsWorkspaceLoadingVisible));
        OnPropertyChanged(nameof(ShowNoSessionSelected));
        OnPropertyChanged(nameof(ShowNoSessions));
        OnPropertyChanged(nameof(ProjectTitle));
    }

    public async Task SelectProjectAsync(ProjectItem project)
    {
        if (SelectedProject?.ProjectId == project.ProjectId || !await EnsureSdkReadyAsync()) return;
        CloseSubscription();
        _sessionPresentationCache.Clear();
        ClearSession();
        ClearRecentContents();
        ClearAllComposerDrafts();
        await RunAsync(async () =>
        {
            await _sdk!.SelectProjectAsync(project.ProjectId);
            SelectedProject = project;
            await Explorer.ReloadAsync();
            await Git.CheckAvailabilityAsync();
            RestoreProjectPresentationState();
            await LoadSessionsAsync();
            RunInBackground(Git.RefreshAsync, "git.refresh");
        }, "Project selected");
    }

    public async Task AddProjectDependencyAsync(string path)
    {
        if (!EnsureSdk() || SelectedProject is null || string.IsNullOrWhiteSpace(path)) return;
        await RunAsync(async () =>
        {
            await _sdk!.AddProjectDependencyAsync(SelectedProject.ProjectId, path);
            await Explorer.ReloadAsync();
        }, "Dependency added");
    }

    public async Task RemoveProjectDependencyAsync(ExplorerNode node)
    {
        if (!EnsureSdk() || SelectedProject is null || !node.CanRemove || node.DependencyId is null) return;
        await RunAsync(async () =>
        {
            await _sdk!.RemoveProjectDependencyAsync(SelectedProject.ProjectId, node.DependencyId);
            await Explorer.ReloadAsync();
        }, "Dependency removed");
    }

    public async Task<bool> CreateSessionAsync(string title)
    {
        if (!EnsureSdk() || SelectedProject is null) return false;
        var createdSuccessfully = false;
        await RunAsync(async () =>
        {
            var created = await _sdk!.CreateSessionAsync(new CreateSessionRequest(
                SelectedProject.ProjectId,
                title.Trim(),
                SelectedModel?.Id));
            await LoadSessionsAsync(created.SessionId);
            createdSuccessfully = true;
        }, "Session created");
        return createdSuccessfully;
    }

    public async Task RenameSessionAsync(SessionItem session, string title)
    {
        if (!EnsureSdk() || string.IsNullOrWhiteSpace(title)) return;
        await RunAsync(async () =>
        {
            await _sdk!.RenameSessionAsync(session.SessionId, title.Trim());
            await LoadSessionsAsync(SelectedSession?.SessionId);
        }, "Session renamed");
    }

    public async Task ArchiveSessionAsync(SessionItem session)
    {
        if (!EnsureSdk() || session.IsArchived || session.IsTurnActive) return;
        await RunAsync(async () =>
        {
            var wasSelected = SelectedSession?.SessionId == session.SessionId;
            await _sdk!.ArchiveSessionAsync(session.SessionId);
            ForgetComposerDraft(session.SessionId);
            if (wasSelected)
            {
                CloseSubscription();
                ClearSession();
            }
            await LoadSessionsAsync();
        }, "Session archived");
    }

    public async Task RestoreSessionAsync(SessionItem session)
    {
        if (!EnsureSdk() || !session.IsArchived) return;
        await RunAsync(async () =>
        {
            await _sdk!.ReopenSessionAsync(session.SessionId);
            await LoadSessionsAsync(session.SessionId);
        }, "Session restored");
    }

    public async Task DeleteSessionPermanentlyAsync(SessionItem session)
    {
        if (!EnsureSdk() || !session.IsArchived) return;
        await RunAsync(async () =>
        {
            var wasSelected = SelectedSession?.SessionId == session.SessionId;
            await _sdk!.DeleteSessionAsync(session.SessionId);
            ForgetComposerDraft(session.SessionId);
            if (wasSelected)
            {
                CloseSubscription();
                ClearSession();
            }
            await LoadSessionsAsync();
        }, "Session permanently deleted");
    }

    public async Task SetSessionPinnedAsync(SessionItem session, bool pinned)
    {
        if (!EnsureSdk()) return;
        await RunAsync(async () =>
        {
            await _sdk!.SetSessionPinnedAsync(session.SessionId, pinned);
            await LoadSessionsAsync(SelectedSession?.SessionId);
        }, pinned ? "Session pinned" : "Session unpinned");
    }

    public async Task SelectSessionAsync(SessionItem session)
    {
        ClearSelectedChildSession();
        CloseEditor();
        var outgoingSessionId = SelectedSession?.SessionId;
        var enteringSession = outgoingSessionId != session.SessionId;
        SaveCurrentComposerDraft();
        var operationId = Guid.NewGuid().ToString("N")[..8];
        var operationTimer = Stopwatch.StartNew();
        LogSession(operationId, session.SessionId, $"select.begin selected={SelectedSession?.SessionId ?? "<none>"} loaded={_loadedSessionId ?? "<none>"} version={_sessionLoadVersion} loading={IsSessionLoading}");
        if (!EnsureSdk())
        {
            LogSession(operationId, session.SessionId, "select.return reason=sdk_unavailable");
            return;
        }
        if (!string.IsNullOrWhiteSpace(session.ModelId))
        {
            SelectedModel = Models.FirstOrDefault(model => model.Id == session.ModelId) ?? SelectedModel;
            SelectedReasoningEffort = !string.IsNullOrWhiteSpace(session.ReasoningEffort)
                && ReasoningEffortOptions.Contains(session.ReasoningEffort, StringComparer.Ordinal)
                ? session.ReasoningEffort
                : ReasoningEffortOptions.FirstOrDefault();
        }
        if (SelectedSession?.SessionId == session.SessionId
            && (_loadedSessionId == session.SessionId || IsSessionLoading))
        {
            RememberRecentSession(session);
            var reason = _loadedSessionId == session.SessionId ? "already_loaded" : "already_loading";
            LogSession(operationId, session.SessionId, $"select.return reason={reason} version={_sessionLoadVersion}");
            return;
        }

        if (outgoingSessionId is { Length: > 0 })
            CacheCurrentSessionPresentation(outgoingSessionId);

        LogSession(operationId, session.SessionId, "select.close_subscription.begin");
        var closeTimer = Stopwatch.StartNew();
        CloseSubscription();
        LogSession(operationId, session.SessionId, $"select.close_subscription.end elapsed_ms={closeTimer.Elapsed.TotalMilliseconds:F1}");
        ClearSession(false);
        // SelectedSession raises ListBox.SelectionChanged synchronously. Mark the load first so
        // that the resulting selection callback cannot start a second load and subscription.
        IsSessionLoading = true;
        SelectedSession = session;
        RestoreComposerDraft(session.SessionId);
        var restoredFromCache = RestoreSessionPresentation(session.SessionId);
        if (restoredFromCache) SessionEntered?.Invoke();
        // A session owns its model selection. Apply it immediately so the
        // composer cannot briefly display the previous session's model.
        RememberRecentSession(session);
        LogSession(operationId, session.SessionId, $"select.selected updated={SelectedSession.SessionId}");
        StatusText = "Loading session...";
        var sessionId = session.SessionId;
        var loadVersion = _sessionLoadVersion;
        LogSession(operationId, sessionId, $"select.loading.begin version={loadVersion}");
        _ = RevealSessionLoadingAsync(sessionId, loadVersion);
        SessionWatch? watch = null;
        try
        {
            var stageTimer = Stopwatch.StartNew();
            LogSession(operationId, sessionId, $"watch.begin version={loadVersion}");
            watch = await _sdk!.WatchSessionAsync(sessionId, eventValue => OnNativeEvent(sessionId, eventValue));
            var snapshot = watch.Snapshot;
            LogSession(operationId, sessionId, $"watch.ready elapsed_ms={stageTimer.Elapsed.TotalMilliseconds:F1} messages={snapshot.Messages.Count} turns={snapshot.ConversationTurns.Count} state=dormant");
            if (!IsCurrentSessionLoad(sessionId, loadVersion))
            {
                LogSession(operationId, sessionId, $"watch.discard reason=stale current={DescribeSessionContext()}");
                return;
            }
            var projection = await Task.Run(() => ProjectSnapshot(snapshot));
            if (!IsCurrentSessionLoad(sessionId, loadVersion))
            {
                LogSession(operationId, sessionId, $"projection.discard reason=stale current={DescribeSessionContext()}");
                return;
            }
            ApplySnapshot(projection, restoredFromCache);
            await LoadSessionControlAsync(sessionId, loadVersion);
            if (!IsCurrentSessionLoad(sessionId, loadVersion))
            {
                LogSession(operationId, sessionId, $"session_control.discard reason=stale current={DescribeSessionContext()}");
                return;
            }
            await LoadSessionImagesAsync(sessionId, loadVersion);
            if (!IsCurrentSessionLoad(sessionId, loadVersion))
            {
                LogSession(operationId, sessionId, $"images.discard reason=stale current={DescribeSessionContext()}");
                return;
            }
            await LoadQueuedInputsAsync(sessionId, loadVersion);
            await LoadCheckpointsAsync(sessionId, loadVersion);
            if (!IsCurrentSessionLoad(sessionId, loadVersion))
            {
                LogSession(operationId, sessionId, $"checkpoints.discard reason=stale current={DescribeSessionContext()}");
                return;
            }
            await ProviderTrace.RefreshAsync(sessionId, loadVersion);
            if (!IsCurrentSessionLoad(sessionId, loadVersion))
            {
                LogSession(operationId, sessionId, $"provider_traces.discard reason=stale current={DescribeSessionContext()}");
                return;
            }
            await LoadChildSessionsAsync();
            if (!IsCurrentSessionLoad(sessionId, loadVersion)) return;

            var activatedWatch = watch
                ?? throw new SdkException("subscription_failed", "Session watch is unavailable");
            _subscription = activatedWatch;
            watch = null;
            _loadedSessionId = sessionId;
            StatusText = "Session loaded";
            ConnectionState = "connected";
            try
            {
                activatedWatch.Start();
            }
            catch
            {
                _subscription = null;
                _loadedSessionId = null;
                activatedWatch.Dispose();
                throw;
            }
            LogSession(operationId, sessionId, $"select.completed elapsed_ms={operationTimer.Elapsed.TotalMilliseconds:F1} version={loadVersion}");
            if (enteringSession) SessionEntered?.Invoke();
        }
        catch (Exception exception)
        {
            LogSession(operationId, sessionId, $"select.failed elapsed_ms={operationTimer.Elapsed.TotalMilliseconds:F1} type={exception.GetType().Name} message={exception.Message}");
            if (IsCurrentSessionLoad(sessionId, loadVersion))
            {
                _loadedSessionId = null;
                SessionLoadError = exception.Message;
                ReportError(exception);
            }
        }
        finally
        {
            watch?.Dispose();
            if (IsCurrentSessionLoad(sessionId, loadVersion))
            {
                IsSessionLoading = false;
                LogSession(operationId, sessionId, $"select.loading.end elapsed_ms={operationTimer.Elapsed.TotalMilliseconds:F1} current={DescribeSessionContext()}");
            }
            else
            {
                LogSession(operationId, sessionId, $"select.finally.stale elapsed_ms={operationTimer.Elapsed.TotalMilliseconds:F1} current={DescribeSessionContext()}");
            }
        }
    }

    internal async Task<bool> NavigateToSessionAsync(
        string sessionId,
        string? parentSessionId = null,
        string? childSessionId = null)
    {
        if (!await EnsureSdkReadyAsync()) return false;
        var primaryId = parentSessionId ?? sessionId;
        var primary = Sessions.FirstOrDefault(item => item.SessionId == primaryId);
        if (primary is null) return false;
        await SelectSessionAsync(primary);
        if (childSessionId is null) return true;
        await LoadChildSessionsAsync();
        var child = ChildSessions.FirstOrDefault(item => item.SessionId == childSessionId);
        if (child is null || child.ParentSessionId != primaryId) return false;
        await SelectChildSessionAsync(child);
        return true;
    }

    public async Task SubmitTurnAsync()
    {
        var text = ComposerText.Trim();
        if (!EnsureSdk() || SelectedSession is null || SelectedModel?.Configured != true) return;
        if (text.Length == 0 && ComposerAttachments.Count == 0) return;
        var submittedModelId = SelectedModel.Id;
        var submittedReasoningEffort = SelectedReasoningEffort;
        var submittedSessionId = SelectedSession.SessionId;
        var idempotencyKey = Guid.NewGuid().ToString("N");
        var attachments = ComposerAttachments.ToArray();
        ComposerText = string.Empty;
        ResetCurrentComposerDraft();
        ComposerAttachments.Clear();
        DisposeSubmittedAttachments();
        _submittedAttachments = attachments;
        _inFlightSubmissions[idempotencyKey] = false;
        IsBusy = true;
        try
        {
            var result = await _sdk!.SubmitTurnAsync(new SubmitTurnRequest(
                submittedSessionId,
                text,
                idempotencyKey,
                submittedModelId,
                submittedReasoningEffort,
                attachments.Select(attachment => attachment.ImageId).ToArray()));
            var submissionStatus = result.Status;
            StatusText = submissionStatus switch
            {
                "queued" => "Follow-up queued",
                "awaiting_approval" => "Turn is awaiting approval",
                _ => "Turn submitted"
            };
            if (submissionStatus != "queued" && SelectedSession is { } session)
            {
                var updated = session with
                {
                    ModelId = submittedModelId,
                    ReasoningEffort = submittedReasoningEffort ?? string.Empty
                };
                var index = Sessions.IndexOf(session);
                if (index >= 0) Sessions[index] = updated;
                SelectedSession = updated;
            }
            await LoadQueuedInputsAsync(submittedSessionId);
            ConnectionState = "connected";
        }
        catch (Exception exception)
        {
            var admitted = _inFlightSubmissions.GetValueOrDefault(idempotencyKey);
            if (!admitted && SelectedSession?.SessionId == submittedSessionId)
            {
                ComposerText = text;
                BindComposerDraftToCurrent(SelectedSession.SessionId);
                foreach (var attachment in attachments) ComposerAttachments.Add(attachment);                
            }
            _submittedAttachments = [];
            ReportTurnError(exception);
        }
        finally
        {
            _inFlightSubmissions.Remove(idempotencyKey);
            IsBusy = false;
        }
    }

    public async Task LoadQueuedInputsAsync(string? requestedSessionId = null, long? loadVersion = null)
    {
        if (_sdk is null || SelectedSession is null) return;
        var sessionId = requestedSessionId ?? SelectedSession.SessionId;
        var messages = await _sdk.GetQueuedMessagesAsync(sessionId);
        if (!IsSessionContextCurrent(sessionId, loadVersion)) return;
        foreach (var queued in QueuedInputs) queued.Dispose();
        QueuedInputs.Clear();
        foreach (var item in messages)
        {
            QueuedInputs.Add(new QueuedInputItem
            {
                QueuedId = item.QueuedId,
                Position = item.Position,
                Mode = item.Mode,
                Text = item.Input,
                ImageIds = item.ImageIds,
                IsTurnActive = IsTurnActive,
                Attachments = item.ImageIds
                    .Where(_sessionImages.ContainsKey)
                    .Select(id => ComposerAttachment.FromSdk(_sessionImages[id]))
                    .ToArray()
            });
        }
    }

    public async Task SteerQueuedInputAsync(QueuedInputItem item)
    {
        if (!EnsureSdk() || SelectedSession is null) return;
        await RunAsync(async () =>
        {
            await _sdk!.SteerQueuedMessageAsync(SelectedSession.SessionId, item.QueuedId);
            await LoadQueuedInputsAsync();
        }, "Steering queued for the next model request");
    }

    public async Task TakeQueuedInputAsync(QueuedInputItem item)
    {
        if (!EnsureSdk() || SelectedSession is null) return;
        var sessionId = SelectedSession.SessionId;
        var draftText = ComposerText;
        var draftImageIds = ComposerAttachments.Select(attachment => attachment.ImageId).ToArray();
        var hasDraft = !string.IsNullOrWhiteSpace(draftText) || draftImageIds.Length > 0;
        var result = hasDraft
            ? await _sdk!.TakeQueuedMessageReplacingWithFollowupAsync(sessionId, item.QueuedId, draftText, draftImageIds)
            : await _sdk!.TakeQueuedMessageAsync(sessionId, item.QueuedId);
        ComposerText = result.Input;
        ReplaceComposerAttachments(result.ImageIds.Select(id => _sessionImages.GetValueOrDefault(id)).Where(image => image is not null).Select(image => ComposerAttachment.FromSdk(image!)));
        if (hasDraft) await LoadQueuedInputsAsync(sessionId);
        else QueuedInputs.Remove(item);
        OnPropertyChanged(nameof(CanSubmit));
        OnPropertyChanged(nameof(IsComposerActionEnabled));
        OnPropertyChanged(nameof(HasRemovedQueuedInput));
        StatusText = hasDraft
            ? "Message returned to the composer; previous draft added to Up next"
            : "Message returned to the composer";
    }

    public async Task RemoveQueuedInputAsync(QueuedInputItem item)
    {
        if (!EnsureSdk() || SelectedSession is null) return;
        var result = await _sdk!.TakeQueuedMessageAsync(SelectedSession.SessionId, item.QueuedId);
        _removedQueuedInput = new SunCode.Sdk.Models.QueuedMessageSnapshot(result.QueuedId, result.Position, result.Mode, result.Input, result.ImageIds);
        OnPropertyChanged(nameof(HasRemovedQueuedInput));
        await LoadQueuedInputsAsync();
        StatusText = "Queued message removed";
    }

    public async Task UndoRemoveQueuedInputAsync()
    {
        if (!EnsureSdk() || SelectedSession is null || _removedQueuedInput is not { } removed) return;
        await _sdk!.RestoreQueuedMessageAsync(SelectedSession.SessionId, removed);
        _removedQueuedInput = null;
        OnPropertyChanged(nameof(HasRemovedQueuedInput));
        await LoadQueuedInputsAsync();
        StatusText = "Queued message restored";
    }

    public async Task RetryLastTurnAsync()
    {
        if (!EnsureSdk() || SelectedSession is null || IsTurnActive) return;
        IsBusy = true;
        try
        {
            var result = await _sdk!.RetryLastTurnAsync(SelectedSession.SessionId);
            StatusText = result.Status switch
            {
                "awaiting_approval" => "Retry is awaiting approval",
                "awaiting_question" => "Retry is awaiting your answer",
                "queued" => "Retry queued for this turn",
                _ => "Turn retry submitted"
            };
            ConnectionState = "connected";
        }
        catch (Exception exception)
        {
            ReportTurnError(exception);
        }
        finally
        {
            IsBusy = false;
        }
    }

    public async Task LoadSessionImagesAsync(string? requestedSessionId = null, long? loadVersion = null)
    {
        if (_sdk is null || SelectedSession is null) return;
        var sessionId = requestedSessionId ?? SelectedSession.SessionId;
        var result = await _sdk.ListSessionImagesAsync(sessionId);
        if (!IsSessionContextCurrent(sessionId, loadVersion)) return;
        ReplaceComposerAttachments(result.Images.Select(ComposerAttachment.FromSdk));
    }

    public async Task<bool> AddSessionImageAsync(
        string displayName,
        string sourceKind,
        string? originalPath,
        string extension,
        byte[] bytes,
        byte[] thumbnailBytes)
    {
        if (!EnsureSdk() || SelectedSession is null) return false;
        if (ComposerAttachments.Count >= 3)
        {
            StatusText = "You can keep up to three images in this session";
            return false;
        }

        IsBusy = true;
        try
        {
            var result = await _sdk!.AddSessionImageAsync(new AddSessionImageRequest(
                SelectedSession.SessionId,
                displayName,
                sourceKind,
                extension,
                Convert.ToBase64String(bytes),
                Convert.ToBase64String(thumbnailBytes),
                originalPath));
            ComposerAttachments.Add(ComposerAttachment.FromSdk(result));
            StatusText = "Image uploaded";
            ConnectionState = "connected";
            return true;
        }
        catch (Exception exception)
        {
            ReportError(exception);
            return false;
        }
        finally
        {
            IsBusy = false;
        }
    }

    public async Task RemoveSessionImageAsync(ComposerAttachment attachment)
    {
        if (!EnsureSdk() || SelectedSession is null) return;
        if (QueuedInputs.Any(item => item.ImageIds.Contains(attachment.ImageId, StringComparer.Ordinal)))
        {
            StatusText = "Remove queued messages that use this image first";
            return;
        }
        await RunAsync(async () =>
        {
            await _sdk!.RemoveSessionImageAsync(SelectedSession.SessionId, attachment.ImageId);
            if (ComposerAttachments.Remove(attachment))
            {
                attachment.Dispose();
            }
        }, "Image removed");
    }

    public async Task CancelTurnAsync()
    {
        if (!EnsureSdk() || SelectedSession is null || !IsTurnActive) return;
        await RunAsync(() => _sdk!.CancelTurnAsync(SelectedSession.SessionId, ActiveTurnId), "Cancellation requested");
    }

    public async Task ResolveApprovalAsync(string decision)
    {
        if (!EnsureSdk() || IsSessionLoading || PendingApproval is null) return;
        await RunAsync(async () =>
        {
            await _sdk!.ResolveApprovalAsync(new ApprovalDecisionRequest(
                PendingApproval.ApprovalId,
                decision switch
                {
                    "allow_once" => ApprovalDecision.AllowOnce,
                    "allow_session" => ApprovalDecision.AllowSession,
                    _ => ApprovalDecision.Deny
                }));
            if (decision == "allow_session") FullControlEnabled = true;
            PendingApproval = null;
        }, decision switch
        {
            "allow_once" => "Approval granted once",
            "allow_session" => "Full Control enabled for this session",
            _ => "Approval denied"
        });
    }

    public void ToggleQuestionOption(QuestionOptionItem option)
    {
        if (PendingQuestion is null) return;
        var prompt = PendingQuestion.Questions.FirstOrDefault(item => item.Options.Contains(option));
        prompt?.Select(option);
    }

    public async Task ReplyQuestionAsync()
    {
        if (!EnsureSdk() || IsSessionLoading || PendingQuestion is null) return;
        var question = PendingQuestion;
        var answers = question.Questions
            .Select(item => (IReadOnlyList<string>)item.Answers)
            .ToArray();
        await RunAsync(async () =>
        {
            await _sdk!.ReplyQuestionAsync(new ReplyQuestionRequest(question.RequestId, answers));
            PendingQuestion = null;
        }, "Answers submitted");
    }

    public async Task RejectQuestionAsync()
    {
        if (!EnsureSdk() || IsSessionLoading || PendingQuestion is null) return;
        var requestId = PendingQuestion.RequestId;
        await RunAsync(async () =>
        {
            await _sdk!.RejectQuestionAsync(requestId);
            PendingQuestion = null;
        }, "Question skipped");
    }

    public async Task DisableFullControlAsync()
    {
        if (!EnsureSdk() || SelectedSession is null || !FullControlEnabled) return;
        var sessionId = SelectedSession.SessionId;
        await RunAsync(async () =>
        {
            await _sdk!.SetSettingAsync(new SetSettingRequest(
                "session",
                SelectedProject?.ProjectId,
                sessionId,
                "full_control",
                JsonSerializer.SerializeToElement(false)));
            if (SelectedSession?.SessionId == sessionId) FullControlEnabled = false;
        }, "Full Control turned off");
    }

    public async Task RestoreCheckpointAsync(SunCode.Desktop.Models.CheckpointItem checkpoint)
    {
        if (!EnsureSdk() || SelectedSession is null || !checkpoint.CanReview) return;
        await RunAsync(async () =>
        {
            await _sdk!.RestoreCheckpointAsync(checkpoint.ManifestId, SelectedSession.SessionId);
            await LoadCheckpointsAsync();
            await Git.RefreshAsync();
        }, "Turn changes restored");
    }

    public async Task RefreshDiagnosticsAsync()
    {
        if (!EnsureSdk()) return;
        await RunAsync(async () =>
        {
            var diagnostics = await _sdk!.GetDiagnosticsAsync();
            DiagnosticsText = diagnostics.Health.Ok
                ? $"Agent  {diagnostics.Health.Agent}\nDatabase  {(diagnostics.Health.Database.Ok ? "Ready" : "Check required")}"
                : "Diagnostics unavailable";
            OnPropertyChanged(nameof(IsAgentHealthy));
        });
    }
}
