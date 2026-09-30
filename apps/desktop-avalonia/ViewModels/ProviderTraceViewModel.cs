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

    internal ProviderTraceViewModel(IProviderTraceHost host) => _host = host;

    public ObservableCollection<ProviderTraceItem> Traces { get; } = [];
    public ObservableCollection<ProviderTraceTurnItem> Turns { get; } = [];
    public ObservableCollection<ProviderTraceTurnItem> FilteredTurns { get; } = [];

    public ProviderTraceItem? SelectedTrace
    {
        get => _selectedTrace;
        set
        {
            if (SetProperty(ref _selectedTrace, value))
            {
                SelectedTraceDetails = null;
                NotifySelectionChanged();
            }
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
    public string Filter { get => _filter; private set { if (SetProperty(ref _filter, value)) OnPropertyChanged(nameof(EmptyMessage)); } }

    public bool HasTraces => Traces.Count > 0;
    public bool HasFilteredTraces => FilteredTurns.Count > 0;
    public bool HasSelectedTrace => SelectedTraceDetails is not null;
    public bool ShowSelectedTraceOverview => HasSelectedTrace;
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

    internal async Task RefreshAsync(string? requestedSessionId, long? loadVersion)
    {
        if (!_host.EnsureSdk() || _host.SelectedSessionId is not { } selectedSessionId)
        {
            Clear();
            return;
        }
        var sessionId = requestedSessionId ?? selectedSessionId;
        if (!_host.IsSessionContextCurrent(sessionId, loadVersion)) return;
        State = "loading";
        Error = string.Empty;
        try
        {
            var result = await _host.Sdk!.GetProviderExchangesAsync(sessionId);
            if (!_host.IsSessionContextCurrent(sessionId, loadVersion)) return;
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
            if (SelectedTrace is { } selected)
            {
                await LoadAsync(selected);
            }
        }
        catch (Exception exception)
        {
            if (!_host.IsSessionContextCurrent(sessionId, loadVersion)) return;
            State = "error";
            Error = exception.Message;
        }
    }

    public async Task LoadAsync(ProviderTraceItem trace)
    {
        if (!_host.EnsureSdk() || _host.SelectedSessionId is not { } sessionId) return;
        SelectedTrace = trace;
        SelectedTraceDetails = trace;
        if (_details.TryGetValue(trace.ExchangeId, out var cached))
        {
            SelectedTraceDetails = cached;
            return;
        }
        State = "loading";
        Error = string.Empty;
        try
        {
            var details = await GetDetailsAsync(sessionId, trace.ExchangeId);
            if (_host.SelectedSessionId != sessionId || SelectedTrace?.ExchangeId != trace.ExchangeId) return;
            SelectedTraceDetails = details;
            State = "ready";
        }
        catch (Exception exception)
        {
            if (_host.SelectedSessionId != sessionId || SelectedTrace?.ExchangeId != trace.ExchangeId) return;
            State = "error";
            Error = exception.Message;
        }
    }

    public void SetFilter(string filter)
    {
        Filter = filter ?? string.Empty;
        ApplyFilter();
    }

    public void SelectTurn()
    {
        SelectedTrace = null;
        if (State == "loading") State = "ready";
    }

    internal void Clear()
    {
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
    internal void OnSessionChanged() => OnPropertyChanged(nameof(EmptyMessage));

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
        try
        {
            var details = await loading;
            _details[exchangeId] = details;
            return details;
        }
        finally
        {
            _detailLoads.Remove(exchangeId);
        }
    }

    private async Task<ProviderTraceItem> LoadDetailsCoreAsync(string sessionId, string exchangeId)
    {
        var result = await _host.Sdk!.GetProviderExchangeAsync(sessionId, exchangeId);
        return ProviderTraceMapper.FromSdk(result);
    }

    private void NotifySelectionChanged()
    {
        OnPropertyChanged(nameof(SelectedTraceTitle));
        OnPropertyChanged(nameof(HasSelectedTrace));
        OnPropertyChanged(nameof(ShowSelectedTraceOverview));
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
