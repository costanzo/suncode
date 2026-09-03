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

    private const double ReviewPaneBreakpoint = 1100;
    private const double NavigationPaneBreakpoint = 860;
    private const double CompactWorkspaceBreakpoint = 620;

    private readonly object _initializationGate = new();
    private AgentSdk? _sdk;
    private Task? _initializationTask;
    private IDisposable? _subscription;
    private BulkObservableCollection<MessageItem> _messages = [];
    private ProjectItem? _selectedProject;
    private SessionItem? _selectedSession;
    private ModelItem? _selectedModel;
    private IReadOnlyList<ComposerAttachment> _submittedAttachments = [];
    private string? _selectedReasoningEffort;
    private GitFileItem? _selectedGitFile;
    private ProviderTraceItem? _selectedProviderTrace;
    private ProviderTraceItem? _selectedProviderTraceDetails;
    private ProviderTraceContentItem? _selectedProviderTraceContent;
    private readonly Dictionary<string, ProviderTraceItem> _providerTraceDetails = new(StringComparer.Ordinal);
    private readonly Dictionary<string, Task<ProviderTraceItem>> _providerTraceDetailLoads = new(StringComparer.Ordinal);
    private readonly HashSet<string> _appliedMessageIds = new(StringComparer.Ordinal);
    private ApprovalItem? _pendingApproval;
    private PendingQuestionItem? _pendingQuestion;
    private string _connectionState = "disconnected";
    private string _statusText = "Starting local agent...";
    private string _composerText = string.Empty;
    private string _activeTurnId = string.Empty;
    private string _activeTurnState = string.Empty;
    private string _themeMode = "light";
    private string _logLevel = "INFO";
    private string _logDirectory = string.Empty;
    private string _imageDirectory = string.Empty;
    private long _logMaxBytes = 10 * 1024 * 1024;
    private int _logRetention = 5;
    private bool _verifyHttpsCertificates = true;
    private bool _useSystemCertificates = true;
    private string _certificatePath = string.Empty;
    private int _toolCallLimit = 64;
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
    public ObservableCollection<ProviderItem> Providers { get; } = [];
    public ObservableCollection<ModelItem> Models { get; } = [];
    public IReadOnlyList<string> ReasoningEffortOptions =>
        SelectedModel?.ReasoningEfforts is { Count: > 0 } efforts
            ? efforts
            : ["low", "medium", "high"];
    public ObservableCollection<CredentialItem> Credentials { get; } = [];
    public ObservableCollection<ComposerAttachment> ComposerAttachments { get; } = [];
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
    public BulkObservableCollection<TodoItem> CurrentTodos { get; } = [];
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
            OnPropertyChanged(nameof(CanChooseModel));
            OnPropertyChanged(nameof(CanChooseReasoningEffort));
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
                if (value?.SupportsReasoningEffort != true)
                {
                    SelectedReasoningEffort = null;
                }
                else if (SelectedReasoningEffort is null)
                {
                    SelectedReasoningEffort = ReasoningEffortOptions.Contains("medium")
                        ? "medium"
                        : ReasoningEffortOptions.FirstOrDefault();
                }
                OnPropertyChanged(nameof(CanSubmit));
                OnPropertyChanged(nameof(CanCompose));
                OnPropertyChanged(nameof(SelectedModelName));
                OnPropertyChanged(nameof(CanChooseReasoningEffort));
                OnPropertyChanged(nameof(ReasoningEffortOptions));
                OnPropertyChanged(nameof(CanAttachImages));
                OnPropertyChanged(nameof(ComposerPlaceholder));
            }
        }
    }

    public string? SelectedReasoningEffort
    {
        get => _selectedReasoningEffort;
        set
        {
            var normalized = SelectedModel?.SupportsReasoningEffort == true
                && value is not null
                && ReasoningEffortOptions.Contains(value, StringComparer.Ordinal)
                ? value
                : null;
            if (SetProperty(ref _selectedReasoningEffort, normalized)) OnPropertyChanged(nameof(CanChooseReasoningEffort));
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

    public PendingQuestionItem? PendingQuestion
    {
        get => _pendingQuestion;
        private set
        {
            if (SetProperty(ref _pendingQuestion, value)) OnPropertyChanged(nameof(HasPendingQuestion));
        }
    }

    public string ConnectionState
    {
        get => _connectionState;
        private set
        {
            if (SetProperty(ref _connectionState, value))
            {
                OnPropertyChanged(nameof(CanOpenProjects));
                OnPropertyChanged(nameof(CanCompose));
                OnPropertyChanged(nameof(CanChooseModel));
                OnPropertyChanged(nameof(CanChooseReasoningEffort));
            }
        }
    }
    public string StatusText { get => _statusText; private set => SetProperty(ref _statusText, value); }
    public void ReportPresentationError(string message) => StatusText = message;
    public string ComposerText { get => _composerText; set { if (SetProperty(ref _composerText, value)) OnPropertyChanged(nameof(CanSubmit)); } }
    public string ActiveTurnId { get => _activeTurnId; private set { if (SetProperty(ref _activeTurnId, value)) { OnPropertyChanged(nameof(IsTurnActive)); OnPropertyChanged(nameof(IsTurnIndicatorDots)); OnPropertyChanged(nameof(CanSubmit)); OnPropertyChanged(nameof(CanCompose)); OnPropertyChanged(nameof(CanChooseReasoningEffort)); } } }
    public string ActiveTurnState { get => _activeTurnState; private set { if (SetProperty(ref _activeTurnState, value)) { OnPropertyChanged(nameof(IsTurnCompacting)); OnPropertyChanged(nameof(IsTurnThinking)); OnPropertyChanged(nameof(IsTurnIndicatorDots)); OnPropertyChanged(nameof(HasFailedTurn)); } } }
    public string ThemeMode { get => _themeMode; private set => SetProperty(ref _themeMode, value); }
    public string LogLevel { get => _logLevel; private set => SetProperty(ref _logLevel, value); }
    public string LogDirectory { get => _logDirectory; private set => SetProperty(ref _logDirectory, value); }
    public string ImageDirectory { get => _imageDirectory; private set => SetProperty(ref _imageDirectory, value); }
    public long LogMaxBytes { get => _logMaxBytes; private set => SetProperty(ref _logMaxBytes, value); }
    public int LogRetention { get => _logRetention; private set => SetProperty(ref _logRetention, value); }
    public bool VerifyHttpsCertificates { get => _verifyHttpsCertificates; private set => SetProperty(ref _verifyHttpsCertificates, value); }
    public int ToolCallLimit { get => _toolCallLimit; private set => SetProperty(ref _toolCallLimit, value); }
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
            OnPropertyChanged(nameof(CanChooseModel));
            OnPropertyChanged(nameof(CanChooseReasoningEffort));
        }
    }
    public string GitBranch { get => _gitBranch; private set { if (SetProperty(ref _gitBranch, value)) { OnPropertyChanged(nameof(GitFooterBranchText)); } } }
    public int GitChangedFiles { get => _gitChangedFiles; private set { if (SetProperty(ref _gitChangedFiles, value)) { OnPropertyChanged(nameof(GitChangeSummary)); OnPropertyChanged(nameof(IsGitClean)); OnPropertyChanged(nameof(IsGitDirty)); } } }
    public int GitAdditions { get => _gitAdditions; private set => SetProperty(ref _gitAdditions, value); }
    public int GitDeletions { get => _gitDeletions; private set => SetProperty(ref _gitDeletions, value); }
    public bool GitStatusTruncated { get => _gitStatusTruncated; private set => SetProperty(ref _gitStatusTruncated, value); }
    public bool GitDiffBinary { get => _gitDiffBinary; private set { if (SetProperty(ref _gitDiffBinary, value)) NotifyGitDiffPresentationChanged(); } }
    public bool GitDiffTruncated { get => _gitDiffTruncated; private set => SetProperty(ref _gitDiffTruncated, value); }
    public int GitDiffAdditions { get => _gitDiffAdditions; private set => SetProperty(ref _gitDiffAdditions, value); }
    public int GitDiffDeletions { get => _gitDiffDeletions; private set => SetProperty(ref _gitDiffDeletions, value); }
    public long SessionTotalTokens { get => _sessionTotalTokens; private set { if (SetProperty(ref _sessionTotalTokens, value)) OnPropertyChanged(nameof(SessionTokenText)); } }
    public bool NavigationVisible { get => _navigationVisible; set { if (SetProperty(ref _navigationVisible, value)) NotifyNavigationLayoutChanged(); } }
    public bool ExplorerVisible { get => _explorerVisible; set { if (SetProperty(ref _explorerVisible, value)) NotifyNavigationLayoutChanged(); } }
    public bool ReviewVisible { get => _reviewVisible; set { if (SetProperty(ref _reviewVisible, value)) NotifyReviewLayoutChanged(); } }
    public bool NavigationPinned { get => _navigationPinned; set => SetProperty(ref _navigationPinned, value); }
    public bool GitVisible { get => _gitVisible; set { if (SetProperty(ref _gitVisible, value)) NotifyDrawerLayoutChanged(nameof(EffectiveGitVisible)); } }
    public bool ProviderTraceVisible { get => _providerTraceVisible; set { if (SetProperty(ref _providerTraceVisible, value)) NotifyDrawerLayoutChanged(nameof(EffectiveProviderTraceVisible)); } }
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
            OnPropertyChanged(nameof(CanChooseModel));
        }
    }
    public bool IsSessionLoadingVisible { get => _isSessionLoadingVisible; private set => SetProperty(ref _isSessionLoadingVisible, value); }

    public bool IsProjectOpen => SelectedProject is not null;
    public bool CanOpenProjects => ConnectionState == "connected";
    public bool HasProjects => Projects.Count > 0;
    public bool HasProjectDependencies => ProjectDependencies.Count > 0;
    public bool SessionSidebarVisible => NavigationVisible && !ExplorerVisible;
    public bool ExplorerSidebarVisible => NavigationVisible && ExplorerVisible;
    public bool EffectiveNavigationVisible => NavigationVisible && _layoutWidth > NavigationPaneBreakpoint;
    public bool EffectiveSessionSidebarVisible => EffectiveNavigationVisible && !ExplorerVisible;
    public bool EffectiveExplorerSidebarVisible => EffectiveNavigationVisible && ExplorerVisible;
    public bool EffectiveReviewVisible => ReviewVisible && _layoutWidth > ReviewPaneBreakpoint;
    public bool EffectiveGitVisible => GitVisible && _layoutWidth > CompactWorkspaceBreakpoint;
    public bool EffectiveProviderTraceVisible => ProviderTraceVisible && _layoutWidth > CompactWorkspaceBreakpoint;
    public bool WorkspaceGuttersVisible => _layoutWidth > CompactWorkspaceBreakpoint;
    public GridLength WorkspaceGutterWidth => WorkspaceGuttersVisible ? new GridLength(26) : new GridLength(0);
    public GridLength WorkspaceGutterGap => WorkspaceGuttersVisible ? new GridLength(4) : new GridLength(0);
    public GridLength NavigationGap => EffectiveNavigationVisible ? new GridLength(4) : new GridLength(0);
    public GridLength ReviewGap => EffectiveReviewVisible ? new GridLength(4) : new GridLength(0);
    public GridLength BottomDrawerGap => EffectiveGitVisible || EffectiveProviderTraceVisible ? new GridLength(4) : new GridLength(0);
    public bool WorkspaceStatusDetailsVisible => _layoutWidth > CompactWorkspaceBreakpoint;
    public bool HasSessions => Sessions.Count > 0;
    public bool HasMessages => Messages.Count > 0;
    public bool HasActivities => Activities.Count > 0;
    public bool HasCurrentTodos => CurrentTodos.Count > 0;
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
    public bool HasPendingQuestion => PendingQuestion is not null;
    public bool HasChangedPaths => ChangedPaths.Count > 0;
    public string TurnChangeSummary => ChangedPaths.Count == 0
        ? "No files changed in this turn"
        : $"{ChangedPaths.Count} {(ChangedPaths.Count == 1 ? "file" : "files")} touched";
    public string RuntimeHealthSummary => IsAgentHealthy ? "Agent and database ready" : "Runtime needs attention";
    public bool IsTurnActive => !string.IsNullOrWhiteSpace(ActiveTurnId);
    public bool IsTurnCompacting => ActiveTurnState == "compacting";
    public bool IsTurnThinking => ActiveTurnState == "calling_model";
    public bool IsTurnIndicatorDots => IsTurnActive && !IsTurnThinking && !IsTurnCompacting;
    public bool HasFailedTurn => ActiveTurnState == "failed";
    public bool UseSystemCertificates { get => _useSystemCertificates; set => SetProperty(ref _useSystemCertificates, value); }
    public string CertificatePath { get => _certificatePath; set => SetProperty(ref _certificatePath, value); }
    public bool CanCompose => (ConnectionState == "connected" || IsSessionLoading) && SelectedSession is not null && SelectedModel?.Configured == true && !HasSessionLoadError;
    public bool CanSubmit => SelectedSession is not null && SelectedModel?.Configured == true && !string.IsNullOrWhiteSpace(ComposerText) && !IsTurnActive && !IsSessionLoading && !HasSessionLoadError;
    public bool CanChooseModel => (ConnectionState == "connected" || IsSessionLoading) && SelectedSession is not null && !HasSessionLoadError;
    public bool CanChooseReasoningEffort => CanCompose && SelectedModel?.SupportsReasoningEffort == true;
    public bool CanAttachImages => CanCompose && SelectedModel?.SupportsVision == true && !IsTurnActive;
    public string ProjectTitle => SelectedProject?.DisplayName ?? "SunCode";
    public string SessionTitle => SelectedSession?.DisplayTitle ?? "No session selected";
    public string SessionTokenText => $"Session {CompactNumber(SessionTotalTokens)} tokens";
    public string SelectedModelName => SelectedModel?.Id ?? string.Empty;
    public string LatestActivityText => Activities.LastOrDefault()?.Text ?? "No tool activity yet";
    public string ComposerPlaceholder => SelectedSession is null
        ? "Create a session first..."
        : SelectedModel is null
            ? "Choose a model first..."
            : SelectedModel.Configured ? "Ask SunCode to work on this project" : "Store the selected provider's API key first...";
    public GridLength NavigationWidth => EffectiveNavigationVisible ? new GridLength(NavigationPaneWidth) : new GridLength(0);
    public GridLength ReviewWidth => EffectiveReviewVisible ? new GridLength(ReviewPaneWidth) : new GridLength(0);
    public GridLength GitFileListWidth => new(Math.Min(260, Math.Max(230, (_layoutWidth - 80) * 0.24)));
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
        if (EffectiveNavigationVisible)
            NavigationPaneWidth = Math.Clamp(NavigationPaneWidth, 236, Math.Min(300, Math.Max(236, _layoutWidth - 560)));
        if (EffectiveReviewVisible)
            ReviewPaneWidth = Math.Clamp(ReviewPaneWidth, 276, Math.Min(352, Math.Max(276, _layoutWidth - 560)));
        NotifyResponsiveLayoutChanged();
        OnPropertyChanged(nameof(GitFileListWidth));
    }

    private void NotifyNavigationLayoutChanged()
    {
        OnPropertyChanged(nameof(SessionSidebarVisible));
        OnPropertyChanged(nameof(ExplorerSidebarVisible));
        OnPropertyChanged(nameof(EffectiveNavigationVisible));
        OnPropertyChanged(nameof(EffectiveSessionSidebarVisible));
        OnPropertyChanged(nameof(EffectiveExplorerSidebarVisible));
        OnPropertyChanged(nameof(NavigationWidth));
        OnPropertyChanged(nameof(NavigationGap));
    }

    private void NotifyReviewLayoutChanged()
    {
        OnPropertyChanged(nameof(EffectiveReviewVisible));
        OnPropertyChanged(nameof(ReviewWidth));
        OnPropertyChanged(nameof(ReviewGap));
    }

    private void NotifyResponsiveLayoutChanged()
    {
        NotifyNavigationLayoutChanged();
        NotifyReviewLayoutChanged();
        OnPropertyChanged(nameof(EffectiveGitVisible));
        OnPropertyChanged(nameof(EffectiveProviderTraceVisible));
        OnPropertyChanged(nameof(BottomDrawerGap));
        OnPropertyChanged(nameof(WorkspaceGuttersVisible));
        OnPropertyChanged(nameof(WorkspaceGutterWidth));
        OnPropertyChanged(nameof(WorkspaceGutterGap));
        OnPropertyChanged(nameof(WorkspaceStatusDetailsVisible));
    }

    private void NotifyDrawerLayoutChanged(string effectivePropertyName)
    {
        OnPropertyChanged(effectivePropertyName);
        OnPropertyChanged(nameof(BottomDrawerGap));
    }
}
