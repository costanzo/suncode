using System.Windows.Input;
using SunCode.Desktop.Infrastructure;

namespace SunCode.Desktop.ViewModels;

public enum SettingsPage
{
    Defaults,
    Appearance,
    Shortcuts,
    Network,
    Remote,
    Computer,
    Browser,
    Mcp,
    LanguageServers,
    Agents,
    Logging,
    Providers
}

internal static class SettingsPageKeys
{
    // Persisted page keys in ui-state.json; keep them stable.
    public static string ToKey(SettingsPage page) => page switch
    {
        SettingsPage.LanguageServers => "lsp",
        _ => page.ToString().ToLowerInvariant()
    };

    public static SettingsPage FromKey(string? key) => key switch
    {
        "lsp" => SettingsPage.LanguageServers,
        _ when Enum.TryParse<SettingsPage>(key, ignoreCase: true, out var page)
               && !int.TryParse(key, out _) => page,
        _ => SettingsPage.Defaults
    };
}

// Settings window navigation: the visible page, which navigation entry is
// highlighted, provider/agent group expansion, and persistence of that state.
// Selecting a page makes its view model the single active poll target.
public sealed class SettingsNavigationViewModel : ObservableObject
{
    private readonly PagePoller _poller;
    private readonly Func<SettingsPage, IPolledPage?> _polledPage;
    private readonly Action<UiSettingsState>? _save;
    private SettingsPage _selectedPage;
    private SettingsPage? _highlightedNavigation = SettingsPage.Defaults;
    private string? _highlightedProviderId;
    private string? _highlightedAgentId;
    private string? _selectedAgentId;
    private string? _selectedProviderId;
    private string _lastProviderId = string.Empty;
    private string _agentId = string.Empty;
    private bool _providersExpanded;
    private bool _agentsExpanded;
    private bool _ready;

    internal SettingsNavigationViewModel(
        PagePoller poller,
        Func<SettingsPage, IPolledPage?> polledPage,
        Action<UiSettingsState>? save = null)
    {
        _poller = poller;
        _polledPage = polledPage;
        _save = save;
        NavigateCommand = new RelayCommand(parameter =>
        {
            if (ParsePage(parameter) is { } page) Navigate(page);
        });
        ShowProviderCommand = new RelayCommand(parameter =>
        {
            if (parameter is string provider) ShowProvider(provider);
        });
        ShowAgentCommand = new RelayCommand(parameter =>
        {
            if (parameter is string agent) ShowAgent(agent);
        });
    }

    // Raised every time the provider panel is (re)shown so the view can reset
    // its transient credential and endpoint inputs. Null shows the overview.
    public event Action<string?>? ProviderPanelShown;

    public ICommand NavigateCommand { get; }
    public ICommand ShowProviderCommand { get; }
    public ICommand ShowAgentCommand { get; }

    public SettingsPage SelectedPage { get => _selectedPage; private set => SetProperty(ref _selectedPage, value); }

    // The top-level entry drawn as selected, or null when a provider or agent
    // sub-entry is selected instead.
    public SettingsPage? HighlightedNavigation { get => _highlightedNavigation; private set => SetProperty(ref _highlightedNavigation, value); }
    public string? HighlightedProviderId { get => _highlightedProviderId; private set => SetProperty(ref _highlightedProviderId, value); }
    public string? HighlightedAgentId { get => _highlightedAgentId; private set => SetProperty(ref _highlightedAgentId, value); }

    public string? SelectedProviderId { get => _selectedProviderId; private set => SetProperty(ref _selectedProviderId, value); }
    public string? SelectedAgentId { get => _selectedAgentId; private set => SetProperty(ref _selectedAgentId, value); }

    // Last provider whose panel was shown; the overview keeps it for persistence.
    public string LastProviderId => _lastProviderId;

    public bool ProvidersExpanded
    {
        get => _providersExpanded;
        private set
        {
            if (SetProperty(ref _providersExpanded, value)) Save();
        }
    }

    public bool AgentsExpanded
    {
        get => _agentsExpanded;
        private set
        {
            if (SetProperty(ref _agentsExpanded, value)) Save();
        }
    }

    public void Navigate(SettingsPage page)
    {
        switch (page)
        {
            case SettingsPage.Providers:
                ShowProviderPanel(null);
                Select(page, highlightTopLevel: true);
                ProvidersExpanded = !ProvidersExpanded;
                break;
            case SettingsPage.Agents:
                ShowAgentPanel(null);
                Select(page, highlightTopLevel: true);
                AgentsExpanded = !AgentsExpanded;
                break;
            default:
                Select(page, highlightTopLevel: true);
                break;
        }
    }

    public void ShowProvider(string provider)
    {
        ShowProviderPanel(provider);
        Select(SettingsPage.Providers, highlightTopLevel: false, providerId: provider);
    }

    public void ShowAgent(string agent)
    {
        ShowAgentPanel(agent);
        Select(SettingsPage.Agents, highlightTopLevel: false, agentId: agent);
    }

    // Restores persisted navigation without writing it back, then enables saving.
    internal void Restore(UiSettingsState state)
    {
        _ready = false;
        ProvidersExpanded = state.ProvidersExpanded;
        AgentsExpanded = state.AgentsExpanded;
        var page = SettingsPageKeys.FromKey(state.Page);
        if (page == SettingsPage.Providers) ShowProviderPanel(state.ProviderId);
        else if (page == SettingsPage.Agents) ShowAgentPanel(state.AgentId);
        else ShowProviderPanel(null);
        Select(page, highlightTopLevel: true);
        _ready = true;
    }

    public void Stop() => _poller.Deactivate();

    private void Select(SettingsPage page, bool highlightTopLevel, string? providerId = null, string? agentId = null)
    {
        HighlightedNavigation = highlightTopLevel ? page : null;
        HighlightedProviderId = highlightTopLevel ? null : providerId;
        HighlightedAgentId = highlightTopLevel ? null : agentId;
        SelectedPage = page;
        _poller.Activate(_polledPage(page));
        Save();
    }

    private void ShowProviderPanel(string? provider)
    {
        if (!string.IsNullOrWhiteSpace(provider)) _lastProviderId = provider;
        SelectedProviderId = string.IsNullOrWhiteSpace(provider) ? null : provider;
        ProviderPanelShown?.Invoke(SelectedProviderId);
    }

    private void ShowAgentPanel(string? agent)
    {
        _agentId = agent ?? string.Empty;
        SelectedAgentId = agent;
    }

    private void Save()
    {
        if (!_ready || _save is null) return;
        _save(new UiSettingsState
        {
            Page = SettingsPageKeys.ToKey(SelectedPage),
            ProviderId = _lastProviderId,
            ProvidersExpanded = ProvidersExpanded,
            AgentId = _agentId,
            AgentsExpanded = AgentsExpanded
        });
    }

    private static SettingsPage? ParsePage(object? parameter) => parameter switch
    {
        SettingsPage page => page,
        string key when Enum.TryParse<SettingsPage>(key, ignoreCase: true, out var page) => page,
        _ => null
    };
}
