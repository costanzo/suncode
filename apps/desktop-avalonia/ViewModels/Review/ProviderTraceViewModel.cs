using System.Collections.ObjectModel;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;

namespace SunCode.Desktop.ViewModels;

// Provider request/response trace for the selected session. Drawer visibility
// stays with the window layout.
public sealed class ProviderTraceViewModel : ObservableObject
{
    private readonly IProviderTraceHost _host;
    private readonly Dictionary<string, ProviderTraceItem> _details = new(StringComparer.Ordinal);
    private readonly Dictionary<string, Task<ProviderTraceItem>> _detailLoads = new(StringComparer.Ordinal);
    private ProviderTraceItem? _selectedTrace;
    private ProviderTraceItem? _selectedTraceDetails;
    private string _state = "idle";
    private string _error = string.Empty;
    private string _filter = string.Empty;
    private long _contextVersion;
    private long _refreshVersion;
    private long _selectionVersion;
    private long _navigationVersion;
    private string _failureNavigationState = string.Empty;
    private (string TurnId, string ExchangeId, string Code)? _failureDestination;

    internal event Action? Refreshed;
    internal event Action? FailureFocusRequested;

    internal ProviderTraceViewModel(IProviderTraceHost host) => _host = host;

    public ObservableCollection<ProviderTraceItem> Traces { get; } = [];
    public ObservableCollection<ProviderTraceTurnItem> Turns { get; } = [];
    public ObservableCollection<ProviderTraceTurnItem> FilteredTurns { get; } = [];

    public ProviderTraceItem? SelectedTrace
    {
        get => _selectedTrace;
        set
        {
            if (ReferenceEquals(_selectedTrace, value)) return;
            _selectedTrace = value;
            _selectionVersion++;
            SelectedTraceDetails = null;
            OnPropertyChanged();
            NotifySelectionChanged();
        }
    }

    public ProviderTraceItem? SelectedTraceDetails
    {
        get => _selectedTraceDetails;
        private set
        {
            if (SetProperty(ref _selectedTraceDetails, value)) NotifySelectionChanged();
        }
    }

    public string State
    {
        get => _state;
        private set
        {
            if (!SetProperty(ref _state, value)) return;
            OnPropertyChanged(nameof(IsLoading));
            OnPropertyChanged(nameof(EmptyMessage));
        }
    }
    public string Error { get => _error; private set => SetProperty(ref _error, value); }
    public string Filter
    {
        get => _filter;
        set
        {
            if (!SetProperty(ref _filter, value ?? string.Empty)) return;
            ApplyFilter();
            OnPropertyChanged(nameof(EmptyMessage));
        }
    }

    public string FailureNavigationState
    {
        get => _failureNavigationState;
        private set
        {
            if (!SetProperty(ref _failureNavigationState, value)) return;
            OnPropertyChanged(nameof(IsRevealingFailure));
            OnPropertyChanged(nameof(ShowFailureNavigationFeedback));
            OnPropertyChanged(nameof(CanRetryFailureNavigation));
            OnPropertyChanged(nameof(FailureNavigationMessage));
            OnPropertyChanged(nameof(ShowSelectedTraceOverview));
            OnPropertyChanged(nameof(ShowEmptyState));
        }
    }
    public bool IsRevealingFailure => FailureNavigationState == "loading";
    public bool ShowFailureNavigationFeedback => FailureNavigationState is "loading" or "missing" or "error";
    public bool CanRetryFailureNavigation => FailureNavigationState == "error";
    public string FailureNavigationMessage => FailureNavigationState switch
    {
        "loading" => LocalizationService.GetString("LocLoadingRequestDetails", "Loading request details…"),
        "missing" => LocalizationService.GetString("LocNoMatchingProviderRequest", "This turn has no matching failed provider request to inspect."),
        "error" => LocalizationService.GetString("LocRequestDetailsLoadFailed", "Could not load request details. Try again."),
        _ => string.Empty
    };

    public bool HasTraces => Traces.Count > 0;
    public bool HasFilteredTraces => FilteredTurns.Count > 0;
    public bool HasSelectedTrace => SelectedTraceDetails is not null;
    public bool ShowSelectedTraceOverview => HasSelectedTrace && !ShowFailureNavigationFeedback;
    public bool ShowEmptyState => !HasSelectedTrace && !ShowFailureNavigationFeedback;
    public string CountText => $"{FilteredTurns.Count} turns · {FilteredTurns.Sum(turn => turn.Calls.Count)} calls";
    public bool IsLoading => State == "loading";
    public string Summary => Traces.Count == 0
        ? "No provider requests"
        : $"{Traces.Count} provider {(Traces.Count == 1 ? "request" : "requests")}";
    public string SelectedTraceTitle => SelectedTraceDetails?.Title ??
                                        SelectedTrace?.Title ?? "No model call selected";
    public string EmptyMessage
    {
        get
        {
            if (State == "loading") return "Loading session trace...";
            if (State == "error") return string.IsNullOrWhiteSpace(Error) ? "Provider trace is unavailable." : Error;
            if (_host.SelectedSessionId is null) return "Select a session to inspect provider requests.";
            if (FilteredTurns.Count == 0 && Filter.Length > 0) return "No turns or model calls match this filter.";
            if (FilteredTurns.Count == 0) return "No turns have been recorded for this session.";
            return "Select a model call to inspect its messages, tools, request, response, and usage.";
        }
    }

    public Task RefreshAsync() => RefreshAsync(null, null);

    internal async Task RefreshAsync(string? requestedSessionId, long? loadVersion, bool loadSelection = true)
    {
        if (IsRevealingFailure && loadSelection) return;
        if (!_host.EnsureSdk() || _host.SelectedSessionId is not { } selectedSessionId)
        {
            Clear();
            return;
        }
        var sessionId = requestedSessionId ?? selectedSessionId;
        if (!_host.IsSessionContextCurrent(sessionId, loadVersion)) return;
        var contextVersion = _contextVersion;
        var refreshVersion = ++_refreshVersion;
        State = "loading";
        Error = string.Empty;
        try
        {
            var result = await _host.GetProviderExchangesAsync(sessionId);
            if (!_host.IsSessionContextCurrent(sessionId, loadVersion) || contextVersion != _contextVersion || refreshVersion != _refreshVersion) return;
            Traces.Clear();
            Turns.Clear();
            _details.Clear();
            _detailLoads.Clear();
            var exchanges = result.Exchanges.Select(ProviderTraceMapper.FromSdk).ToList();
            foreach (var exchange in exchanges) Traces.Add(exchange);
            var modelId = _host.SelectedModelId;
            var latestUsage = result.Exchanges
                .Where(item => item.Usage is not null && (modelId is null || item.ModelId == modelId))
                .OrderBy(item => item.StartedAt, StringComparer.Ordinal)
                .ThenBy(item => item.Iteration)
                .LastOrDefault()?.Usage;
            if (latestUsage is not null) _host.UpdateContextUsage(latestUsage);
            for (var index = 0; index < result.Turns.Count; index++)
            {
                var item = result.Turns[index];
                var turnId = item.TurnId;
                var calls = exchanges.Where(call => call.TurnId == turnId).OrderBy(call => call.Iteration).ThenBy(call => call.StartedAt).ToList();
                Turns.Add(ProviderTraceMapper.TurnFromSdk(item, result.Turns.Count - index, calls));
            }
            ApplyFilter();
            State = "ready";
            Refreshed?.Invoke();
            if (loadSelection && !IsRevealingFailure && SelectedTrace is { } selected)
            {
                await LoadAsync(selected);
            }
        }
        catch (Exception exception)
        {
            if (!_host.IsSessionContextCurrent(sessionId, loadVersion) || contextVersion != _contextVersion || refreshVersion != _refreshVersion) return;
            State = "error";
            Error = exception.Message;
        }
    }

    public Task LoadAsync(ProviderTraceItem trace) => LoadAsync(trace, fromFailure: false);

    private async Task LoadAsync(ProviderTraceItem trace, bool fromFailure)
    {
        if (!_host.EnsureSdk() || _host.SelectedSessionId is not { } sessionId) return;
        if (!fromFailure)
        {
            _navigationVersion++;
            FailureNavigationState = string.Empty;
        }
        SelectedTrace = trace;
        SelectedTraceDetails = trace;
        var contextVersion = _contextVersion;
        var refreshVersion = _refreshVersion;
        var selectionVersion = ++_selectionVersion;
        if (_details.TryGetValue(trace.ExchangeId, out var cached))
        {
            SelectedTraceDetails = cached;
            State = "ready";
            Error = string.Empty;
            return;
        }
        State = "loading";
        Error = string.Empty;
        try
        {
            var details = await GetDetailsAsync(sessionId, trace.ExchangeId);
            if (!_host.IsSessionContextCurrent(sessionId, null) || contextVersion != _contextVersion || refreshVersion != _refreshVersion || selectionVersion != _selectionVersion) return;
            SelectedTraceDetails = details;
            State = "ready";
        }
        catch (Exception exception)
        {
            if (!_host.IsSessionContextCurrent(sessionId, null) || contextVersion != _contextVersion || refreshVersion != _refreshVersion || selectionVersion != _selectionVersion) return;
            State = "error";
            Error = exception.Message;
        }
    }

    public void SetFilter(string filter)
    {
        Filter = filter ?? string.Empty;
    }

    public void SelectTurn()
    {
        _navigationVersion++;
        FailureNavigationState = string.Empty;
        SelectedTrace = null;
        if (State == "loading") State = "ready";
    }

    internal void Clear()
    {
        _contextVersion++;
        _refreshVersion++;
        _navigationVersion++;
        _failureDestination = null;
        FailureNavigationState = string.Empty;
        Traces.Clear();
        Turns.Clear();
        FilteredTurns.Clear();
        SelectedTrace = null;
        SelectedTraceDetails = null;
        _details.Clear();
        _detailLoads.Clear();
        State = "idle";
        Error = string.Empty;
        Filter = string.Empty;
        NotifyCollectionsChanged();
    }

    // The empty-state text depends on whether the window has a session selected.
    internal void OnSessionChanged()
    {
        Clear();
        OnPropertyChanged(nameof(EmptyMessage));
    }

    internal void RefreshLocalizedText() => OnPropertyChanged(nameof(FailureNavigationMessage));

    internal TurnFailureItem? FindFailure(string turnId, string code, string exchangeId = "")
    {
        if (code.Length == 0) code = Turns.FirstOrDefault(item => item.TurnId == turnId)?.ErrorCode ?? string.Empty;
        if (code.Length == 0 && exchangeId.Length == 0) return null;
        var calls = Traces.Where(item => item.TurnId == turnId)
            .OrderByDescending(item => item.StartedAt, StringComparer.Ordinal)
            .ThenByDescending(item => item.CompletedAt, StringComparer.Ordinal)
            .ToList();
        if (exchangeId.Length == 0 && calls.Count > 1
            && calls[0].StartedAt == calls[1].StartedAt && calls[0].CompletedAt == calls[1].CompletedAt)
            return null;
        var call = exchangeId.Length > 0
            ? calls.FirstOrDefault(item => item.ExchangeId == exchangeId)
            : calls.FirstOrDefault();
        if (call is not { IsFailed: true, HasError: true }) return null;
        var failure = TurnFailureItem.FromCall(call);
        return code.Length == 0 || code == failure.Code ? failure : null;
    }

    internal async Task RevealFailureAsync(string turnId, string exchangeId, string code)
    {
        if (IsRevealingFailure || _host.SelectedSessionId is not { } sessionId) return;
        var contextVersion = _contextVersion;
        var navigationVersion = ++_navigationVersion;
        _failureDestination = (turnId, exchangeId, code);
        SelectedTrace = null;
        FailureNavigationState = "loading";
        await RefreshAsync(sessionId, null, loadSelection: false);
        if (!_host.IsSessionContextCurrent(sessionId, null) || contextVersion != _contextVersion || navigationVersion != _navigationVersion) return;
        if (State == "error")
        {
            FailureNavigationState = "error";
            FailureFocusRequested?.Invoke();
            return;
        }
        var failure = FindFailure(turnId, code, exchangeId);
        if (failure is null)
        {
            SelectedTrace = null;
            FailureNavigationState = "missing";
            FailureFocusRequested?.Invoke();
            return;
        }
        var trace = Traces.First(item => item.ExchangeId == failure.ExchangeId && item.TurnId == turnId);
        var turn = Turns.First(item => item.TurnId == turnId);
        turn.IsExpanded = true;
        if (!FilteredTurns.SelectMany(item => item.Calls).Any(item => item.ExchangeId == trace.ExchangeId)) Filter = string.Empty;
        ApplyFilter();
        await LoadAsync(trace, fromFailure: true);
        if (!_host.IsSessionContextCurrent(sessionId, null) || contextVersion != _contextVersion || navigationVersion != _navigationVersion) return;
        FailureNavigationState = State == "error" ? "error" : "ready";
        FailureFocusRequested?.Invoke();
    }

    public Task RetryFailureNavigationAsync() => _failureDestination is { } destination
        ? RevealFailureAsync(destination.TurnId, destination.ExchangeId, destination.Code)
        : Task.CompletedTask;

    // Applies the current filter; exposed for tests that seed Turns directly.
    internal void ApplyFilter()
    {
        var selectedId = SelectedTrace?.ExchangeId;
        FilteredTurns.Clear();
        foreach (var turn in Turns)
        {
            var turnMatches = ProviderTraceMapper.TurnMatches(turn, Filter);
            var calls = turnMatches
                ? turn.Calls
                : turn.Calls.Where(trace => ProviderTraceMapper.Matches(trace, Filter)).ToList();
            if (turnMatches || calls.Count > 0)
            {
                FilteredTurns.Add(turn with { Calls = calls });
            }
        }
        var visibleCalls = FilteredTurns.SelectMany(turn => turn.Calls).ToList();
        SelectedTrace = visibleCalls.FirstOrDefault(item => item.ExchangeId == selectedId)
            ?? visibleCalls.FirstOrDefault();
        NotifyCollectionsChanged();
    }

    private async Task<ProviderTraceItem> GetDetailsAsync(string sessionId, string exchangeId)
    {
        if (_details.TryGetValue(exchangeId, out var cached)) return cached;
        if (!_detailLoads.TryGetValue(exchangeId, out var loading))
        {
            loading = LoadDetailsCoreAsync(sessionId, exchangeId);
            _detailLoads[exchangeId] = loading;
        }
        var contextVersion = _contextVersion;
        var refreshVersion = _refreshVersion;
        try
        {
            var details = await loading;
            if (contextVersion == _contextVersion && refreshVersion == _refreshVersion && _host.IsSessionContextCurrent(sessionId, null))
                _details[exchangeId] = details;
            return details;
        }
        finally
        {
            if (_detailLoads.TryGetValue(exchangeId, out var current) && ReferenceEquals(current, loading)) _detailLoads.Remove(exchangeId);
        }
    }

    private async Task<ProviderTraceItem> LoadDetailsCoreAsync(string sessionId, string exchangeId)
    {
        var result = await _host.GetProviderExchangeAsync(sessionId, exchangeId);
        return ProviderTraceMapper.FromSdk(result);
    }

    private void NotifySelectionChanged()
    {
        OnPropertyChanged(nameof(SelectedTraceTitle));
        OnPropertyChanged(nameof(HasSelectedTrace));
        OnPropertyChanged(nameof(ShowSelectedTraceOverview));
        OnPropertyChanged(nameof(ShowEmptyState));
    }

    private void NotifyCollectionsChanged()
    {
        OnPropertyChanged(nameof(HasTraces));
        OnPropertyChanged(nameof(HasFilteredTraces));
        OnPropertyChanged(nameof(CountText));
        OnPropertyChanged(nameof(Summary));
        OnPropertyChanged(nameof(EmptyMessage));
    }
}
