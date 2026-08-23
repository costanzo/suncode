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
using SunCode.Desktop.Runtime;

namespace SunCode.Desktop.ViewModels;

public sealed class DesktopViewModel : ObservableObject, IDisposable
{
    private RuntimeSdk? _sdk;
    private IDisposable? _subscription;
    private BulkObservableCollection<MessageItem> _messages = [];
    private ProjectItem? _selectedProject;
    private SessionItem? _selectedSession;
    private ModelItem? _selectedModel;
    private GitFileItem? _selectedGitFile;
    private ProviderTraceItem? _selectedProviderTrace;
    private ProviderTraceItem? _selectedProviderTraceDetails;
    private ProviderTraceContentItem? _selectedProviderTraceContent;
    private readonly Dictionary<string, ProviderTraceItem> _providerTraceDetails = new(StringComparer.Ordinal);
    private readonly Dictionary<string, Task<ProviderTraceItem>> _providerTraceDetailLoads = new(StringComparer.Ordinal);
    private readonly HashSet<string> _appliedMessageIds = new(StringComparer.Ordinal);
    private ApprovalItem? _pendingApproval;
    private string _connectionState = "disconnected";
    private string _statusText = "Starting local runtime...";
    private string _composerText = string.Empty;
    private string _activeTurnId = string.Empty;
    private string _themeMode = "dark";
    private string _logLevel = "INFO";
    private string _logDirectory = string.Empty;
    private long _logMaxBytes = 10 * 1024 * 1024;
    private int _logRetention = 5;
    private string _diagnosticsText = "Diagnostics unavailable";
    private string _gitState = "idle";
    private string _gitError = string.Empty;
    private string _gitDiffState = "idle";
    private string _gitDiffError = string.Empty;
    private string _gitPatch = string.Empty;
    private string _gitScope = "all";
    private string _gitFilter = string.Empty;
    private string _providerTraceState = "idle";
    private string _providerTraceError = string.Empty;
    private string _providerTraceFilter = string.Empty;
    private string _sessionLoadError = string.Empty;
    private string _gitBranch = string.Empty;
    private bool _gitStatusTruncated;
    private bool _fullControlEnabled;
    private bool _gitDiffBinary;
    private bool _gitDiffTruncated;
    private int _gitDiffAdditions;
    private int _gitDiffDeletions;
    private int _gitChangedFiles;
    private int _gitAdditions;
    private int _gitDeletions;
    private int _gitConflicts;
    private long _sessionTotalTokens;
    private long _sessionLoadVersion;
    private string? _loadedSessionId;
    private bool _navigationVisible = true;
    private bool _reviewVisible = true;
    private bool _navigationPinned = true;
    private bool _explorerVisible;
    private bool _gitVisible;
    private bool _providerTraceVisible;
    private double _layoutWidth = 1440;
    private double _navigationPaneWidth = 272;
    private double _reviewPaneWidth = 312;
    private double _bottomDrawerHeight = 360;
    private bool _isBusy;
    private bool _isSessionLoading;
    private bool _isSessionLoadingVisible;
    private bool _disposed;

    public event Action<string>? ThemeChanged;
    public event Action? ConversationChanged;

    public ObservableCollection<ProjectItem> Projects { get; } = [];
    public ObservableCollection<SessionItem> Sessions { get; } = [];
    public ObservableCollection<ModelItem> Models { get; } = [];
    public ObservableCollection<CredentialItem> Credentials { get; } = [];
    public ObservableCollection<ProjectDependencyItem> ProjectDependencies { get; } = [];
    public ObservableCollection<ExplorerNode> ExplorerRoots { get; } = [];
    public BulkObservableCollection<MessageItem> Messages
    {
        get => _messages;
        private set
        {
            if (ReferenceEquals(_messages, value)) return;
            _messages = value;
            OnPropertyChanged();
            OnPropertyChanged(nameof(HasMessages));
        }
    }
    public BulkObservableCollection<ActivityItem> Activities { get; } = [];
    public BulkObservableCollection<string> ChangedPaths { get; } = [];
    public BulkObservableCollection<CheckpointItem> Checkpoints { get; } = [];
    public ObservableCollection<GitFileItem> GitFiles { get; } = [];
    public ObservableCollection<GitFileItem> FilteredGitFiles { get; } = [];
    public ObservableCollection<DiffLineItem> DiffLines { get; } = [];
    public ObservableCollection<ProviderTraceItem> ProviderTraces { get; } = [];
    public ObservableCollection<ProviderTraceTurnItem> ProviderTraceTurns { get; } = [];
    public ObservableCollection<ProviderTraceTurnItem> FilteredProviderTraceTurns { get; } = [];

    public ProjectItem? SelectedProject
    {
        get => _selectedProject;
        private set
        {
            if (SetProperty(ref _selectedProject, value))
            {
                OnPropertyChanged(nameof(IsProjectOpen));
                OnPropertyChanged(nameof(ProjectTitle));
            }
        }
    }

    public SessionItem? SelectedSession
    {
        get => _selectedSession;
        private set
        {
            if (ReferenceEquals(_selectedSession, value)) return;
            _selectedSession = value;
            OnPropertyChanged();
            OnPropertyChanged(nameof(SessionTitle));
            OnPropertyChanged(nameof(CanSubmit));
            OnPropertyChanged(nameof(CanCompose));
            OnPropertyChanged(nameof(HasSelectedSession));
            OnPropertyChanged(nameof(ComposerPlaceholder));
        }
    }

    public ModelItem? SelectedModel
    {
        get => _selectedModel;
        set
        {
            if (SetProperty(ref _selectedModel, value))
            {
                OnPropertyChanged(nameof(CanSubmit));
                OnPropertyChanged(nameof(CanCompose));
                OnPropertyChanged(nameof(SelectedModelName));
                OnPropertyChanged(nameof(ComposerPlaceholder));
            }
        }
    }

    public GitFileItem? SelectedGitFile
    {
        get => _selectedGitFile;
        set
        {
            if (SetProperty(ref _selectedGitFile, value)) OnPropertyChanged(nameof(SelectedGitPath));
        }
    }

    public ProviderTraceItem? SelectedProviderTrace
    {
        get => _selectedProviderTrace;
        set
        {
            if (SetProperty(ref _selectedProviderTrace, value))
            {
                SelectedProviderTraceDetails = null;
                SelectedProviderTraceContent = null;
                OnPropertyChanged(nameof(SelectedProviderTraceTitle));
                OnPropertyChanged(nameof(HasSelectedProviderTrace));
                OnPropertyChanged(nameof(ShowSelectedProviderTraceOverview));
            }
        }
    }

    public ProviderTraceItem? SelectedProviderTraceDetails
    {
        get => _selectedProviderTraceDetails;
        private set
        {
            if (SetProperty(ref _selectedProviderTraceDetails, value))
            {
                OnPropertyChanged(nameof(SelectedProviderTraceTitle));
                OnPropertyChanged(nameof(HasSelectedProviderTrace));
            }
        }
    }

    public ProviderTraceContentItem? SelectedProviderTraceContent
    {
        get => _selectedProviderTraceContent;
        private set
        {
            if (SetProperty(ref _selectedProviderTraceContent, value))
            {
                OnPropertyChanged(nameof(SelectedProviderTraceTitle));
                OnPropertyChanged(nameof(HasSelectedProviderTraceContent));
                OnPropertyChanged(nameof(ShowSelectedProviderTraceOverview));
            }
        }
    }

    public ApprovalItem? PendingApproval
    {
        get => _pendingApproval;
        private set
        {
            if (SetProperty(ref _pendingApproval, value)) OnPropertyChanged(nameof(HasPendingApproval));
        }
    }

    public string ConnectionState
    {
        get => _connectionState;
        private set
        {
            if (SetProperty(ref _connectionState, value)) OnPropertyChanged(nameof(CanCompose));
        }
    }
    public string StatusText { get => _statusText; private set => SetProperty(ref _statusText, value); }
    public string ComposerText { get => _composerText; set { if (SetProperty(ref _composerText, value)) OnPropertyChanged(nameof(CanSubmit)); } }
    public string ActiveTurnId { get => _activeTurnId; private set { if (SetProperty(ref _activeTurnId, value)) { OnPropertyChanged(nameof(IsTurnActive)); OnPropertyChanged(nameof(CanSubmit)); OnPropertyChanged(nameof(CanCompose)); } } }
    public string ThemeMode { get => _themeMode; private set => SetProperty(ref _themeMode, value); }
    public string LogLevel { get => _logLevel; private set => SetProperty(ref _logLevel, value); }
    public string LogDirectory { get => _logDirectory; private set => SetProperty(ref _logDirectory, value); }
    public long LogMaxBytes { get => _logMaxBytes; private set => SetProperty(ref _logMaxBytes, value); }
    public int LogRetention { get => _logRetention; private set => SetProperty(ref _logRetention, value); }
    public string DiagnosticsText { get => _diagnosticsText; private set => SetProperty(ref _diagnosticsText, value); }
    public bool FullControlEnabled { get => _fullControlEnabled; private set => SetProperty(ref _fullControlEnabled, value); }
    public string GitState
    {
        get => _gitState;
        private set
        {
            if (!SetProperty(ref _gitState, value)) return;
            OnPropertyChanged(nameof(IsGitReady));
            OnPropertyChanged(nameof(IsGitClean));
            OnPropertyChanged(nameof(IsGitDirty));
            OnPropertyChanged(nameof(IsGitLoading));
            OnPropertyChanged(nameof(GitFooterBranchText));
            OnPropertyChanged(nameof(GitEmptyMessage));
        }
    }
    public string GitError { get => _gitError; private set => SetProperty(ref _gitError, value); }
    public string GitDiffState
    {
        get => _gitDiffState;
        private set
        {
            if (!SetProperty(ref _gitDiffState, value)) return;
            NotifyGitDiffPresentationChanged();
        }
    }
    public string GitDiffError { get => _gitDiffError; private set => SetProperty(ref _gitDiffError, value); }
    public string GitPatch { get => _gitPatch; private set => SetProperty(ref _gitPatch, value); }
    public string GitScope { get => _gitScope; private set => SetProperty(ref _gitScope, value); }
    public string GitFilter { get => _gitFilter; private set { if (SetProperty(ref _gitFilter, value)) OnPropertyChanged(nameof(GitEmptyMessage)); } }
    public string ProviderTraceState
    {
        get => _providerTraceState;
        private set
        {
            if (!SetProperty(ref _providerTraceState, value)) return;
            OnPropertyChanged(nameof(IsProviderTraceLoading));
            OnPropertyChanged(nameof(ProviderTraceEmptyMessage));
        }
    }
    public string ProviderTraceError { get => _providerTraceError; private set => SetProperty(ref _providerTraceError, value); }
    public string ProviderTraceFilter { get => _providerTraceFilter; private set { if (SetProperty(ref _providerTraceFilter, value)) OnPropertyChanged(nameof(ProviderTraceEmptyMessage)); } }
    public string SessionLoadError
    {
        get => _sessionLoadError;
        private set
        {
            if (!SetProperty(ref _sessionLoadError, value)) return;
            OnPropertyChanged(nameof(HasSessionLoadError));
            OnPropertyChanged(nameof(CanSubmit));
            OnPropertyChanged(nameof(CanCompose));
        }
    }
    public string GitBranch { get => _gitBranch; private set { if (SetProperty(ref _gitBranch, value)) { OnPropertyChanged(nameof(GitSummary)); OnPropertyChanged(nameof(GitFooterBranchText)); } } }
    public int GitChangedFiles { get => _gitChangedFiles; private set { if (SetProperty(ref _gitChangedFiles, value)) { OnPropertyChanged(nameof(GitSummary)); OnPropertyChanged(nameof(GitChangeSummary)); OnPropertyChanged(nameof(IsGitClean)); OnPropertyChanged(nameof(IsGitDirty)); } } }
    public int GitAdditions { get => _gitAdditions; private set => SetProperty(ref _gitAdditions, value); }
    public int GitDeletions { get => _gitDeletions; private set => SetProperty(ref _gitDeletions, value); }
    public int GitConflicts { get => _gitConflicts; private set => SetProperty(ref _gitConflicts, value); }
    public bool GitStatusTruncated { get => _gitStatusTruncated; private set => SetProperty(ref _gitStatusTruncated, value); }
    public bool GitDiffBinary { get => _gitDiffBinary; private set { if (SetProperty(ref _gitDiffBinary, value)) NotifyGitDiffPresentationChanged(); } }
    public bool GitDiffTruncated { get => _gitDiffTruncated; private set => SetProperty(ref _gitDiffTruncated, value); }
    public int GitDiffAdditions { get => _gitDiffAdditions; private set => SetProperty(ref _gitDiffAdditions, value); }
    public int GitDiffDeletions { get => _gitDiffDeletions; private set => SetProperty(ref _gitDiffDeletions, value); }
    public long SessionTotalTokens { get => _sessionTotalTokens; private set { if (SetProperty(ref _sessionTotalTokens, value)) OnPropertyChanged(nameof(SessionTokenText)); } }
    public bool NavigationVisible { get => _navigationVisible; set { if (SetProperty(ref _navigationVisible, value)) { OnPropertyChanged(nameof(NavigationWidth)); OnPropertyChanged(nameof(SessionSidebarVisible)); OnPropertyChanged(nameof(ExplorerSidebarVisible)); } } }
    public bool ExplorerVisible { get => _explorerVisible; set { if (SetProperty(ref _explorerVisible, value)) { OnPropertyChanged(nameof(SessionSidebarVisible)); OnPropertyChanged(nameof(ExplorerSidebarVisible)); } } }
    public bool ReviewVisible { get => _reviewVisible; set { if (SetProperty(ref _reviewVisible, value)) OnPropertyChanged(nameof(ReviewWidth)); } }
    public bool NavigationPinned { get => _navigationPinned; set => SetProperty(ref _navigationPinned, value); }
    public bool GitVisible { get => _gitVisible; set => SetProperty(ref _gitVisible, value); }
    public bool ProviderTraceVisible { get => _providerTraceVisible; set => SetProperty(ref _providerTraceVisible, value); }
    public double NavigationPaneWidth { get => _navigationPaneWidth; set { if (SetProperty(ref _navigationPaneWidth, value)) OnPropertyChanged(nameof(NavigationWidth)); } }
    public double ReviewPaneWidth { get => _reviewPaneWidth; set { if (SetProperty(ref _reviewPaneWidth, value)) OnPropertyChanged(nameof(ReviewWidth)); } }
    public double BottomDrawerHeight { get => _bottomDrawerHeight; set => SetProperty(ref _bottomDrawerHeight, value); }
    public bool IsBusy { get => _isBusy; private set => SetProperty(ref _isBusy, value); }
    public bool IsSessionLoading
    {
        get => _isSessionLoading;
        private set
        {
            if (!SetProperty(ref _isSessionLoading, value)) return;
            if (!value) IsSessionLoadingVisible = false;
            OnPropertyChanged(nameof(CanSubmit));
            OnPropertyChanged(nameof(CanCompose));
        }
    }
    public bool IsSessionLoadingVisible { get => _isSessionLoadingVisible; private set => SetProperty(ref _isSessionLoadingVisible, value); }

    public bool IsProjectOpen => SelectedProject is not null;
    public bool HasProjects => Projects.Count > 0;
    public bool HasProjectDependencies => ProjectDependencies.Count > 0;
    public bool SessionSidebarVisible => NavigationVisible && !ExplorerVisible;
    public bool ExplorerSidebarVisible => NavigationVisible && ExplorerVisible;
    public bool HasSessions => Sessions.Count > 0;
    public bool HasMessages => Messages.Count > 0;
    public bool HasActivities => Activities.Count > 0;
    public bool HasCheckpoints => Checkpoints.Count > 0;
    public bool HasFilteredGitFiles => FilteredGitFiles.Count > 0;
    public bool HasProviderTraces => ProviderTraces.Count > 0;
    public bool HasFilteredProviderTraces => FilteredProviderTraceTurns.Count > 0;
    public bool HasSelectedProviderTrace => SelectedProviderTraceDetails is not null;
    public bool HasSelectedProviderTraceContent => SelectedProviderTraceContent is not null;
    public bool ShowSelectedProviderTraceOverview => HasSelectedProviderTrace && !HasSelectedProviderTraceContent;
    public bool HasSessionLoadError => !string.IsNullOrWhiteSpace(SessionLoadError);
    public string GitFileCountText => $"{FilteredGitFiles.Count} {(FilteredGitFiles.Count == 1 ? "file" : "files")}";
    public string ProviderTraceCountText => $"{FilteredProviderTraceTurns.Count} turns · {FilteredProviderTraceTurns.Sum(turn => turn.Calls.Count)} calls";
    public bool HasSelectedSession => SelectedSession is not null;
    public bool HasPendingApproval => PendingApproval is not null;
    public bool IsTurnActive => !string.IsNullOrWhiteSpace(ActiveTurnId);
    public bool CanCompose => (ConnectionState == "connected" || IsSessionLoading) && SelectedSession is not null && SelectedModel?.Configured == true && !HasSessionLoadError;
    public bool CanSubmit => SelectedSession is not null && SelectedModel?.Configured == true && !string.IsNullOrWhiteSpace(ComposerText) && !IsTurnActive && !IsSessionLoading && !HasSessionLoadError;
    public string ProjectTitle => SelectedProject?.DisplayName ?? "SunCode";
    public string SessionTitle => SelectedSession?.DisplayTitle ?? "No session selected";
    public string SessionTokenText => $"Session {CompactNumber(SessionTotalTokens)} tokens";
    public string SelectedModelName => SelectedModel?.Id ?? string.Empty;
    public string LatestActivityText => Activities.LastOrDefault()?.Text ?? "No tool activity yet";
    public string ComposerPlaceholder => SelectedSession is null
        ? "Create a session first..."
        : SelectedModel is null
            ? "Choose a model first..."
            : SelectedModel.Configured ? "Tell SunCode what to do..." : "Store the selected provider's API key first...";
    public string GitSummary => GitState == "not_repository"
        ? "Not a Git repository"
        : string.IsNullOrWhiteSpace(GitBranch) ? "Git status unavailable" : $"{GitBranch}  ·  {GitChangedFiles} changed";
    public GridLength NavigationWidth => NavigationVisible ? new GridLength(NavigationPaneWidth) : new GridLength(0);
    public GridLength ReviewWidth => ReviewVisible ? new GridLength(ReviewPaneWidth) : new GridLength(0);
    public GridLength GitFileListWidth => new(Math.Min(300, Math.Max(228, (_layoutWidth - 80) * 0.28)));
    public bool IsGitReady => GitState == "ready";
    public bool IsGitClean => IsGitReady && GitChangedFiles == 0;
    public bool IsGitDirty => IsGitReady && GitChangedFiles > 0;
    public bool IsGitLoading => GitState == "loading" || GitDiffState == "loading";
    public bool HasGitDiffLines => GitDiffState == "ready" && !GitDiffBinary && DiffLines.Count > 0;
    public bool ShowGitDiffEmpty => !HasGitDiffLines;
    public bool ShowGitDiffStats => GitDiffState == "ready" && !GitDiffBinary;
    public string SelectedGitPath => SelectedGitFile?.Path ?? "No file selected";
    public string GitFooterBranchText => GitState switch
    {
        "loading" => "Reading Git...",
        "not_repository" => "Not a Git repository",
        "error" => "Git unavailable",
        _ => string.IsNullOrWhiteSpace(GitBranch) ? "Detached HEAD" : GitBranch
    };
    public string GitChangeSummary => GitChangedFiles == 0 ? "Clean" : $"{GitChangedFiles} changed";
    public string GitEmptyMessage
    {
        get
        {
            if (GitState == "loading") return "Reading repository changes...";
            if (GitState == "not_repository") return "This project is not inside a Git repository.";
            if (GitState == "error") return string.IsNullOrWhiteSpace(GitError) ? "Git status is unavailable." : GitError;
            if (FilteredGitFiles.Count == 0 && GitFilter.Length > 0) return "No changed files match this filter.";
            if (FilteredGitFiles.Count == 0 && GitState == "ready") return GitScope == "all" ? "Working tree clean." : $"No {GitScope} changes.";
            if (GitDiffState == "loading") return "Loading diff...";
            if (GitDiffState == "error") return string.IsNullOrWhiteSpace(GitDiffError) ? "This diff is unavailable." : GitDiffError;
            if (GitDiffBinary) return "Binary files cannot be displayed as text.";
            return "Select a changed file to inspect its diff.";
        }
    }

    public bool IsProviderTraceLoading => ProviderTraceState == "loading";
    public string ProviderTraceSummary => ProviderTraces.Count == 0
        ? "No provider requests"
        : $"{ProviderTraces.Count} provider {(ProviderTraces.Count == 1 ? "request" : "requests")}";
    public string SelectedProviderTraceTitle => SelectedProviderTraceContent is { } content
        ? $"{SelectedProviderTraceDetails?.Title ?? SelectedProviderTrace?.Title} · {content.Title}"
        : SelectedProviderTraceDetails?.Title ?? SelectedProviderTrace?.Title ?? "No model call selected";
    public string ProviderTraceEmptyMessage
    {
        get
        {
            if (ProviderTraceState == "loading") return "Loading session trace...";
            if (ProviderTraceState == "error") return string.IsNullOrWhiteSpace(ProviderTraceError) ? "Provider trace is unavailable." : ProviderTraceError;
            if (SelectedSession is null) return "Select a session to inspect provider requests.";
            if (FilteredProviderTraceTurns.Count == 0 && ProviderTraceFilter.Length > 0) return "No turns or model calls match this filter.";
            if (FilteredProviderTraceTurns.Count == 0) return "No turns have been recorded for this session.";
            return "Select a model call to inspect its messages, tools, request, response, and usage.";
        }
    }

    public bool ScopeAll => GitScope == "all";
    public bool ScopeStaged => GitScope == "staged";
    public bool ScopeUnstaged => GitScope == "unstaged";

    public void UpdateLayoutWidth(double width)
    {
        _layoutWidth = width;
        NavigationPaneWidth = Math.Clamp(NavigationPaneWidth, 180, Math.Min(420, Math.Max(180, _layoutWidth - 560)));
        ReviewPaneWidth = Math.Clamp(ReviewPaneWidth, 220, Math.Min(460, Math.Max(220, _layoutWidth - 560)));
        OnPropertyChanged(nameof(NavigationWidth));
        OnPropertyChanged(nameof(ReviewWidth));
        OnPropertyChanged(nameof(GitFileListWidth));
    }

    public async Task InitializeAsync()
    {
        if (_sdk is not null || _disposed) return;
        ConnectionState = "connecting";
        StatusText = "Starting local runtime...";
        try
        {
            _sdk = await RuntimeSdk.OpenAsync();
            await _sdk.HealthAsync();
            ConnectionState = "connected";
            StatusText = "Connected to local runtime";
            await LoadModelsAsync();
            await LoadSettingsAsync();
            await LoadCredentialsAsync();
            await LoadProjectsAsync();
            await RefreshDiagnosticsAsync();
        }
        catch (Exception exception)
        {
            ReportError(exception);
        }
    }

    public async Task OpenProjectAsync(string path)
    {
        if (!EnsureSdk() || string.IsNullOrWhiteSpace(path)) return;
        await RunAsync(async () =>
        {
            var opened = await _sdk!.OpenProjectAsync(path);
            await LoadProjectsAsync();
            var projectId = opened.String("projectId");
            var project = Projects.FirstOrDefault(item => item.ProjectId == projectId);
            if (project is not null) await SelectProjectAsync(project);
        }, "Project opened");
    }

    public async Task<ProjectItem?> RegisterProjectAsync(string path)
    {
        if (!EnsureSdk() || string.IsNullOrWhiteSpace(path)) return null;
        IsBusy = true;
        try
        {
            var opened = await _sdk!.OpenProjectAsync(path);
            await LoadProjectsAsync();
            var project = Projects.FirstOrDefault(item => item.ProjectId == opened.String("projectId"));
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
        if (!EnsureSdk() || SelectedProject?.ProjectId == project.ProjectId) return;
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

    public void BackToProjects()
    {
        CloseSubscription();
        ClearSession();
        SelectedProject = null;
        Sessions.Clear();
        ProjectDependencies.Clear();
        ExplorerRoots.Clear();
        ClearGit();
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

    public async Task RenameSessionAsync(string title)
    {
        if (!EnsureSdk() || SelectedSession is null || string.IsNullOrWhiteSpace(title)) return;
        await RunAsync(async () =>
        {
            await _sdk!.RenameSessionAsync(SelectedSession.SessionId, title.Trim());
            await LoadSessionsAsync(SelectedSession.SessionId);
        }, "Session renamed");
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

    public async Task ArchiveSessionAsync()
    {
        if (!EnsureSdk() || SelectedSession is null) return;
        var sessionId = SelectedSession.SessionId;
        await RunAsync(async () =>
        {
            CloseSubscription();
            await _sdk!.ArchiveSessionAsync(sessionId);
            ClearSession();
            await LoadSessionsAsync();
        }, "Session archived");
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
        ComposerText = string.Empty;
        await RunAsync(async () =>
        {
            var result = await _sdk!.SubmitTurnAsync(SelectedSession.SessionId, text, SelectedModel.Id);
            StatusText = result.String("status") switch
            {
                "queued" => "Message queued for this turn",
                "awaiting_approval" => "Turn is awaiting approval",
                _ => "Turn submitted"
            };
        });
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
                : $"Runtime  {health.String("runtime")}\nDatabase  {(database.Bool("ok") ? "Ready" : "Check required")}";
            OnPropertyChanged(nameof(IsRuntimeHealthy));
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
            GitConflicts = status.Int("conflicts");
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

    public Task RefreshProviderTracesAsync() => RefreshProviderTracesAsync(null, null);

    private async Task RefreshProviderTracesAsync(string? requestedSessionId, long? loadVersion)
    {
        if (!EnsureSdk() || SelectedSession is null)
        {
            ClearProviderTraces();
            return;
        }
        var sessionId = requestedSessionId ?? SelectedSession.SessionId;
        if (!IsSessionContextCurrent(sessionId, loadVersion)) return;
        ProviderTraceState = "loading";
        ProviderTraceError = string.Empty;
        try
        {
            var result = await _sdk!.ListProviderExchangesAsync(sessionId);
            if (!IsSessionContextCurrent(sessionId, loadVersion)) return;
            ProviderTraces.Clear();
            ProviderTraceTurns.Clear();
            _providerTraceDetails.Clear();
            _providerTraceDetailLoads.Clear();
            var exchanges = result.Array("exchanges").OfType<JsonObject>().Select(ProviderTraceFromJson).ToList();
            foreach (var exchange in exchanges) ProviderTraces.Add(exchange);
            var turnValues = result.Array("turns").OfType<JsonObject>().ToList();
            for (var index = 0; index < turnValues.Count; index++)
            {
                var item = turnValues[index];
                var turnId = item.String("turnId", "turn_id");
                var calls = exchanges.Where(call => call.TurnId == turnId).OrderBy(call => call.Iteration).ThenBy(call => call.StartedAt).ToList();
                ProviderTraceTurns.Add(ProviderTraceTurnFromJson(item, turnValues.Count - index, calls));
            }
            ApplyProviderTraceFilter();
            ProviderTraceState = "ready";
            if (SelectedProviderTrace is { } selected)
            {
                await LoadProviderTraceAsync(selected);
            }
        }
        catch (Exception exception)
        {
            if (!IsSessionContextCurrent(sessionId, loadVersion)) return;
            ProviderTraceState = "error";
            ProviderTraceError = exception.Message;
        }
    }

    public async Task LoadProviderTraceAsync(ProviderTraceItem trace)
    {
        if (!EnsureSdk() || SelectedSession is null) return;
        var sessionId = SelectedSession.SessionId;
        SelectedProviderTrace = trace;
        SelectedProviderTraceContent = null;
        if (_providerTraceDetails.TryGetValue(trace.ExchangeId, out var cached))
        {
            SelectedProviderTraceDetails = cached;
            PopulateProviderTraceContents(trace, cached);
            return;
        }
        ProviderTraceState = "loading";
        ProviderTraceError = string.Empty;
        try
        {
            var details = await GetProviderTraceDetailsAsync(sessionId, trace.ExchangeId);
            if (SelectedSession?.SessionId != sessionId || SelectedProviderTrace?.ExchangeId != trace.ExchangeId) return;
            PopulateProviderTraceContents(trace, details);
            SelectedProviderTraceDetails = details;
            ProviderTraceState = "ready";
        }
        catch (Exception exception)
        {
            if (SelectedSession?.SessionId != sessionId || SelectedProviderTrace?.ExchangeId != trace.ExchangeId) return;
            ProviderTraceState = "error";
            ProviderTraceError = exception.Message;
        }
    }

    public async Task LoadProviderTraceContentsAsync(ProviderTraceItem trace)
    {
        if (trace.ContentsLoaded || trace.ContentsLoading || !EnsureSdk() || SelectedSession is null) return;
        trace.ContentsLoading = true;
        trace.Contents.Clear();
        trace.Contents.Add(ProviderTraceContentItem.Placeholder("Loading call contents..."));
        try
        {
            ProviderTraceItem details;
            if (!_providerTraceDetails.TryGetValue(trace.ExchangeId, out details!))
            {
                var sessionId = SelectedSession.SessionId;
                details = await GetProviderTraceDetailsAsync(sessionId, trace.ExchangeId);
                if (SelectedSession?.SessionId != sessionId) return;
            }
            PopulateProviderTraceContents(trace, details);
            if (SelectedProviderTrace?.ExchangeId == trace.ExchangeId)
                SelectedProviderTraceDetails = details;
        }
        catch (Exception exception)
        {
            trace.Contents.Clear();
            trace.Contents.Add(ProviderTraceContentItem.Placeholder("Call contents are unavailable"));
            ProviderTraceError = exception.Message;
        }
        finally
        {
            trace.ContentsLoading = false;
        }
    }

    public async Task SelectProviderTraceContentAsync(ProviderTraceContentItem content)
    {
        if (content.IsPlaceholder) return;
        var trace = ProviderTraces.FirstOrDefault(item => item.ExchangeId == content.ExchangeId);
        if (trace is null) return;
        if (!_providerTraceDetails.TryGetValue(trace.ExchangeId, out var details))
        {
            await LoadProviderTraceContentsAsync(trace);
            if (!_providerTraceDetails.TryGetValue(trace.ExchangeId, out details)) return;
        }
        SelectedProviderTrace = trace;
        SelectedProviderTraceDetails = details;
        SelectedProviderTraceContent = content;
    }

    public void SetProviderTraceFilter(string filter)
    {
        ProviderTraceFilter = filter ?? string.Empty;
        ApplyProviderTraceFilter();
    }

    public void SelectProviderTraceTurn()
    {
        SelectedProviderTrace = null;
        if (ProviderTraceState == "loading") ProviderTraceState = "ready";
    }

    public void SetGitScope(string scope)
    {
        GitScope = scope is "staged" or "unstaged" ? scope : "all";
        OnPropertyChanged(nameof(ScopeAll));
        OnPropertyChanged(nameof(ScopeStaged));
        OnPropertyChanged(nameof(ScopeUnstaged));
        ApplyGitFilter();
        if (SelectedGitFile is not null && FilteredGitFiles.Contains(SelectedGitFile))
            _ = LoadGitDiffAsync(SelectedGitFile, GitScope);
    }

    public void SetGitFilter(string filter)
    {
        GitFilter = filter ?? string.Empty;
        ApplyGitFilter();
        if (SelectedGitFile is not null) _ = LoadGitDiffAsync(SelectedGitFile, GitScope);
    }

    public bool IsRuntimeHealthy => DiagnosticsText.Contains("Ready", StringComparison.Ordinal);

    public async Task SaveCredentialAsync(string provider, string apiKey)
    {
        if (!EnsureSdk() || string.IsNullOrWhiteSpace(provider) || string.IsNullOrWhiteSpace(apiKey)) return;
        await RunAsync(async () =>
        {
            await _sdk!.SetCredentialAsync(provider, apiKey.Trim());
            await LoadCredentialsAsync();
            await LoadModelsAsync();
        }, "Credential stored");
    }

    public async Task RemoveCredentialAsync(string provider)
    {
        if (!EnsureSdk() || string.IsNullOrWhiteSpace(provider)) return;
        await RunAsync(async () =>
        {
            await _sdk!.RemoveCredentialAsync(provider);
            await LoadCredentialsAsync();
            await LoadModelsAsync();
        }, "Credential removed");
    }

    public async Task SaveDefaultModelAsync(ModelItem model)
    {
        if (!EnsureSdk()) return;
        await RunAsync(async () =>
        {
            await _sdk!.SetSettingAsync("default_model", model.Id);
            SelectedModel = model;
        }, "Default model saved");
    }

    public async Task SaveThemeAsync(string mode)
    {
        if (!EnsureSdk() || mode is not ("dark" or "light")) return;
        await RunAsync(async () =>
        {
            await _sdk!.SetSettingAsync("theme_mode", mode);
            SetTheme(mode);
        }, "Theme saved");
    }

    public async Task<bool> SaveLoggingSettingsAsync(
        string level,
        string? directory,
        string maxBytesText,
        string retentionText)
    {
        if (!EnsureSdk()) return false;

        level = level.Trim().ToUpperInvariant();
        directory = directory?.Trim() ?? string.Empty;
        if (level is not ("TRACE" or "DEBUG" or "INFO" or "WARN" or "ERROR" or "OFF"))
        {
            StatusText = "Choose a valid logging level";
            return false;
        }
        if (!long.TryParse(maxBytesText.Trim(), NumberStyles.Integer, CultureInfo.InvariantCulture, out var maxBytes)
            || maxBytes < 1024)
        {
            StatusText = "Maximum log size must be at least 1024 bytes";
            return false;
        }
        if (!int.TryParse(retentionText.Trim(), NumberStyles.Integer, CultureInfo.InvariantCulture, out var retention)
            || retention is < 0 or > 100)
        {
            StatusText = "Log retention must be between 0 and 100 files";
            return false;
        }

        IsBusy = true;
        var sdk = _sdk!;
        try
        {
            await sdk.SetSettingAsync("log_level", level);
            await sdk.SetSettingAsync("log_directory", directory);
            await sdk.SetSettingAsync("log_max_bytes", maxBytes);
            await sdk.SetSettingAsync("log_retention", retention);
            LogLevel = level;
            LogDirectory = directory;
            LogMaxBytes = maxBytes;
            LogRetention = retention;
            DiagnosticLog.Configure(level, directory, maxBytes, retention);
            StatusText = "Logging settings saved";
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

    public bool IsProviderConfigured(string provider) =>
        Credentials.Any(item => item.Provider == provider && item.Configured);

    public string ProviderModels(string provider)
    {
        var models = Models.Where(item => item.Provider == provider).Select(item => item.Display).ToArray();
        return models.Length == 0 ? "No models available" : string.Join(Environment.NewLine, models);
    }

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
        Sessions.Clear();
        foreach (var item in result.Array("sessions").OfType<JsonObject>())
        {
            Sessions.Add(new SessionItem(item.String("sessionId"), item.String("title"), item.String("lastActivityAt"), !string.IsNullOrWhiteSpace(item.String("pinAt", "pin_at"))));
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
        Models.Clear();
        foreach (var item in result.Array("models").OfType<JsonObject>())
        {
            Models.Add(new ModelItem(item.String("id"), item.String("provider"), item.String("availability")));
        }
        SelectedModel = Models.FirstOrDefault(item => item.Id == selectedId) ?? Models.FirstOrDefault();
    }

    private async Task LoadCredentialsAsync()
    {
        if (_sdk is null) return;
        var result = await _sdk.ListCredentialsAsync();
        Credentials.Clear();
        foreach (var item in result.Array("credentials").OfType<JsonObject>())
        {
            Credentials.Add(new CredentialItem(item.String("provider"), item.Bool("configured")));
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
        var retention = LongSetting("log_retention", 5);
        var configuredLevel = StringSetting("log_level", "INFO").Trim().ToUpperInvariant();
        LogLevel = configuredLevel is "TRACE" or "DEBUG" or "INFO" or "WARN" or "ERROR" or "OFF"
            ? configuredLevel
            : "INFO";
        LogDirectory = StringSetting("log_directory", string.Empty);
        var maxBytes = LongSetting("log_max_bytes", 10 * 1024 * 1024);
        LogMaxBytes = maxBytes >= 1024 ? maxBytes : 10 * 1024 * 1024;
        LogRetention = retention is >= 0 and <= 100 ? (int)retention : 5;
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
    }

    internal static SessionSnapshotProjection ProjectSnapshot(JsonObject snapshot)
    {
        var messages = new List<MessageItem>();
        var activities = new List<ActivityItem>();
        var changedPaths = new List<string>();
        var changedPathSet = new HashSet<string>(StringComparer.Ordinal);
        ApprovalItem? pendingApproval = null;
        var activeTurnId = string.Empty;

        var conversationTurns = snapshot.Array("conversationTurns").OfType<JsonObject>().ToArray();
        if (conversationTurns.Length > 0)
        {
            foreach (var turn in conversationTurns)
            {
                var turnId = turn.String("turnId", "turn_id");
                var state = turn.String("state");
                if (!IsTerminalTurnState(state)) activeTurnId = turnId;
                var toolUses = turn.Array("toolUses").OfType<JsonObject>().ToArray();
                var toolsById = toolUses
                    .Where(item => item.String("toolCallId", "tool_call_id").Length > 0)
                    .ToDictionary(item => item.String("toolCallId", "tool_call_id"), StringComparer.Ordinal);
                var projectedToolIds = new HashSet<string>(StringComparer.Ordinal);
                foreach (var item in turn.Array("messages").OfType<JsonObject>()
                    .OrderBy(item => item.String("createdAt", "created_at"), StringComparer.Ordinal))
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
                            IsProcess = role == "assistant",
                            CanBeFinalAssistant = role == "assistant" && message.Array("tool_calls").Count == 0
                        });
                    }

                    foreach (var call in message.Array("tool_calls").OfType<JsonObject>())
                    {
                        var toolCallId = call.String("call_id", "toolCallId", "tool_call_id");
                        if (toolCallId.Length == 0 || !projectedToolIds.Add(toolCallId) ||
                            !toolsById.TryGetValue(toolCallId, out var toolUse)) continue;
                        messages.Add(ToolMessageItem(toolUse, turnId, messages.Count + 1));
                    }
                }
                foreach (var item in toolUses
                    .Where(item => !projectedToolIds.Contains(item.String("toolCallId", "tool_call_id")))
                    .OrderBy(item => item.String("createdAt", "created_at"), StringComparer.Ordinal)
                    .ThenBy(item => item.Int("ordinal")))
                {
                    messages.Add(ToolMessageItem(item, turnId, messages.Count + 1));
                }
                ConfigureTurnPresentation(messages, turnId, state, expanded: !IsTerminalTurnState(state));
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

            foreach (var path in new[] { payload.String("path"), payload.String("from"), payload.String("to") })
            {
                if (path.Length > 0 && changedPathSet.Add(path)) changedPaths.Add(path);
            }

            if (type == "approval.requested") pendingApproval = ApprovalItem.FromPayload(payload);
            if (type == "approval.resolved") pendingApproval = null;
            if (type == "turn.state")
            {
                var state = payload.String("state");
                activeTurnId = state is "completed" or "failed" or "cancelled" or "interrupted"
                    ? string.Empty
                    : payload.String("turn_id");
            }
        }

        return new SessionSnapshotProjection(messages, activities, changedPaths, pendingApproval, activeTurnId);
    }

    internal void ApplySnapshot(SessionSnapshotProjection projection)
    {
        _appliedMessageIds.Clear();
        foreach (var message in projection.Messages)
        {
            if (message.MessageId.Length > 0) _appliedMessageIds.Add(message.MessageId);
        }
        Messages = new BulkObservableCollection<MessageItem>(projection.Messages);
        Activities.ReplaceAll(projection.Activities);
        ChangedPaths.ReplaceAll(projection.ChangedPaths);
        PendingApproval = projection.PendingApproval;
        ActiveTurnId = projection.ActiveTurnId;
        OnPropertyChanged(nameof(HasMessages));
        OnPropertyChanged(nameof(HasActivities));
        OnPropertyChanged(nameof(LatestActivityText));
        ConversationChanged?.Invoke();
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
            StatusText = $"Ignored malformed runtime event: {exception.Message}";
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
                    Messages.Add(new MessageItem
                    {
                        Role = "assistant",
                        Text = delta,
                        ContentSequence = Messages.Count + 1,
                        TurnId = turnId,
                        Streaming = true,
                        IsProcess = true,
                        CanBeFinalAssistant = false
                    });
                    ConversationChanged?.Invoke();
                }
            }
            else if (delta.Length > 0)
            {
                assistant.Text += delta;
                ConversationChanged?.Invoke();
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
                    Messages.Add(new MessageItem
                    {
                        MessageId = messageId,
                        Role = type == "message.user" ? "user" : "assistant",
                        Text = text,
                        ContentSequence = Messages.Count + 1,
                        TurnId = turnId,
                        IsProcess = type == "message.assistant",
                        CanBeFinalAssistant = canBeFinalAssistant
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
                ConversationChanged?.Invoke();
            }
        }
        else if (type is "tool.requested" or "tool.state" or "tool.result")
        {
            ApplyToolEvent(payload, type);
            Activities.Add(new ActivityItem(type, text, Activities.Count + 1, payload.String("state"), payload.String("name")));
            OnPropertyChanged(nameof(HasActivities));
            OnPropertyChanged(nameof(LatestActivityText));
            ConversationChanged?.Invoke();
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
                pathAdded = true;
            }
        }
        if (type == "approval.requested") PendingApproval = ApprovalItem.FromPayload(payload);
        if (type == "approval.resolved")
        {
            if (payload.String("decision") == "allow_session") FullControlEnabled = true;
            PendingApproval = null;
        }
        if (type == "turn.state")
        {
            var state = payload.String("state");
            var turnId = payload.String("turn_id");
            ActiveTurnId = IsTerminalTurnState(state) ? string.Empty : turnId;
            ConfigureTurnPresentation(Messages, turnId, state, expanded: !IsTerminalTurnState(state));
            ConversationChanged?.Invoke();
        }
        if (live && type.StartsWith("checkpoint.", StringComparison.Ordinal)) _ = LoadCheckpointsAsync();
        if (live && type == "usage.updated") _ = LoadSessionUsageAsync();
        if (live && type.StartsWith("provider.exchange.", StringComparison.Ordinal) && ProviderTraceVisible) _ = RefreshProviderTracesAsync();
        if (live && (type.StartsWith("checkpoint.", StringComparison.Ordinal) || pathAdded)) _ = RefreshGitAsync();
    }

    public void ToggleTurnProcess(MessageItem toggleItem)
    {
        if (!toggleItem.ShowProcessToggle) return;
        toggleItem.ProcessExpanded = !toggleItem.ProcessExpanded;
        foreach (var item in Messages.Where(item =>
            item.TurnId == toggleItem.TurnId && item.IsProcess))
        {
            item.IsVisible = item == toggleItem || toggleItem.ProcessExpanded;
            item.ProcessContentVisible = toggleItem.ProcessExpanded;
        }
    }

    private void ApplyToolEvent(JsonObject payload, string eventType)
    {
        var turnId = payload.String("turn_id");
        var toolCallId = payload.String("tool_call_id");
        if (turnId.Length == 0 || toolCallId.Length == 0) return;
        var existing = Messages.LastOrDefault(message =>
            message.IsTool && message.TurnId == turnId && message.ToolCallId == toolCallId);
        var state = eventType == "tool.state"
            ? payload.String("state")
            : existing?.ToolState ?? "requested";
        var name = payload.String("name");
        if (name.Length == 0) name = existing?.ToolName ?? "tool";
        var request = eventType == "tool.requested"
            ? Pretty(payload["arguments"])
            : existing?.ToolRequest ?? string.Empty;
        var result = eventType == "tool.result"
            ? Pretty(payload["result"])
            : existing?.ToolResult ?? string.Empty;
        var error = eventType == "tool.state"
            ? payload.String("reason")
            : existing?.ToolError ?? string.Empty;
        var replacement = new MessageItem
        {
            Role = "tool",
            Kind = "tool",
            Text = name,
            ContentSequence = existing?.ContentSequence ?? Messages.Count + 1,
            TurnId = turnId,
            ToolCallId = toolCallId,
            ToolName = name,
            ToolState = state,
            ToolDetail = result.Length > 0 ? result : request,
            ToolRequest = request,
            ToolResult = result,
            ToolError = error,
            IsProcess = true
        };
        if (existing is null) Messages.Add(replacement);
        else Messages[Messages.IndexOf(existing)] = replacement;
    }

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
        IsProcess = true
    };

    private static void ConfigureTurnPresentation(
        IEnumerable<MessageItem> source,
        string turnId,
        string state,
        bool expanded)
    {
        var turnItems = source.Where(item => item.TurnId == turnId).ToArray();
        var assistants = turnItems.Where(item => item.Role == "assistant").ToArray();
        foreach (var assistant in assistants)
        {
            assistant.IsFinalAssistant = false;
            assistant.IsProcess = true;
        }
        foreach (var item in turnItems)
        {
            item.ShowProcessToggle = false;
            item.ProcessContentVisible = item.IsProcess;
        }
        var terminal = IsTerminalTurnState(state);
        if (!terminal)
        {
            foreach (var item in turnItems) item.IsVisible = true;
            return;
        }

        var final = assistants.LastOrDefault(item => item.CanBeFinalAssistant);
        if (final is null)
        {
            foreach (var item in turnItems) item.IsVisible = true;
            return;
        }

        final.IsFinalAssistant = true;
        final.IsProcess = false;
        var processItems = turnItems.Where(item => item.IsProcess).ToArray();
        var toggleItem = processItems.FirstOrDefault();
        if (toggleItem is not null)
        {
            toggleItem.ShowProcessToggle = true;
            toggleItem.ProcessItemCount = processItems.Length;
            toggleItem.ProcessExpanded = expanded;
        }
        foreach (var item in processItems)
        {
            item.IsVisible = item == toggleItem || expanded;
            item.ProcessContentVisible = expanded;
        }
        final.IsVisible = true;
    }

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
            "checkpoint.captured" => $"Checkpoint captured for {payload.String("path")}",
            "checkpoint.restore_failed" => "Undo stopped because a file changed outside SunCode",
            "turn.state" => $"Turn {payload.String("state")}",
            "assistant.delta" => payload.String("text"),
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

    private async Task RunAsync(Func<Task> operation, string? success = null)
    {
        IsBusy = true;
        try
        {
            await operation();
            if (success is not null) StatusText = success;
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

    private async Task RunAsync(Func<Task<JsonObject>> operation, string? success = null) =>
        await RunAsync(async () => { await operation(); }, success);

    private bool EnsureSdk()
    {
        if (_sdk is not null && !_disposed) return true;
        StatusText = "Runtime SDK is not connected";
        ConnectionState = "error";
        return false;
    }

    private void ReportError(Exception exception)
    {
        ConnectionState = "error";
        StatusText = exception.Message;
    }

    private void SetTheme(string mode)
    {
        ThemeMode = mode;
        ThemeChanged?.Invoke(mode);
    }

    private void CloseSubscription()
    {
        var hadSubscription = _subscription is not null;
        if (hadSubscription) DiagnosticLog.Debug("session", "close_subscription.dispose.begin");
        _subscription?.Dispose();
        _subscription = null;
        if (hadSubscription) DiagnosticLog.Debug("session", "close_subscription.dispose.end");
    }

    private void ClearSession(bool clearSelection = true)
    {
        Interlocked.Increment(ref _sessionLoadVersion);
        _loadedSessionId = null;
        IsSessionLoading = false;
        SessionLoadError = string.Empty;
        Messages = [];
        _appliedMessageIds.Clear();
        Activities.Clear();
        ChangedPaths.Clear();
        Checkpoints.Clear();
        DiffLines.Clear();
        ClearProviderTraces();
        PendingApproval = null;
        FullControlEnabled = false;
        ActiveTurnId = string.Empty;
        SessionTotalTokens = 0;
        if (clearSelection) SelectedSession = null;
        OnPropertyChanged(nameof(HasActivities));
        OnPropertyChanged(nameof(HasCheckpoints));
    }

    private bool IsCurrentSessionLoad(string sessionId, long loadVersion) =>
        !_disposed
        && _sessionLoadVersion == loadVersion
        && SelectedSession?.SessionId == sessionId;

    private async Task RevealSessionLoadingAsync(string sessionId, long loadVersion)
    {
        await Task.Delay(120);
        if (IsSessionLoading && IsCurrentSessionLoad(sessionId, loadVersion))
        {
            IsSessionLoadingVisible = true;
            LogSession("loading", sessionId, $"visible=true version={loadVersion}");
        }
    }

    private bool IsSessionContextCurrent(string sessionId, long? loadVersion) =>
        !_disposed
        && SelectedSession?.SessionId == sessionId
        && (loadVersion is null || _sessionLoadVersion == loadVersion.Value);

    private string DescribeSessionContext() =>
        $"selected={SelectedSession?.SessionId ?? "<none>"},loaded={_loadedSessionId ?? "<none>"},version={_sessionLoadVersion},loading={IsSessionLoading}";

    private static void LogSession(string operationId, string sessionId, string message) =>
        DiagnosticLog.Write(SessionLogLevel(operationId, message), "session", $"op={operationId} session={sessionId} {message}");

    private static DiagnosticLogLevel SessionLogLevel(string operationId, string message)
    {
        if (message.Contains("failed", StringComparison.OrdinalIgnoreCase)) return DiagnosticLogLevel.Error;
        if (message.Contains("discard", StringComparison.OrdinalIgnoreCase)
            || message.Contains("ignored", StringComparison.OrdinalIgnoreCase)
            || message.Contains("stale", StringComparison.OrdinalIgnoreCase)
            || message.Contains("resync", StringComparison.OrdinalIgnoreCase)) return DiagnosticLogLevel.Warn;
        if (operationId == "event") return DiagnosticLogLevel.Trace;
        if (message.Contains(".begin", StringComparison.Ordinal)
            || message.Contains(".end", StringComparison.Ordinal)
            || message.Contains(".completed", StringComparison.Ordinal)
            || message.Contains(".selected", StringComparison.Ordinal)
            || message.Contains("visible=", StringComparison.Ordinal)) return DiagnosticLogLevel.Debug;
        return DiagnosticLogLevel.Info;
    }

    private void ClearGit()
    {
        GitFiles.Clear();
        FilteredGitFiles.Clear();
        DiffLines.Clear();
        SelectedGitFile = null;
        GitState = "idle";
        GitError = string.Empty;
        GitDiffState = "idle";
        GitDiffError = string.Empty;
        GitPatch = string.Empty;
        GitBranch = string.Empty;
        GitChangedFiles = 0;
        GitAdditions = 0;
        GitDeletions = 0;
        GitConflicts = 0;
        GitStatusTruncated = false;
        GitDiffBinary = false;
        GitDiffTruncated = false;
        GitDiffAdditions = 0;
        GitDiffDeletions = 0;
    }

    private void ClearProviderTraces()
    {
        ProviderTraces.Clear();
        ProviderTraceTurns.Clear();
        FilteredProviderTraceTurns.Clear();
        SelectedProviderTrace = null;
        SelectedProviderTraceDetails = null;
        SelectedProviderTraceContent = null;
        _providerTraceDetails.Clear();
        _providerTraceDetailLoads.Clear();
        ProviderTraceState = "idle";
        ProviderTraceError = string.Empty;
        ProviderTraceFilter = string.Empty;
        OnPropertyChanged(nameof(HasProviderTraces));
        OnPropertyChanged(nameof(HasFilteredProviderTraces));
        OnPropertyChanged(nameof(ProviderTraceCountText));
        OnPropertyChanged(nameof(ProviderTraceSummary));
        OnPropertyChanged(nameof(ProviderTraceEmptyMessage));
    }

    private void ApplyGitFilter()
    {
        var selectedPath = SelectedGitFile?.Path;
        FilteredGitFiles.Clear();
        foreach (var file in GitFiles.Where(file =>
                     (GitScope == "all" || GitScope == "staged" && file.Staged || GitScope == "unstaged" && file.Unstaged) &&
                     (GitFilter.Length == 0 || file.Path.Contains(GitFilter, StringComparison.OrdinalIgnoreCase))))
        {
            FilteredGitFiles.Add(file);
        }
        SelectedGitFile = FilteredGitFiles.FirstOrDefault(file => file.Path == selectedPath)
            ?? FilteredGitFiles.FirstOrDefault();
        OnPropertyChanged(nameof(HasFilteredGitFiles));
        OnPropertyChanged(nameof(GitFileCountText));
        OnPropertyChanged(nameof(GitEmptyMessage));
    }

    private void ApplyProviderTraceFilter()
    {
        var selectedId = SelectedProviderTrace?.ExchangeId;
        FilteredProviderTraceTurns.Clear();
        foreach (var turn in ProviderTraceTurns)
        {
            var turnMatches = ProviderTraceTurnMatches(turn, ProviderTraceFilter);
            var calls = turnMatches
                ? turn.Calls
                : turn.Calls.Where(trace => ProviderTraceMatches(trace, ProviderTraceFilter)).ToList();
            if (turnMatches || calls.Count > 0)
            {
                FilteredProviderTraceTurns.Add(turn with { Calls = calls });
            }
        }
        var visibleCalls = FilteredProviderTraceTurns.SelectMany(turn => turn.Calls).ToList();
        SelectedProviderTrace = visibleCalls.FirstOrDefault(item => item.ExchangeId == selectedId)
            ?? visibleCalls.FirstOrDefault();
        OnPropertyChanged(nameof(HasProviderTraces));
        OnPropertyChanged(nameof(HasFilteredProviderTraces));
        OnPropertyChanged(nameof(ProviderTraceCountText));
        OnPropertyChanged(nameof(ProviderTraceSummary));
        OnPropertyChanged(nameof(ProviderTraceEmptyMessage));
    }

    private static bool ProviderTraceMatches(ProviderTraceItem trace, string filter)
    {
        if (string.IsNullOrWhiteSpace(filter)) return true;
        return trace.ExchangeId.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || trace.TurnId.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || trace.Provider.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || trace.ModelId.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || trace.WireModel.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || trace.ProviderRequestId.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || trace.ProviderResponseId.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || trace.InputText.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || trace.OutputText.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || trace.ToolCallsText.Contains(filter, StringComparison.OrdinalIgnoreCase);
    }

    private static bool ProviderTraceTurnMatches(ProviderTraceTurnItem turn, string filter)
    {
        if (string.IsNullOrWhiteSpace(filter)) return true;
        return turn.TurnId.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || turn.ModelId.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || turn.State.Contains(filter, StringComparison.OrdinalIgnoreCase)
            || turn.Sequence.ToString().Contains(filter, StringComparison.OrdinalIgnoreCase);
    }

    private static ProviderTraceTurnItem ProviderTraceTurnFromJson(JsonObject item, int sequence, IReadOnlyList<ProviderTraceItem> calls) =>
        new(
            item.String("turnId", "turn_id"),
            item.String("state"),
            item.String("modelId", "model_id"),
            item.String("createdAt", "created_at"),
            item.String("startedAt", "started_at"),
            item.String("completedAt", "completed_at"),
            item.Long("inputTokens", "input_tokens"),
            item.Long("outputTokens", "output_tokens"),
            item.Long("totalTokens", "total_tokens"),
            sequence,
            calls);

    private static ProviderTraceItem ProviderTraceFromJson(JsonObject item)
    {
        var usage = item.Object("usage");
        var messages = item.Array("messages").OfType<JsonObject>().Select(message => new ProviderTraceMessageItem(
            message.String("messageId", "message_id"),
            message.String("role"),
            MessageText(message.Object("message")),
            message.String("createdAt", "created_at"))).ToList();
        var tools = item.Array("toolUses", "tool_uses").OfType<JsonObject>().Select(tool => new ProviderTraceToolItem(
            tool.String("toolCallId", "tool_call_id"),
            tool.String("name"),
            tool.String("state"),
            Pretty(tool["request"]),
            Pretty(tool["result"]),
            tool.String("errorCode", "error_code"),
            tool.String("createdAt", "created_at"))).ToList();
        return new ProviderTraceItem(
            item.String("exchangeId", "exchange_id"),
            item.String("turnId", "turn_id"),
            item.String("provider"),
            item.String("modelId", "model_id"),
            item.String("wireModel", "wire_model"),
            item.String("providerRequestId", "provider_request_id"),
            item.String("providerResponseId", "provider_response_id"),
            item.String("state"),
            item.Int("iteration"),
            item.String("startedAt", "started_at"),
            item.String("completedAt", "completed_at"),
            OptionalLong(usage, "input_tokens"),
            OptionalLong(usage, "output_tokens"),
            OptionalLong(usage, "cache_read_tokens"),
            OptionalLong(usage, "cache_write_tokens"),
            OptionalLong(usage, "total_tokens"),
            item.String("finishReason", "finish_reason"),
            Pretty(item["inputMessages"] ?? item["input_messages"]),
            OutputText(item["outputMessage"] ?? item["output_message"]),
            Pretty(item["toolCalls"] ?? item["tool_calls"]),
            Pretty(item["error"]),
            messages,
            tools);
    }

    private async Task<ProviderTraceItem> GetProviderTraceDetailsAsync(string sessionId, string exchangeId)
    {
        if (_providerTraceDetails.TryGetValue(exchangeId, out var cached)) return cached;
        if (!_providerTraceDetailLoads.TryGetValue(exchangeId, out var loading))
        {
            loading = LoadProviderTraceDetailsCoreAsync(sessionId, exchangeId);
            _providerTraceDetailLoads[exchangeId] = loading;
        }
        try
        {
            var details = await loading;
            _providerTraceDetails[exchangeId] = details;
            return details;
        }
        finally
        {
            _providerTraceDetailLoads.Remove(exchangeId);
        }
    }

    private async Task<ProviderTraceItem> LoadProviderTraceDetailsCoreAsync(string sessionId, string exchangeId)
    {
        var result = await _sdk!.ProviderExchangeAsync(sessionId, exchangeId);
        return ProviderTraceFromJson(result);
    }

    private static void PopulateProviderTraceContents(ProviderTraceItem trace, ProviderTraceItem details)
    {
        if (trace.ContentsLoaded) return;
        var contents = new List<ProviderTraceContentItem>();
        var identities = new HashSet<string>(StringComparer.Ordinal);

        void AddMessage(string role, string content, string createdAt)
        {
            if (role is not ("user" or "assistant" or "thinking") || string.IsNullOrWhiteSpace(content)) return;
            var identity = $"{role}\n{content}";
            if (!identities.Add(identity)) return;
            contents.Add(new ProviderTraceContentItem(
                trace.ExchangeId,
                role,
                role switch
                {
                    "user" => "User message",
                    "assistant" => "Assistant message",
                    _ => "Thinking message"
                },
                Preview(content),
                content,
                string.Empty,
                string.Empty,
                string.Empty,
                createdAt));
        }

        JsonArray? inputMessages = null;
        if (!string.IsNullOrWhiteSpace(details.InputText))
        {
            try
            {
                inputMessages = JsonNode.Parse(details.InputText) as JsonArray;
            }
            catch (JsonException)
            {
                // The raw request remains available in the call overview.
            }
        }
        if (inputMessages is not null)
        {
            foreach (var message in inputMessages.OfType<JsonObject>())
                AddMessage(message.String("role"), MessageText(message), string.Empty);
        }
        foreach (var message in details.Messages)
            AddMessage(message.Role, message.Content, message.CreatedAt);
        AddMessage("assistant", details.OutputText, details.CompletedAt);

        contents.AddRange(details.Tools.Select(tool => new ProviderTraceContentItem(
            trace.ExchangeId,
            "tool",
            tool.Name,
            tool.StateText,
            string.Empty,
            tool.Request,
            tool.Result,
            tool.ErrorCode,
            tool.CreatedAt)));

        trace.Contents.Clear();
        foreach (var content in contents) trace.Contents.Add(content);
        if (trace.Contents.Count == 0)
            trace.Contents.Add(ProviderTraceContentItem.Placeholder("No messages or tool uses"));
        trace.ContentsLoaded = true;
    }

    private static string Preview(string value)
    {
        var compact = string.Join(" ", value.Split((char[]?)null, StringSplitOptions.RemoveEmptyEntries));
        return compact.Length <= 72 ? compact : $"{compact[..72]}…";
    }

    private static long? OptionalLong(JsonObject value, string name)
    {
        if (value.Count == 0) return null;
        if (value[name] is not JsonValue item) return null;
        return item.TryGetValue<long>(out var result) ? result : null;
    }

    private static string OutputText(JsonNode? node)
    {
        if (node is not JsonObject message) return string.Empty;
        var text = MessageText(message);
        return string.IsNullOrWhiteSpace(text) ? Pretty(node) : text;
    }

    private static string Pretty(JsonNode? node)
    {
        if (node is null) return string.Empty;
        return node.ToJsonString(DisplayJson.Options);
    }

    private void NotifyGitDiffPresentationChanged()
    {
        OnPropertyChanged(nameof(HasGitDiffLines));
        OnPropertyChanged(nameof(ShowGitDiffEmpty));
        OnPropertyChanged(nameof(ShowGitDiffStats));
        OnPropertyChanged(nameof(GitEmptyMessage));
    }

    private static string CompactNumber(long value)
    {
        if (value < 1_000) return value.ToString("N0");
        if (value < 1_000_000) return $"{value / 1_000d:0.#}k";
        return $"{value / 1_000_000d:0.#}m";
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;
        Interlocked.Increment(ref _sessionLoadVersion);
        CloseSubscription();
        _sdk?.Dispose();
        _sdk = null;
    }
}

internal sealed record SessionSnapshotProjection(
    IReadOnlyList<MessageItem> Messages,
    IReadOnlyList<ActivityItem> Activities,
    IReadOnlyList<string> ChangedPaths,
    ApprovalItem? PendingApproval,
    string ActiveTurnId);
