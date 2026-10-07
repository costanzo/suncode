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

namespace SunCode.Desktop.ViewModels;

public sealed partial class DesktopViewModel : ObservableObject, IDisposable, IViewModelHost, IProviderTraceHost
{

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
    private readonly HashSet<string> _appliedMessageIds = new(StringComparer.Ordinal);
    private ApprovalItem? _pendingApproval;
    private PendingQuestionItem? _pendingQuestion;
    private string _connectionState = "disconnected";
    private string _statusText = "Starting local agent...";
    private string _composerText = string.Empty;
    private string _activeTurnId = string.Empty;
    private string _activeTurnState = string.Empty;
    private string _lastTurnId = string.Empty;
    private int _toolCallLimit = 64;
    private string _diagnosticsText = "Diagnostics unavailable";
    private string _sessionLoadError = string.Empty;
    private bool _fullControlEnabled;
    private long _sessionLoadVersion;
    private long _childSessionsLoadVersion;
    private string? _loadedSessionId;
    private ChildSessionItem? _selectedChildSession;
    private ApprovalItem? _childPendingApproval;
    private bool _isBusy;
    private bool _isSessionLoading;
    private bool _isSessionLoadingVisible;
    private bool _disposed;
    private readonly DispatcherTimer _conversationDurationTimer = new() { Interval = TimeSpan.FromSeconds(1) };
    private DateTimeOffset? _activeTurnStartedAt;
    private string _activeTurnTimingTurnId = string.Empty;

    public event Action? SessionEntered;

    public AppSettingsViewModel AppSettings { get; }
    public McpServersViewModel Mcp { get; }
    public LanguageServersViewModel LanguageServers { get; }
    public NetworkSettingsViewModel Network { get; }
    public RemoteServerViewModel Remote { get; }
    public GitReviewViewModel Git { get; }
    public ProviderTraceViewModel ProviderTrace { get; }
    public BrowserRuntimeViewModel Browser { get; }
    public ComputerRuntimeViewModel Computer { get; }
    public WorkspaceLayoutViewModel Layout { get; }
    public ProviderTrafficViewModel ProviderTraffic { get; } = new();
    public ContextUsageViewModel ContextUsage { get; }
    public bool IsProviderTrafficVisible => IsProjectOpen;
    public ToolActivityViewModel ToolActivity { get; } = new();
    public ObservableCollection<ProjectItem> Projects { get; } = [];
    public ObservableCollection<SessionItem> Sessions { get; } = [];
    public ObservableCollection<SessionItem> ArchivedSessions { get; } = [];
    public ObservableCollection<ProviderItem> Providers { get; } = [];
    public ObservableCollection<ModelItem> Models { get; } = [];
    public IReadOnlyList<string> ReasoningEffortOptions =>
        SelectedModel?.ReasoningEfforts is { Count: > 0 } efforts
            ? efforts
            : ["low", "medium", "high"];
    public ObservableCollection<CredentialItem> Credentials { get; } = [];
    public ObservableCollection<AgentItem> Agents { get; } = [];
    public ObservableCollection<ComposerAttachment> ComposerAttachments { get; } = [];
    public double ComposerBottomClearance => ComposerAttachments.Count > 0 ? 180 : 128;
    public BulkObservableCollection<MessageItem> Messages
    {
        get => _messages;
        private set
        {
            if (ReferenceEquals(_messages, value)) return;
            _messages = value;
            OnPropertyChanged();
            OnPropertyChanged(nameof(HasMessages));
            NotifyAssistantStreamingChanged();
        }
    }
    public BulkObservableCollection<ActivityItem> Activities { get; } = [];
    public BulkObservableCollection<TodoItem> CurrentTodos { get; } = [];
    public BulkObservableCollection<string> ChangedPaths { get; } = [];
    public BulkObservableCollection<CheckpointItem> Checkpoints { get; } = [];
    public ObservableCollection<ChildSessionItem> ChildSessions { get; } = [];
    public ObservableCollection<ChildSessionTimelineItem> ChildSessionTimeline { get; } = [];

    public ProjectItem? SelectedProject
    {
        get => _selectedProject;
        private set
        {
            if (SetProperty(ref _selectedProject, value))
            {
                OnPropertyChanged(nameof(IsProjectOpen));
                OnPropertyChanged(nameof(IsProviderTrafficVisible));
                OnPropertyChanged(nameof(ProjectTitle));
                Mcp.OnProjectChanged();
            }
        }
    }

    internal void SetSelectedProjectForTests(ProjectItem project) => SelectedProject = project;
    internal void ClearSessionForTests() => ClearSession();

    public SessionItem? SelectedSession
    {
        get => _selectedSession;
        private set
        {
            if (ReferenceEquals(_selectedSession, value)) return;
            _selectedSession = value;
            OnPropertyChanged();
            OnPropertyChanged(nameof(SessionTitle));
            NotifyCurrentContentChanged();
            OnPropertyChanged(nameof(CanSubmit));
            OnPropertyChanged(nameof(CanCompose));
            OnPropertyChanged(nameof(CanChooseModel));
            OnPropertyChanged(nameof(CanChooseReasoningEffort));
            OnPropertyChanged(nameof(HasSelectedSession));
            OnPropertyChanged(nameof(ShowChatInput));
            OnPropertyChanged(nameof(ComposerPlaceholder));
            OnPropertyChanged(nameof(IsModelUnavailable));
            ProviderTrace.OnSessionChanged();
            SaveProjectUiState(saved => SaveCurrentContentState(saved));
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
                    SelectedReasoningEffort = ReasoningEffortOptions.FirstOrDefault();
                }
                OnPropertyChanged(nameof(CanSubmit));
                OnPropertyChanged(nameof(CanCompose));
                OnPropertyChanged(nameof(SelectedModelName));
                OnPropertyChanged(nameof(CanChooseReasoningEffort));
                OnPropertyChanged(nameof(ReasoningEffortOptions));
                OnPropertyChanged(nameof(CanAttachImages));
                OnPropertyChanged(nameof(ComposerPlaceholder));
                OnPropertyChanged(nameof(IsModelUnavailable));
                ContextUsage.Reset();
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

    public ApprovalItem? PendingApproval
    {
        get => _pendingApproval;
        internal set
        {
            if (SetProperty(ref _pendingApproval, value))
            {
                OnPropertyChanged(nameof(HasPendingApproval));
                OnPropertyChanged(nameof(ReviewGutterAttention));
                NotifyReviewPresentationChanged();
            }
        }
    }

    public PendingQuestionItem? PendingQuestion
    {
        get => _pendingQuestion;
        internal set
        {
            if (SetProperty(ref _pendingQuestion, value))
            {
                OnPropertyChanged(nameof(HasPendingQuestion));
                OnPropertyChanged(nameof(ReviewGutterAttention));
                NotifyReviewPresentationChanged();
            }
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
    public string ActiveTurnId { get => _activeTurnId; private set { if (SetProperty(ref _activeTurnId, value)) { OnPropertyChanged(nameof(IsTurnActive)); OnPropertyChanged(nameof(IsTurnIndicatorDots)); OnPropertyChanged(nameof(CanSubmit)); OnPropertyChanged(nameof(CanCompose)); OnPropertyChanged(nameof(CanChooseReasoningEffort)); OnPropertyChanged(nameof(CanAttachImages)); NotifyReviewPresentationChanged(); } } }
    public string ActiveTurnState { get => _activeTurnState; private set { if (SetProperty(ref _activeTurnState, value)) { OnPropertyChanged(nameof(IsTurnCompacting)); OnPropertyChanged(nameof(IsTurnThinking)); OnPropertyChanged(nameof(IsTurnIndicatorDots)); OnPropertyChanged(nameof(HasFailedTurn)); NotifyReviewPresentationChanged(); } } }
    public string ActiveTurnDurationText => _activeTurnStartedAt is { } started
        ? FormatDuration(started.ToString("O"), string.Empty, ActiveTurnState)
        : string.Empty;
    public string LastTurnId { get => _lastTurnId; private set => SetProperty(ref _lastTurnId, value); }
    public int ToolCallLimit { get => _toolCallLimit; private set => SetProperty(ref _toolCallLimit, value); }
    public string DiagnosticsText { get => _diagnosticsText; private set => SetProperty(ref _diagnosticsText, value); }
    public bool FullControlEnabled { get => _fullControlEnabled; private set => SetProperty(ref _fullControlEnabled, value); }
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
    // Drawer visibility lives in Layout; these keep the event-projection partials unchanged.
    private bool GitVisible { get => Layout.GitVisible; set => Layout.GitVisible = value; }
    private bool ProviderTraceVisible { get => Layout.ProviderTraceVisible; set => Layout.ProviderTraceVisible = value; }
    private bool ToolActivityVisible { get => Layout.ToolActivityVisible; set => Layout.ToolActivityVisible = value; }
    public ChildSessionItem? SelectedChildSession
    {
        get => _selectedChildSession;
        private set
        {
            if (!SetProperty(ref _selectedChildSession, value)) return;
            OnPropertyChanged(nameof(IsChildSessionVisible));
            OnPropertyChanged(nameof(IsConversationVisible));
            OnPropertyChanged(nameof(IsEditorVisible));
            OnPropertyChanged(nameof(HasSelectedChildSession));
            OnPropertyChanged(nameof(ChildSessionTitle));
            NotifyCurrentContentChanged();
        }
    }
    public ApprovalItem? ChildPendingApproval
    {
        get => _childPendingApproval;
        internal set
        {
            if (SetProperty(ref _childPendingApproval, value))
            {
                OnPropertyChanged(nameof(HasChildPendingApproval));
                OnPropertyChanged(nameof(ChildSessionsGutterAttention));
            }
        }
    }
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
    public bool HasSessions => Sessions.Count > 0;
    public bool HasArchivedSessions => ArchivedSessions.Count > 0;
    public bool HasMessages => Messages.Count > 0;
    public bool HasActivities => Activities.Count > 0;
    public bool HasCurrentTodos => CurrentTodos.Count > 0;
    public bool IsReviewTodosVisible => HasCurrentTodos && IsReviewRunning;
    public bool HasCheckpoints => Checkpoints.Count > 0;
    public bool HasChildSessions => ChildSessions.Count > 0;
    public bool HasSelectedChildSession => SelectedChildSession is not null;
    public bool HasChildPendingApproval => ChildPendingApproval is not null;
    public bool ReviewGutterAttention => (HasPendingApproval || HasPendingQuestion) && !Layout.ReviewVisible;
    public bool ChildSessionsGutterAttention => HasChildPendingApproval && !Layout.ChildSessionsVisible;
    public bool IsChildSessionVisible => SelectedChildSession is not null;
    public string ChildSessionTitle => SelectedChildSession?.Title ?? string.Empty;
    public bool HasSessionLoadError => !string.IsNullOrWhiteSpace(SessionLoadError);
    public bool HasSelectedSession => SelectedSession is not null;
    public bool ShowChatInput => SelectedSession is not null && !SelectedSession.IsArchived;
    public bool HasPendingApproval => PendingApproval is not null;
    public bool HasPendingQuestion => PendingQuestion is not null;
    public bool HasChangedPaths => ChangedPaths.Count > 0;
    public string TurnChangeSummary => ChangedPaths.Count == 0
        ? "No files changed in this turn"
        : $"{ChangedPaths.Count} {(ChangedPaths.Count == 1 ? "file" : "files")} touched";
    public bool IsTurnActive => !string.IsNullOrWhiteSpace(ActiveTurnId);
    public bool IsTurnCompacting => ActiveTurnState == "compacting";
    public bool IsTurnThinking => ActiveTurnState == "calling_model" && !IsAssistantStreaming;
    public bool IsAssistantStreaming => Messages.Any(message => message.Streaming);
    public bool IsTurnIndicatorDots => IsTurnActive && !IsTurnThinking && !IsTurnCompacting && !IsAssistantStreaming;
    public bool HasFailedTurn => ActiveTurnState == "failed";
    public string ReviewHeadingText => HasFailedTurn ? "Turn stopped" : IsTurnCompacting ? "Compacting context" : IsTurnActive ? "1 active process" : HasPendingApproval || HasPendingQuestion ? "Awaiting input" : "No active process";
    public string ReviewStatusText => HasFailedTurn
        ? LocalizationService.GetString("LocTurnFailed", "Turn failed")
        : IsTurnCompacting
            ? LocalizationService.GetString("LocCompactingConversationContext", "Compacting conversation context")
            : IsTurnActive
                ? LocalizationService.GetString("LocAgentRunning", "Agent running")
                : HasPendingApproval
                    ? LocalizationService.GetString("LocWaitingForApproval", "Waiting for approval")
                    : HasPendingQuestion
                        ? LocalizationService.GetString("LocWaitingForAnswer", "Waiting for answer")
                        : LocalizationService.GetString("LocAgentIdle", "Agent idle");
    public string ReviewStatusState => HasFailedTurn ? "failed" : IsTurnCompacting ? "compacting" : IsTurnActive ? "running" : HasPendingApproval ? "approval" : HasPendingQuestion ? "question" : "idle";
    public bool IsReviewIdle => !IsTurnActive && !HasPendingApproval && !HasPendingQuestion && !HasFailedTurn && !IsTurnCompacting;
    public bool IsReviewRunning => IsTurnActive && !IsTurnCompacting && !HasFailedTurn;
    public bool IsReviewCompacting => IsTurnCompacting;
    public bool IsReviewApproval => HasPendingApproval;
    public bool IsReviewQuestion => !HasPendingApproval && HasPendingQuestion;
    public bool IsReviewWaiting => HasPendingApproval || HasPendingQuestion;
    public bool IsReviewFailed => HasFailedTurn;
    public bool IsReviewChangesVisible => !IsReviewIdle && !IsReviewCompacting && !IsReviewFailed && HasChangedPaths;
    public bool IsReviewCheckpointVisible => IsReviewRunning && HasCheckpoints;
    public bool CanCompose => (ConnectionState == "connected" || IsSessionLoading) && SelectedSession is not null && !SelectedSession.IsArchived && SelectedModel?.Configured == true && !HasSessionLoadError;
    public bool CanSubmit => SelectedSession is not null && !SelectedSession.IsArchived && SelectedModel?.Configured == true && !string.IsNullOrWhiteSpace(ComposerText) && !IsTurnActive && !IsSessionLoading && !HasSessionLoadError;
    public bool CanChooseModel => (ConnectionState == "connected" || IsSessionLoading) && SelectedSession is not null && !SelectedSession.IsArchived && !HasSessionLoadError;
    public bool CanChooseReasoningEffort => CanCompose && SelectedModel?.SupportsReasoningEffort == true;
    public bool CanAttachImages => CanCompose && SelectedModel?.SupportsVision == true && !IsTurnActive;

    private void NotifyAssistantStreamingChanged()
    {
        OnPropertyChanged(nameof(IsAssistantStreaming));
        OnPropertyChanged(nameof(IsTurnIndicatorDots));
        OnPropertyChanged(nameof(IsTurnThinking));
    }

    private void UpdateContextUsage(SunCode.Sdk.Models.AgentUsage usage) => ContextUsage.Update(usage);

    private void NotifyReviewPresentationChanged()
    {
        OnPropertyChanged(nameof(ReviewHeadingText));
        OnPropertyChanged(nameof(ReviewStatusText));
        OnPropertyChanged(nameof(ReviewStatusState));
        OnPropertyChanged(nameof(IsReviewIdle));
        OnPropertyChanged(nameof(IsReviewRunning));
        OnPropertyChanged(nameof(IsReviewCompacting));
        OnPropertyChanged(nameof(IsReviewApproval));
        OnPropertyChanged(nameof(IsReviewQuestion));
        OnPropertyChanged(nameof(IsReviewWaiting));
        OnPropertyChanged(nameof(IsReviewFailed));
        OnPropertyChanged(nameof(IsReviewChangesVisible));
        OnPropertyChanged(nameof(IsReviewCheckpointVisible));
        OnPropertyChanged(nameof(IsReviewTodosVisible));
    }
    public string ProjectTitle => SelectedProject?.DisplayName ?? "SunCode";
    public string SessionTitle => SelectedSession?.DisplayTitle ?? "No session selected";
    public string SelectedModelName => SelectedModel?.Id ?? string.Empty;
    public string LatestActivityText => Activities.LastOrDefault()?.Text ?? "No tool activity yet";
    public string ComposerPlaceholder => SelectedSession is null
        ? "Create a session first..."
        : SelectedModel is null
            ? "Choose a model first..."
            : SelectedModel.Configured ? "Ask SunCode to work on this project" : string.Empty;
    public bool IsModelUnavailable => SelectedSession is not null && SelectedModel is not null && !SelectedModel.Configured;
}
