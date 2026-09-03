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
using SunCode.Desktop.Agent;

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
            var opened = await _sdk!.OpenProjectAsync(path);
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
            var opened = await _sdk!.OpenProjectAsync(path);
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

    public async Task SelectProjectAsync(ProjectItem project)
    {
        if (SelectedProject?.ProjectId == project.ProjectId || !await EnsureSdkReadyAsync()) return;
        CloseSubscription();
        ClearSession();
        await RunAsync(async () =>
        {
            await _sdk!.SelectProjectAsync(project.ProjectId);
            SelectedProject = project;
            await LoadProjectDependenciesAsync();
            ResetExplorerRoots();
            await LoadSessionsAsync();
            await RefreshGitAsync();
        }, "Project selected");
    }

    public async Task AddProjectDependencyAsync(string path)
    {
        if (!EnsureSdk() || SelectedProject is null || string.IsNullOrWhiteSpace(path)) return;
        await RunAsync(async () =>
        {
            await _sdk!.AddProjectDependencyAsync(SelectedProject.ProjectId, path);
            await LoadProjectDependenciesAsync();
            ResetExplorerRoots();
            await LoadExplorerRootsAsync();
        }, "Dependency added");
    }

    public async Task RemoveProjectDependencyAsync(ExplorerNode node)
    {
        if (!EnsureSdk() || SelectedProject is null || !node.CanRemove || node.DependencyId is null) return;
        await RunAsync(async () =>
        {
            await _sdk!.RemoveProjectDependencyAsync(SelectedProject.ProjectId, node.DependencyId);
            await LoadProjectDependenciesAsync();
            ResetExplorerRoots();
            await LoadExplorerRootsAsync();
        }, "Dependency removed");
    }

    public async Task LoadExplorerChildrenAsync(ExplorerNode node)
    {
        if (!EnsureSdk() || SelectedProject is null || !node.IsDirectory || node.IsGroup || node.IsLoaded || node.IsLoading) return;
        node.IsLoading = true;
        try
        {
            var result = await _sdk!.ListProjectDirectoryAsync(
                SelectedProject.ProjectId,
                node.DependencyId,
                node.Path);
            node.Children.Clear();
            foreach (var item in result.Array("entries").OfType<JsonObject>())
            {
                node.Children.Add(new ExplorerNode(
                    item.String("name"),
                    item.String("path"),
                    item.String("kind"),
                    node.DependencyId));
            }
            node.IsLoaded = true;
        }
        catch (Exception exception)
        {
            ReportError(exception);
        }
        finally
        {
            node.IsLoading = false;
        }
    }

    public async Task RefreshExplorerAsync()
    {
        if (SelectedProject is null) return;
        ResetExplorerRoots();
        await LoadExplorerRootsAsync();
    }

    public async Task LoadExplorerRootsAsync()
    {
        foreach (var root in ExplorerRoots)
        {
            if (root.IsGroup)
            {
                foreach (var dependency in root.Children)
                    await LoadExplorerChildrenAsync(dependency);
            }
            else
            {
                await LoadExplorerChildrenAsync(root);
            }
        }
    }

    public async Task CreateSessionAsync(string title)
    {
        if (!EnsureSdk() || SelectedProject is null) return;
        await RunAsync(async () =>
        {
            var created = await _sdk!.CreateSessionAsync(SelectedProject.ProjectId, title.Trim(), SelectedModel?.Id);
            await LoadSessionsAsync(created.String("sessionId"));
        }, "Session created");
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
        if (!EnsureSdk()) return;
        await RunAsync(async () =>
        {
            if (SelectedSession?.SessionId == session.SessionId)
            {
                CloseSubscription();
                ClearSession();
            }
            await _sdk!.ArchiveSessionAsync(session.SessionId);
            await LoadSessionsAsync();
        }, "Session archived");
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
        var operationId = Guid.NewGuid().ToString("N")[..8];
        var operationTimer = Stopwatch.StartNew();
        LogSession(operationId, session.SessionId, $"select.begin selected={SelectedSession?.SessionId ?? "<none>"} loaded={_loadedSessionId ?? "<none>"} version={_sessionLoadVersion} loading={IsSessionLoading}");
        if (!EnsureSdk())
        {
            LogSession(operationId, session.SessionId, "select.return reason=sdk_unavailable");
            return;
        }
        if (SelectedSession?.SessionId == session.SessionId
            && (_loadedSessionId == session.SessionId || IsSessionLoading))
        {
            var reason = _loadedSessionId == session.SessionId ? "already_loaded" : "already_loading";
            LogSession(operationId, session.SessionId, $"select.return reason={reason} version={_sessionLoadVersion}");
            return;
        }

        LogSession(operationId, session.SessionId, "select.close_subscription.begin");
        var closeTimer = Stopwatch.StartNew();
        CloseSubscription();
        LogSession(operationId, session.SessionId, $"select.close_subscription.end elapsed_ms={closeTimer.Elapsed.TotalMilliseconds:F1}");
        ClearSession(false);
        // SelectedSession raises ListBox.SelectionChanged synchronously. Mark the load first so
        // that the resulting selection callback cannot start a second load and subscription.
        IsSessionLoading = true;
        SelectedSession = session;
        LogSession(operationId, session.SessionId, $"select.selected updated={SelectedSession.SessionId}");
        StatusText = "Loading session...";
        var sessionId = session.SessionId;
        var loadVersion = _sessionLoadVersion;
        LogSession(operationId, sessionId, $"select.loading.begin version={loadVersion}");
        _ = RevealSessionLoadingAsync(sessionId, loadVersion);
        try
        {
            var stageTimer = Stopwatch.StartNew();
            LogSession(operationId, sessionId, $"snapshot.begin version={loadVersion}");
            var snapshot = await _sdk!.SessionSnapshotAsync(sessionId);
            LogSession(operationId, sessionId, $"snapshot.end elapsed_ms={stageTimer.Elapsed.TotalMilliseconds:F1} messages={snapshot.Array("messages").Count()} events={snapshot.Array("events").Count()}");
            if (!IsCurrentSessionLoad(sessionId, loadVersion))
            {
                LogSession(operationId, sessionId, $"snapshot.discard reason=stale current={DescribeSessionContext()}");
                return;
            }
            var projection = await Task.Run(() => ProjectSnapshot(snapshot));
            if (!IsCurrentSessionLoad(sessionId, loadVersion))
            {
                LogSession(operationId, sessionId, $"projection.discard reason=stale current={DescribeSessionContext()}");
                return;
            }
            ApplySnapshot(projection);
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
            await LoadSessionUsageAsync(sessionId, loadVersion);
            if (!IsCurrentSessionLoad(sessionId, loadVersion))
            {
                LogSession(operationId, sessionId, $"usage.discard reason=stale current={DescribeSessionContext()}");
                return;
            }
            await LoadCheckpointsAsync(sessionId, loadVersion);
            if (!IsCurrentSessionLoad(sessionId, loadVersion))
            {
                LogSession(operationId, sessionId, $"checkpoints.discard reason=stale current={DescribeSessionContext()}");
                return;
            }
            if (ProviderTraceVisible)
            {
                await RefreshProviderTracesAsync(sessionId, loadVersion);
            }
            if (!IsCurrentSessionLoad(sessionId, loadVersion))
            {
                LogSession(operationId, sessionId, $"provider_traces.discard reason=stale current={DescribeSessionContext()}");
                return;
            }

            var subscription = _sdk.Subscribe(sessionId, 0, json => OnNativeEvent(sessionId, json));
            if (!IsCurrentSessionLoad(sessionId, loadVersion))
            {
                LogSession(operationId, sessionId, $"subscribe.discard reason=stale current={DescribeSessionContext()}");
                subscription.Dispose();
                return;
            }

            _subscription = subscription;
            _loadedSessionId = sessionId;
            StatusText = "Session loaded";
            ConnectionState = "connected";
            LogSession(operationId, sessionId, $"select.completed elapsed_ms={operationTimer.Elapsed.TotalMilliseconds:F1} version={loadVersion}");
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

    public async Task SubmitTurnAsync()
    {
        var text = ComposerText.Trim();
        if (!EnsureSdk() || SelectedSession is null || SelectedModel?.Configured != true || text.Length == 0 || IsTurnActive) return;
        var attachments = ComposerAttachments.ToArray();
        DisposeSubmittedAttachments();
        _submittedAttachments = attachments;
        IsBusy = true;
        try
        {
            var result = await _sdk!.SubmitTurnAsync(
                SelectedSession.SessionId,
                text,
                SelectedModel.Id,
                SelectedReasoningEffort,
                attachments.Select(attachment => attachment.ImageId).ToArray());
            ComposerText = string.Empty;
            ComposerAttachments.Clear();
            StatusText = result.String("status") switch
            {
                "queued" => "Message queued for this turn",
                "awaiting_approval" => "Turn is awaiting approval",
                _ => "Turn submitted"
            };
            ConnectionState = "connected";
        }
        catch (Exception exception)
        {
            // ComposerAttachments still owns any images that were not consumed by a live
            // message.user event, so do not dispose them here.
            _submittedAttachments = [];
            ReportError(exception);
        }
        finally
        {
            IsBusy = false;
        }
    }

    public async Task RetryLastTurnAsync()
    {
        if (!EnsureSdk() || SelectedSession is null || IsTurnActive) return;
        IsBusy = true;
        try
        {
            var result = await _sdk!.RetryLastTurnAsync(SelectedSession.SessionId);
            StatusText = result.String("status") switch
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
            ReportError(exception);
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
        ReplaceComposerAttachments(result.Array("images").OfType<JsonObject>().Select(ComposerAttachment.FromPayload));
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
            var payload = new JsonObject
            {
                ["displayName"] = displayName,
                ["sourceKind"] = sourceKind,
                ["originalPath"] = originalPath is null ? null : JsonValue.Create(originalPath),
                ["extension"] = extension,
                ["bytesBase64"] = Convert.ToBase64String(bytes),
                ["thumbnailBase64"] = Convert.ToBase64String(thumbnailBytes)
            };
            var result = await _sdk!.AddSessionImageAsync(SelectedSession.SessionId, payload);
            ComposerAttachments.Add(ComposerAttachment.FromPayload(result));
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
        if (!EnsureSdk() || PendingApproval is null) return;
        await RunAsync(async () =>
        {
            await _sdk!.ResolveApprovalAsync(PendingApproval.ApprovalId, decision);
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
        if (!EnsureSdk() || PendingQuestion is null) return;
        var question = PendingQuestion;
        var answers = new JsonArray(question.Questions
            .Select(item => (JsonNode)new JsonArray(item.Answers
                .Select(value => (JsonNode?)JsonValue.Create(value)).ToArray()))
            .ToArray());
        await RunAsync(async () =>
        {
            await _sdk!.ReplyQuestionAsync(question.RequestId, answers);
            PendingQuestion = null;
        }, "Answers submitted");
    }

    public async Task RejectQuestionAsync()
    {
        if (!EnsureSdk() || PendingQuestion is null) return;
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
            await _sdk!.SetSessionFullControlAsync(sessionId, false);
            if (SelectedSession?.SessionId == sessionId) FullControlEnabled = false;
        }, "Full Control turned off");
    }

    public async Task RestoreCheckpointAsync(CheckpointItem checkpoint)
    {
        if (!EnsureSdk() || SelectedSession is null || !checkpoint.CanReview) return;
        await RunAsync(async () =>
        {
            await _sdk!.RestoreCheckpointAsync(checkpoint.ManifestId, SelectedSession.SessionId);
            await LoadCheckpointsAsync();
            await RefreshGitAsync();
        }, "Turn changes restored");
    }

    public async Task RefreshDiagnosticsAsync()
    {
        if (!EnsureSdk()) return;
        await RunAsync(async () =>
        {
            var diagnostics = await _sdk!.DiagnosticsAsync();
            var health = diagnostics.Object("health");
            var database = health.Object("database");
            DiagnosticsText = health.Count == 0
                ? "Diagnostics unavailable"
                : $"Agent  {health.String("agent")}\nDatabase  {(database.Bool("ok") ? "Ready" : "Check required")}";
            OnPropertyChanged(nameof(IsAgentHealthy));
        });
    }

    public async Task RefreshGitAsync()
    {
        if (!EnsureSdk() || SelectedProject is null)
        {
            ClearGit();
            return;
        }
        GitState = "loading";
        GitError = string.Empty;
        try
        {
            var status = await _sdk!.GitStatusAsync(SelectedProject.ProjectId);
            GitBranch = status.String("branch");
            GitChangedFiles = status.Int("changed_files");
            GitAdditions = status.Int("additions");
            GitDeletions = status.Int("deletions");
            GitStatusTruncated = status.Bool("truncated");
            GitFiles.Clear();
            foreach (var node in status.Array("files").OfType<JsonObject>())
            {
                GitFiles.Add(new GitFileItem(
                    node.String("path"), node.String("status"), node.Bool("staged"), node.Bool("unstaged"),
                    node.Bool("conflicted"), node.Int("additions"), node.Int("deletions"),
                    node.String("old_path"), node.Bool("binary")));
            }
            ApplyGitFilter();
            GitState = "ready";
        }
        catch (SdkException exception) when (exception.Code == "not_git_repository")
        {
            ClearGit();
            GitState = "not_repository";
            GitError = exception.Message;
        }
        catch (Exception exception)
        {
            GitState = "error";
            GitError = exception.Message;
        }
    }

    public async Task LoadGitDiffAsync(GitFileItem file, string scope = "all")
    {
        if (!EnsureSdk() || SelectedProject is null) return;
        SelectedGitFile = file;
        DiffLines.Clear();
        GitPatch = string.Empty;
        GitDiffError = string.Empty;
        GitDiffBinary = false;
        GitDiffTruncated = false;
        GitDiffAdditions = 0;
        GitDiffDeletions = 0;
        GitDiffState = "loading";
        try
        {
            var diff = await _sdk!.GitDiffAsync(SelectedProject.ProjectId, scope, file.Path);
            GitPatch = diff.String("patch");
            GitDiffBinary = diff.Bool("binary");
            GitDiffTruncated = diff.Bool("truncated");
            GitDiffAdditions = diff.Int("additions");
            GitDiffDeletions = diff.Int("deletions");
            foreach (var hunk in diff.Array("hunks").OfType<JsonObject>())
            {
                DiffLines.Add(new DiffLineItem("hunk", hunk.String("header"), string.Empty, string.Empty));
                foreach (var line in hunk.Array("lines").OfType<JsonObject>())
                {
                    DiffLines.Add(new DiffLineItem(
                        line.String("kind"), line.String("text"),
                        line["old_line"]?.ToString() ?? string.Empty,
                        line["new_line"]?.ToString() ?? string.Empty));
                }
            }
            OnPropertyChanged(nameof(HasGitDiffLines));
            OnPropertyChanged(nameof(ShowGitDiffEmpty));
            GitDiffState = "ready";
        }
        catch (Exception exception)
        {
            GitDiffState = "error";
            GitDiffError = exception.Message;
        }
    }
}
