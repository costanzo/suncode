using System.Collections.Specialized;
using System.ComponentModel;
using Avalonia.Threading;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Infrastructure.Updates;

namespace SunCode.Desktop.ViewModels;

// Root view model of the Settings window. It composes the window's
// DesktopViewModel feature children with Settings-only edit state and owns
// navigation plus the single shared page poller.
public sealed class SettingsWindowViewModel : ObservableObject, IDisposable
{
    private readonly PagePoller _poller;
    private bool _disposed;
    private bool _opened;
    private Task? _agentLoad;

    public SettingsWindowViewModel(DesktopViewModel desktop)
        : this(desktop, new PagePoller())
    {
    }

    internal SettingsWindowViewModel(DesktopViewModel desktop, PagePoller poller)
    {
        Desktop = desktop;
        _poller = poller;
        Navigation = new SettingsNavigationViewModel(poller, PolledPage, SaveNavigation);
        Navigation.PropertyChanged += NavigationChanged;
        Defaults = new DefaultsSettingsViewModel(desktop);
        Terminal = new ShellSettingsViewModel(desktop);
        Appearance = new AppearanceSettingsViewModel(desktop, desktop.AppSettings);
        Logging = new LoggingSettingsViewModel(desktop, desktop.AppSettings);
        NetworkEditor = new NetworkSettingsEditorViewModel(desktop, desktop.Network);
        RemoteEditor = new RemoteServerEditorViewModel(desktop.Remote, PostToUi);
        Updates = new UpdatesSettingsViewModel(UpdateRuntime.Service, locale: () => desktop.AppSettings.Language);
        Computer.SelectedModel = desktop.SelectedModel;
        desktop.PropertyChanged += DesktopPropertyChanged;
        desktop.Models.CollectionChanged += ModelsChanged;
        desktop.AppSettings.LanguageChanged += LanguageChanged;
    }

    public DesktopViewModel Desktop { get; }
    public SettingsNavigationViewModel Navigation { get; }

    public ComputerRuntimeViewModel Computer => Desktop.Computer;
    public BrowserRuntimeViewModel Browser => Desktop.Browser;
    public McpServersViewModel Mcp => Desktop.Mcp;
    public LanguageServersViewModel LanguageServers => Desktop.LanguageServers;
    public NetworkSettingsViewModel Network => Desktop.Network;
    public DefaultsSettingsViewModel Defaults { get; }
    public ShellSettingsViewModel Terminal { get; }
    public AppearanceSettingsViewModel Appearance { get; }
    public LoggingSettingsViewModel Logging { get; }
    public NetworkSettingsEditorViewModel NetworkEditor { get; }
    public RemoteServerEditorViewModel RemoteEditor { get; }
    public UpdatesSettingsViewModel Updates { get; }

    // Connects view services (confirmation dialogs, clipboard) from the window.
    internal void AttachDialogs(ISettingsDialogs? dialogs)
    {
        Browser.AttachDialogs(dialogs);
        RemoteEditor.AttachDialogs(dialogs);
        Updates.AttachDialogs(dialogs);
    }

    // Loads page data and restores the last navigation state when the window opens.
    public async Task OpenAsync(SettingsPage? initialPage = null)
    {
        Navigation.Restore(Desktop.SavedSettingsNavigation);
        if (initialPage is { } page) Navigation.Navigate(page);
        _opened = true;
        if (Navigation.SelectedPage == SettingsPage.Updates)
        {
            await Updates.LoadIdentityAsync();
            return;
        }
        await EnsureAgentPagesLoadedAsync();
    }

    private Task EnsureAgentPagesLoadedAsync() => _agentLoad ??= LoadAgentPagesAsync();
    private async void NavigationChanged(object? sender, PropertyChangedEventArgs e)
    {
        if (!_opened || e.PropertyName != nameof(SettingsNavigationViewModel.SelectedPage) || Navigation.SelectedPage == SettingsPage.Updates) return;
        try { await EnsureAgentPagesLoadedAsync(); }
        catch (Exception exception) { DiagnosticLog.Error("settings.load", exception, "operation=agent_pages"); }
    }
    private async Task LoadAgentPagesAsync()
    {
        await Desktop.LoadProjectToolCallLimitAsync();
        await Computer.LoadAsync();
        await Browser.LoadAsync();
        Defaults.Load();
        Appearance.Load();
        Logging.Load();
        NetworkEditor.Load();
        await RemoteEditor.LoadAsync();
    }

    private static void PostToUi(Action action)
    {
        if (Dispatcher.UIThread.CheckAccess()) action();
        else Dispatcher.UIThread.Post(action);
    }

    private IPolledPage? PolledPage(SettingsPage page) => page switch
    {
        SettingsPage.Terminal => Terminal,
        SettingsPage.Mcp => Mcp,
        SettingsPage.LanguageServers => LanguageServers,
        SettingsPage.Computer => Computer,
        SettingsPage.Browser => Browser,
        SettingsPage.Updates => Updates,
        _ => null
    };

    private void SaveNavigation(UiSettingsState state) => Desktop.SaveSettingsNavigation(
        state.Page, state.ProviderId, state.ProvidersExpanded, state.AgentId, state.AgentsExpanded);

    private void DesktopPropertyChanged(object? sender, PropertyChangedEventArgs e)
    {
        if (e.PropertyName == nameof(DesktopViewModel.SelectedModel))
            Computer.SelectedModel = Desktop.SelectedModel;
        else if (e.PropertyName == nameof(DesktopViewModel.SelectedProject))
            Browser.RefreshPresentation();
    }

    private void ModelsChanged(object? sender, NotifyCollectionChangedEventArgs e)
    {
        Defaults.RefreshModelOptions();
        Computer.SelectedModel = Desktop.SelectedModel;
    }

    private void LanguageChanged(string locale)
    {
        Terminal.RefreshLocalizedText();
        Computer.RefreshPresentation();
        Browser.RefreshPresentation();
        NetworkEditor.RefreshLocalizedText();
        Appearance.RefreshLocalizedText();
        Updates.RefreshLocalizedText();
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;
        _poller.Dispose();
        AttachDialogs(null);
        RemoteEditor.Dispose();
        Terminal.Dispose();
        Updates.Dispose();
        Navigation.PropertyChanged -= NavigationChanged;
        Desktop.PropertyChanged -= DesktopPropertyChanged;
        Desktop.Models.CollectionChanged -= ModelsChanged;
        Desktop.AppSettings.LanguageChanged -= LanguageChanged;
    }
}
