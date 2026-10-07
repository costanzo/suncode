using System.Collections.Specialized;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Input.Platform;
using Avalonia.Interactivity;
using Avalonia.Media;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using ConfirmationWindow = SunCode.Desktop.Views.DialogWindow.DialogWindow;

namespace SunCode.Desktop.Views.Settings;

// Settings pages bind to SettingsWindowViewModel. This code-behind owns only
// view services (dialogs, clipboard, tool windows) and the model-provider
// panel, whose control exposes its state as styled properties.
public sealed partial class SettingsWindow : Window, ISettingsDialogs
{
    private static string L(string key, string fallback) => LocalizationService.GetString(key, fallback);

    private static string LF(string key, string fallback, params object[] args) =>
        string.Format(System.Globalization.CultureInfo.CurrentCulture, L(key, fallback), args);

    private readonly SingletonWindow<McpServerEditorWindow> _mcpEditor = new();
    private readonly SingletonWindow<LanguageServerEditorWindow> _languageServerEditor = new();
    private SettingsWindowViewModel? _settings;

    public SettingsWindow()
    {
        InitializeComponent();
        // Block the window's DesktopViewModel from reaching bindings typed for
        // SettingsWindowViewModel before Rebind assigns it.
        SettingsRoot.DataContext = null;
        McpPage.AddRequested += () => OpenMcpEditor(null);
        McpPage.EditRequested += OpenMcpEditor;
        McpPage.DeleteRequested += DeleteMcpServer;
        LanguageServersPage.AddRequested += () => OpenLanguageServerEditor(null);
        LanguageServersPage.EditRequested += OpenLanguageServerEditor;
        LanguageServersPage.DeleteRequested += DeleteLanguageServer;
        WindowDecorations = Avalonia.Controls.WindowDecorations.Full;
        Icon = AppIcon.Window;
        AddHandler(KeyDownEvent, WindowKeyDown, RoutingStrategies.Tunnel);
        DataContextChanged += (_, _) => Rebind();
        Opened += async (_, _) =>
        {
            Rebind();
            if (_settings is not null) await _settings.OpenAsync();
        };
        Closed += (_, _) =>
        {
            Unbind();
            _mcpEditor.Close();
            _languageServerEditor.Close();
        };
    }

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;
    private string ActiveProviderId => _settings?.Navigation.LastProviderId ?? string.Empty;

    private void Rebind()
    {
        var next = DataContext as DesktopViewModel;
        if (ReferenceEquals(next, _settings?.Desktop)) return;
        Unbind();
        if (next is null) return;
        _settings = new SettingsWindowViewModel(next);
        _settings.Navigation.ProviderPanelShown += ShowProviderPanel;
        _settings.AttachDialogs(this);
        next.Models.CollectionChanged += ModelsCollectionChanged;
        next.AppSettings.LanguageChanged += LanguageChanged;
        SettingsRoot.DataContext = _settings;
    }

    private void Unbind()
    {
        if (_settings is not { } settings) return;
        settings.Navigation.ProviderPanelShown -= ShowProviderPanel;
        settings.Desktop.Models.CollectionChanged -= ModelsCollectionChanged;
        settings.Desktop.AppSettings.LanguageChanged -= LanguageChanged;
        settings.Dispose();
        _settings = null;
        SettingsRoot.DataContext = null;
    }

    private void WindowKeyDown(object? sender, KeyEventArgs e)
    {
        if (e.Key != Key.Escape) return;
        e.Handled = true;
        Close();
    }

    private void CloseSettings(object? sender, RoutedEventArgs e) => Close();

    private void AgentSelected(object? sender, string agent) => _settings?.Navigation.ShowAgent(agent);

    private void ProviderSelected(object? sender, string provider) => _settings?.Navigation.ShowProvider(provider);

    // Tool windows and confirmations

    private void OpenMcpEditor(McpServerItem? server) => _mcpEditor.Show(
        this, () => new McpServerEditorWindow(ViewModel.Mcp, server), () => McpNavigation.Focus());

    private void OpenLanguageServerEditor(LanguageServerItem? server) => _languageServerEditor.Show(
        this, () => new LanguageServerEditorWindow(ViewModel.LanguageServers, server), () => LanguageServersNavigation.Focus());

    private void DeleteMcpServer(McpServerItem server) => Confirm(
        new ConfirmationRequest(
            L("LocDeleteMcpServerTitle", "Delete MCP server?"),
            L("LocDeleteMcpServerMessage", "Its tools will be removed from new model requests. An already executing call may finish."),
            server.DisplayName,
            L("LocMcpServerCaps", "MCP SERVER"),
            L("LocDeleteServer", "Delete server")),
        () => _ = ViewModel.Mcp.DeleteServerAsync(server));

    private void DeleteLanguageServer(LanguageServerItem server) => Confirm(
        new ConfirmationRequest(
            L("LocDeleteLanguageServerTitle", "Delete language server?"),
            L("LocDeleteLanguageServerMessage", "Its project runtimes will stop and semantic results will no longer be available to new agent turns."),
            server.DisplayName,
            L("LocLanguageServerCaps", "LANGUAGE SERVER"),
            L("LocDeleteServer", "Delete server")),
        () => _ = ViewModel.LanguageServers.DeleteServerAsync(server));

    // Shared modal confirmation; the window stays disabled while it is open.
    private void Confirm(ConfirmationRequest request, Action confirmed)
    {
        var dialog = new ConfirmationWindow(
            request.Title, request.Message, request.Target, confirmed, request.TargetLabel, request.ConfirmLabel);
        IsEnabled = false;
        dialog.Closed += (_, _) => IsEnabled = true;
        _ = dialog.ShowDialog(this);
    }

    void ISettingsDialogs.Confirm(ConfirmationRequest request, Action confirmed) => Confirm(request, confirmed);

    async Task<bool> ISettingsDialogs.CopyTextAsync(string text)
    {
        if (TopLevel.GetTopLevel(this)?.Clipboard is not { } clipboard) return false;
        await clipboard.SetTextAsync(text);
        return true;
    }

    // Model provider panel

    private void ShowProviderPanel(string? provider)
    {
        ProviderManager.SelectedProviderId = provider;
        if (string.IsNullOrWhiteSpace(provider)) return;

        ProviderManager.EndpointText = ViewModel.ProviderEndpoint(provider);
        ProviderManager.EndpointStatusText = string.Empty;
        ProviderManager.EndpointStatusBrush = this.FindResource("TextSecondaryBrush") as IBrush;
        ProviderManager.ApiKeyText = string.Empty;
        var providerName = ViewModel.Providers.FirstOrDefault(item => item.Id == provider)?.DisplayName ?? provider;
        ProviderManager.ApiKeyPlaceholderText = LF("LocPasteApiKey", "Paste {0} API key", providerName);
        RefreshProvider();
    }

    private async void SaveCredential(object? sender, RoutedEventArgs e)
    {
        var value = ProviderManager.ApiKeyText?.Trim();
        if (string.IsNullOrWhiteSpace(value)) return;
        await ViewModel.SaveCredentialAsync(ActiveProviderId, value);
        ProviderManager.ApiKeyText = string.Empty;
        RefreshProvider();
    }

    private void ProviderApiKeyChanged(object? sender, TextChangedEventArgs e) =>
        ProviderManager.CanSaveCredential = !string.IsNullOrWhiteSpace(ProviderManager.ApiKeyText);

    private void ProviderEndpointChanged(object? sender, TextChangedEventArgs e) =>
        RefreshProviderEndpointActions();

    private async void SaveProviderEndpoint(object? sender, RoutedEventArgs e) =>
        await SaveProviderEndpointAsync(ProviderManager.EndpointText);

    private async void ResetProviderEndpoint(object? sender, RoutedEventArgs e)
    {
        var defaultEndpoint = ViewModel.Providers.FirstOrDefault(item => item.Id == ActiveProviderId)?.DefaultApiBase ?? string.Empty;
        if (string.IsNullOrWhiteSpace(defaultEndpoint))
            defaultEndpoint = ProviderManager.EndpointText?.Trim() ?? string.Empty;
        if (string.IsNullOrWhiteSpace(defaultEndpoint)) return;
        ProviderManager.EndpointText = defaultEndpoint;
        await SaveProviderEndpointAsync(defaultEndpoint);
    }

    private async Task SaveProviderEndpointAsync(string? endpoint)
    {
        var saved = await ViewModel.SaveProviderEndpointAsync(ActiveProviderId, endpoint);
        ProviderManager.EndpointStatusText = ViewModel.StatusText;
        ProviderManager.EndpointStatusBrush = this.FindResource(saved ? "SuccessBrush" : "DangerBrush") as IBrush;
        if (saved) ProviderManager.EndpointText = ViewModel.ProviderEndpoint(ActiveProviderId);
        RefreshProviderEndpointActions();
    }

    private async void RemoveCredential(object? sender, RoutedEventArgs e)
    {
        await ViewModel.RemoveCredentialAsync(ActiveProviderId);
        RefreshProvider();
    }

    private void RefreshProvider()
    {
        var configured = ViewModel.IsProviderConfigured(ActiveProviderId);
        ProviderManager.CredentialConfigured = configured;
        ProviderManager.CredentialStatusText = configured
            ? L("LocApiKeyConfigured", "API key configured")
            : L("LocNoApiKeyConfigured", "No API key configured");
        ProviderManager.CanRemoveCredential = configured;
        ProviderManager.CanSaveCredential = !string.IsNullOrWhiteSpace(ProviderManager.ApiKeyText);
        RefreshProviderEndpointActions();
        ProviderManager.ProviderModels = ViewModel.Models
            .Where(item => item.Provider == ActiveProviderId)
            .Select(item => new ProviderModelItem(item.Display, configured))
            .ToArray();
    }

    private void RefreshProviderEndpointActions()
    {
        var endpoint = ProviderManager.EndpointText?.Trim() ?? string.Empty;
        var savedEndpoint = ViewModel.ProviderEndpoint(ActiveProviderId).Trim();
        var defaultEndpoint = ViewModel.Providers.FirstOrDefault(item => item.Id == ActiveProviderId)?.DefaultApiBase ?? string.Empty;
        ProviderManager.CanSaveEndpoint = endpoint.Length > 0
            && !string.Equals(endpoint, savedEndpoint, StringComparison.Ordinal);
        ProviderManager.CanResetEndpoint = defaultEndpoint.Length > 0
            && !string.Equals(endpoint.TrimEnd('/'), defaultEndpoint.TrimEnd('/'), StringComparison.Ordinal);
    }

    private void LanguageChanged(string locale)
    {
        if (!string.IsNullOrWhiteSpace(ProviderManager.SelectedProviderId)) RefreshProvider();
    }

    private void ModelsCollectionChanged(object? sender, NotifyCollectionChangedEventArgs e)
    {
        if (!string.IsNullOrWhiteSpace(ProviderManager.SelectedProviderId)) RefreshProvider();
    }
}
