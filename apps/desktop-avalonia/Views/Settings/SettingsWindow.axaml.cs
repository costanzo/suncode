using System.Collections.Specialized;
using System.IO;
using System.Linq;
using Avalonia.Controls;
using Avalonia;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Media;
using Avalonia.Threading;
using Avalonia.VisualTree;
using SunCode.Desktop.Controls;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using ConfirmationWindow = SunCode.Desktop.Views.DialogWindow.DialogWindow;

namespace SunCode.Desktop.Views.Settings;

public sealed partial class SettingsWindow : Window
{
    private static readonly IReadOnlyList<SCComboBoxItem> ThemeOptions =
    [
        new("Dark", "dark"),
        new("Light", "light")
    ];

    private static readonly IReadOnlyList<SCComboBoxItem> LogLevelOptions =
    [
        new("TRACE", "TRACE"),
        new("DEBUG", "DEBUG"),
        new("INFO", "INFO"),
        new("WARN", "WARN"),
        new("ERROR", "ERROR"),
        new("OFF", "OFF")
    ];

    private static readonly IReadOnlyList<SCComboBoxItem> ProxyModeOptions =
    [
        new("No proxy", "no_proxy"),
        new("System proxy", "system"),
        new("Custom proxy", "custom")
    ];

    private bool _ready;
    private bool _providersExpanded = false;
    private bool _agentsExpanded = false;
    private string _provider = string.Empty;
    private string _agent = string.Empty;
    private DesktopViewModel? _subscribedViewModel;
    private McpServerEditorWindow? _mcpEditorWindow;
    private LanguageServerEditorWindow? _languageServerEditorWindow;
    private readonly DispatcherTimer _mcpPollTimer = new() { Interval = TimeSpan.FromSeconds(2) };
    private readonly DispatcherTimer _languageServerPollTimer = new() { Interval = TimeSpan.FromSeconds(2) };
    
    private int _baselineToolCallLimit;
    private string _baselineLogDirectory = string.Empty;
    private string _baselineLogLevel = string.Empty;
    private long _baselineLogMaxBytes;
    private int _baselineLogRetention;
    private String _baselineImageDirectory = string.Empty;
    private bool _baselineVerifyHttpsCertificates;
    private bool _baselineUseSystemCertificates;
    private string _baselineCertificatePath = string.Empty;
    private string _baselineProxyMode = "system";
    private string _baselineProxyUrl = string.Empty;
    private string _baselineProxyUsername = string.Empty;
    private string _baselineProxyBypass = string.Empty;
    private bool _baselineProxyPasswordConfigured;
    private bool _clearProxyPassword;

    public SettingsWindow()
    {
        InitializeComponent();
        DefaultsPage.DefaultModelChanged += DefaultModelChanged;
        DefaultsPage.SaveToolCallLimitRequested += SaveToolCallLimit;
        DefaultsPage.ToolCallLimitChanged += ToolCallLimitValueChanged;
        AppearancePage.ThemeChanged += ThemeChanged;
        LoggingPage.LogLevelChanged += LogLevelChanged;
        LoggingPage.LogDirectoryChanged += LoggingDirectoryChanged;
        LoggingPage.LogMaxMegabytesChanged += LoggingMaxMegabytesChanged;
        LoggingPage.LogRetentionChanged += LoggingRetentionChanged;
        LoggingPage.ImageDirectoryChanged += ImageDirectoryChanged;
        LoggingPage.SaveLoggingRequested += SaveLogging;
        LoggingPage.SaveImageDirectoryRequested += SaveImageDirectory;
        NetworkPage.HttpsCertificateVerificationChanged += HttpsCertificateVerificationChanged;
        NetworkPage.SystemCertificatesChanged += SystemCertificatesChanged;
        NetworkPage.SaveHttpsCertificateVerificationRequested += SaveHttpsCertificateVerification;
        NetworkPage.CertificatePathChanged += CertificatePathChanged;
        NetworkPage.ProxyModeChanged += ProxyModeChanged;
        NetworkPage.ProxyTextChanged += ProxyTextChanged;
        NetworkPage.RemoveProxyPasswordRequested += RemoveProxyPassword;
        NetworkPage.SaveProxyRequested += SaveProxy;
        McpPage.AddRequested += AddMcpServer;
        McpPage.EditRequested += EditMcpServer;
        McpPage.DeleteRequested += DeleteMcpServer;
        LanguageServersPage.AddRequested += AddLanguageServer;
        LanguageServersPage.EditRequested += EditLanguageServer;
        LanguageServersPage.DeleteRequested += DeleteLanguageServer;
        _mcpPollTimer.Tick += McpPollTick;
        _languageServerPollTimer.Tick += LanguageServerPollTick;
        WindowDecorations = Avalonia.Controls.WindowDecorations.Full;
        Icon = new WindowIcon(Avalonia.Platform.AssetLoader.Open(new Uri("avares://SunCode/Assets/logo/suncode-logo-128.png")));
        AddHandler(KeyDownEvent, WindowKeyDown, RoutingStrategies.Tunnel);
        DataContextChanged += (_, _) => RebindViewModelSubscriptions();
        Opened += async (_, _) =>
        {
            RebindViewModelSubscriptions();
            await ViewModel.LoadProjectToolCallLimitAsync();
            RefreshModelSelector();
            AppearancePage.ThemeSelectorControl.ItemsSource = ThemeOptions;
            AppearancePage.ThemeSelectorControl.SelectedItem = ThemeOptions.FirstOrDefault(item => Equals(item.Value, ViewModel.ThemeMode));
            LoggingPage.LogLevelSelectorControl.ItemsSource = LogLevelOptions;
            LoggingPage.LogLevelSelectorControl.SelectedItem = LogLevelOptions.FirstOrDefault(item => Equals(item.Value, ViewModel.LogLevel));
            DefaultsPage.ToolCallLimit.Value = ViewModel.ToolCallLimit;
            DefaultsPage.ToolCallLimit.IsEnabled = ViewModel.IsProjectOpen;
            DefaultsPage.ToolCallLimitScopeText.Text = ViewModel.SelectedProject is { } project
                ? $"Project: {project.DisplayName}"
                : "Open a project to configure this setting.";
            _baselineToolCallLimit = ViewModel.ToolCallLimit;
            LoggingPage.LogDirectoryInputControl.Text = ViewModel.EffectiveLogDirectory;
            LoggingPage.ImageDirectoryInputControl.Text = ViewModel.EffectiveImageDirectory;
            LoggingPage.LogMaxMegabytesInputControl.Value = Math.Max(1, ViewModel.LogMaxBytes / (1024 * 1024));
            LoggingPage.LogRetentionInputControl.Value = ViewModel.LogRetention;
            _baselineLogLevel = ViewModel.LogLevel;
            _baselineLogDirectory = NormalizeDirectory(ViewModel.EffectiveLogDirectory);
            _baselineLogMaxBytes = ViewModel.LogMaxBytes;
            _baselineLogRetention = ViewModel.LogRetention;
            _baselineImageDirectory = NormalizeDirectory(ViewModel.EffectiveImageDirectory);
            NetworkPage.VerifyHttpsCertificatesToggleControl.IsChecked = ViewModel.VerifyHttpsCertificates;
            NetworkPage.UseSystemCertificatesToggleControl.IsChecked = ViewModel.UseSystemCertificates;
            NetworkPage.CertificatePathInputControl.Text = ViewModel.CertificatePath;
            NetworkPage.CertificatePathInputControl.IsEnabled = ViewModel.UseSystemCertificates == false;
            NetworkPage.ProxyModeSelectorControl.ItemsSource = ProxyModeOptions;
            NetworkPage.ProxyModeSelectorControl.SelectedItem = ProxyModeOptions.FirstOrDefault(item => Equals(item.Value, ViewModel.ProxyMode));
            NetworkPage.ProxyUrlInputControl.Text = ViewModel.ProxyUrl;
            NetworkPage.ProxyUsernameInputControl.Text = ViewModel.ProxyUsername;
            NetworkPage.ProxyPasswordInputControl.Text = string.Empty;
            NetworkPage.ProxyBypassInputControl.Text = ViewModel.ProxyBypassRules;
            _baselineProxyMode = ViewModel.ProxyMode;
            _baselineProxyUrl = ViewModel.ProxyUrl;
            _baselineProxyUsername = ViewModel.ProxyUsername;
            _baselineProxyBypass = NormalizeProxyBypass(ViewModel.ProxyBypassRules);
            _baselineProxyPasswordConfigured = ViewModel.ProxyPasswordConfigured;
            _clearProxyPassword = false;
            _baselineVerifyHttpsCertificates = ViewModel.VerifyHttpsCertificates;
            _baselineUseSystemCertificates = ViewModel.UseSystemCertificates;
            _baselineCertificatePath = ViewModel.CertificatePath ?? string.Empty;
            RefreshHttpsCertificateWarning();
            RefreshCertificateTrustPresentation();
            RefreshProxyPresentation();
            LoggingPage.LoggingStatusText.Text = "Local settings";
            LoggingPage.ImageDirectoryStatusText.Text = "Local settings";
            RefreshDefaultsDirtyState();
            RefreshLoggingDirtyState();
            RefreshImageDirectoryDirtyState();
            RefreshHttpsDirtyState();
            RefreshProxyDirtyState();
            ProvidersChevron.RenderTransform = new Avalonia.Media.RotateTransform(_providersExpanded ? 90 : 0);
            AgentsChevron.RenderTransform = new Avalonia.Media.RotateTransform(_agentsExpanded ? 90 : 0);
            var savedNavigation = ViewModel.SavedSettingsNavigation;
            SetProvidersExpanded(savedNavigation.ProvidersExpanded);
            SetAgentsExpanded(savedNavigation.AgentsExpanded);
            if (savedNavigation.Page == "providers")
            {
                ShowProviderPanel(savedNavigation.ProviderId);
                SelectPage("providers", null);
            }
            else if (savedNavigation.Page == "agents")
            {
                ShowAgentPanel(savedNavigation.AgentId);
                SelectPage("agents", null);
            }
            else
            {
                ShowProviderPanel(null);
                SelectPage(savedNavigation.Page, null);
            }
            _ready = true;
        };
        Closed += (_, _) =>
        {
            _mcpPollTimer.Stop();
            _languageServerPollTimer.Stop();
            _mcpEditorWindow?.Close();
            _languageServerEditorWindow?.Close();
        };
    }

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;

    private void WindowKeyDown(object? sender, KeyEventArgs e)
    {
        if (e.Key != Key.Escape) return;
        e.Handled = true;
        Close();
    }

    private void CloseSettings(object? sender, RoutedEventArgs e) => Close();
    private void ShowDefaults(object? sender, RoutedEventArgs e) => SelectPage("defaults", sender as Button);
    private void ShowAppearance(object? sender, RoutedEventArgs e) => SelectPage("appearance", sender as Button);
    private void ShowShortcuts(object? sender, RoutedEventArgs e) => SelectPage("shortcuts", sender as Button);
    private void ShowNetwork(object? sender, RoutedEventArgs e) => SelectPage("network", sender as Button);
    private void ShowMcp(object? sender, RoutedEventArgs e) => SelectPage("mcp", sender as Button);
    private void ShowLanguageServers(object? sender, RoutedEventArgs e) => SelectPage("lsp", sender as Button);
    private void ShowLogging(object? sender, RoutedEventArgs e) => SelectPage("logging", sender as Button);

    private void ShowAgents(object? sender, RoutedEventArgs e)
    {
        ShowAgentPanel(null);
        SelectPage("agents", sender as Button);
        SetAgentsExpanded(!_agentsExpanded);
    }

    private void SetAgentsExpanded(bool expanded)
    {
        _agentsExpanded = expanded;
        AgentNavigation.IsVisible = _agentsExpanded;
        AgentsChevron.RenderTransform = new Avalonia.Media.RotateTransform(_agentsExpanded ? 90 : 0);
        if (_ready) ViewModel.SaveSettingsNavigation(CurrentPage(), _provider, _providersExpanded, _agent, _agentsExpanded);
    }

    private void ShowAgent(object? sender, RoutedEventArgs e)
    {
        if (sender is not Button { Tag: string agent }) return;
        ShowAgentPanel(agent);
        SelectPage("agents", sender as Button);
    }

    private void AgentSelected(object? sender, string agent)
    {
        ShowAgentPanel(agent);
        var navigation = this.GetVisualDescendants().OfType<Button>()
            .FirstOrDefault(button => button.Classes.Contains("navigation") && Equals(button.Tag, agent));
        SelectPage("agents", navigation);
    }

    private void ShowAgentPanel(string? agent)
    {
        _agent = agent ?? string.Empty;
        AgentsPage.SelectedAgentId = agent;
    }

    private void ShowProviders(object? sender, RoutedEventArgs e)
    {
        ShowProviderPanel(null);
        SelectPage("providers", sender as Button);
        SetProvidersExpanded(!_providersExpanded);
    }

    private void SetProvidersExpanded(bool expanded)
    {
        _providersExpanded = expanded;
        ProviderNavigation.IsVisible = _providersExpanded;
        ProvidersChevron.RenderTransform = new Avalonia.Media.RotateTransform(_providersExpanded ? 90 : 0);
        if (_ready) ViewModel.SaveSettingsNavigation(CurrentPage(), _provider, _providersExpanded, _agent, _agentsExpanded);
    }

    private void ShowProvider(object? sender, RoutedEventArgs e)
    {
        if (sender is not Button { Tag: string provider }) return;
        ShowProviderPanel(provider);
        SelectPage("providers", sender as Button);
    }

    private void ProviderSelected(object? sender, string provider)
    {
        ShowProviderPanel(provider);
        var navigation = this.GetVisualDescendants()
            .OfType<Button>()
            .FirstOrDefault(button => button.Classes.Contains("navigation") && Equals(button.Tag, provider));
        SelectPage("providers", navigation);
    }

    private void ShowProviderPanel(string? provider)
    {
        if (!string.IsNullOrWhiteSpace(provider))
        {
            _provider = provider;
        }

        ProviderManager.SelectedProviderId = provider;
        if (string.IsNullOrWhiteSpace(provider))
        {
            return;
        }

        ProviderManager.EndpointText = ViewModel.ProviderEndpoint(provider);
        ProviderManager.EndpointStatusText = string.Empty;
        ProviderManager.EndpointStatusBrush = this.FindResource("TextSecondaryBrush") as IBrush;
        ProviderManager.ApiKeyText = string.Empty;
        var providerName = ViewModel.Providers.FirstOrDefault(item => item.Id == provider)?.DisplayName ?? provider;
        ProviderManager.ApiKeyPlaceholderText = $"Paste {providerName} API key";
        RefreshProvider();
    }

    private void SelectPage(string page, Button? selected)
    {
        DefaultsPage.IsVisible = page == "defaults";
        AppearancePage.IsVisible = page == "appearance";
        ShortcutsPage.IsVisible = page == "shortcuts";
        NetworkPage.IsVisible = page == "network";
        LoggingPage.IsVisible = page == "logging";
        McpPage.IsVisible = page == "mcp";
        LanguageServersPage.IsVisible = page == "lsp";
        AgentsPage.IsVisible = page == "agents";
        ProvidersPage.IsVisible = page == "providers";
        if (page == "mcp")
        {
            _ = ViewModel.LoadMcpServersAsync();
            _mcpPollTimer.Start();
        }
        else
        {
            _mcpPollTimer.Stop();
        }
        if (page == "lsp")
        {
            _ = ViewModel.StartLanguageServerProjectAsync();
            _ = ViewModel.LoadLanguageServersAsync();
            _languageServerPollTimer.Start();
        }
        else
        {
            _languageServerPollTimer.Stop();
        }
        foreach (var button in this.GetVisualDescendants().OfType<Button>().Where(button => button.Classes.Contains("navigation")))
            button.Classes.Set("selected", button == selected);
        if (page == "defaults") DefaultsNavigation.Classes.Set("selected", true);
        if (page == "appearance") AppearanceNavigation.Classes.Set("selected", true);
        if (page == "shortcuts") ShortcutsNavigation.Classes.Set("selected", true);
        if (page == "network") NetworkNavigation.Classes.Set("selected", true);
        if (page == "logging") LoggingNavigation.Classes.Set("selected", true);
        if (page == "mcp") McpNavigation.Classes.Set("selected", true);
        if (page == "lsp") LanguageServersNavigation.Classes.Set("selected", true);
        if (page == "agents" && selected is null) AgentsNavigation.Classes.Set("selected", true);
        if (page == "providers" && selected is null) ProvidersNavigation.Classes.Set("selected", true);
        if (_ready) ViewModel.SaveSettingsNavigation(page, _provider, _providersExpanded, _agent, _agentsExpanded);
    }

    private string CurrentPage() => DefaultsPage.IsVisible ? "defaults"
        : AppearancePage.IsVisible ? "appearance"
        : ShortcutsPage.IsVisible ? "shortcuts"
        : NetworkPage.IsVisible ? "network"
        : McpPage.IsVisible ? "mcp"
        : LanguageServersPage.IsVisible ? "lsp"
        : AgentsPage.IsVisible ? "agents"
        : LoggingPage.IsVisible ? "logging"
        : "providers";

    private async void McpPollTick(object? sender, EventArgs e)
    {
        if (McpPage.IsVisible) await ViewModel.LoadMcpServersAsync();
    }

    private async void LanguageServerPollTick(object? sender, EventArgs e)
    {
        if (LanguageServersPage.IsVisible) await ViewModel.LoadLanguageServersAsync();
    }

    private void AddMcpServer() => OpenMcpEditor(null);

    private void EditMcpServer(McpServerItem server) => OpenMcpEditor(server);

    private void OpenMcpEditor(McpServerItem? server)
    {
        if (_mcpEditorWindow is not null)
        {
            _mcpEditorWindow.Activate();
            return;
        }
        _mcpEditorWindow = new McpServerEditorWindow(ViewModel, server);
        _mcpEditorWindow.Closed += (_, _) =>
        {
            _mcpEditorWindow = null;
            McpNavigation.Focus();
        };
        _mcpEditorWindow.Show(this);
    }

    private void DeleteMcpServer(McpServerItem server)
    {
        var dialog = new ConfirmationWindow(
            "Delete MCP server?",
            "Its tools will be removed from new model requests. An already executing call may finish.",
            server.DisplayName,
            () => _ = ViewModel.DeleteMcpServerAsync(server),
            "MCP SERVER",
            "Delete server");
        IsEnabled = false;
        dialog.Closed += (_, _) => IsEnabled = true;
        _ = dialog.ShowDialog(this);
    }

    private void AddLanguageServer() => OpenLanguageServerEditor(null);

    private void EditLanguageServer(LanguageServerItem server) => OpenLanguageServerEditor(server);

    private void OpenLanguageServerEditor(LanguageServerItem? server)
    {
        if (_languageServerEditorWindow is not null)
        {
            _languageServerEditorWindow.Activate();
            return;
        }
        _languageServerEditorWindow = new LanguageServerEditorWindow(ViewModel, server);
        _languageServerEditorWindow.Closed += (_, _) =>
        {
            _languageServerEditorWindow = null;
            LanguageServersNavigation.Focus();
        };
        _languageServerEditorWindow.Show(this);
    }

    private void DeleteLanguageServer(LanguageServerItem server)
    {
        var dialog = new ConfirmationWindow(
            "Delete language server?",
            "Its project runtimes will stop and semantic results will no longer be available to new agent turns.",
            server.DisplayName,
            () => _ = ViewModel.DeleteLanguageServerAsync(server),
            "LANGUAGE SERVER",
            "Delete server");
        IsEnabled = false;
        dialog.Closed += (_, _) => IsEnabled = true;
        _ = dialog.ShowDialog(this);
    }

    private async void DefaultModelChanged(object? sender, SelectionChangedEventArgs e)
    {
        if (_ready && DefaultsPage.ModelSelector.SelectedItem?.Value is ModelItem model) await ViewModel.SaveDefaultModelAsync(model);
    }

    private async void ThemeChanged(object? sender, SelectionChangedEventArgs e)
    {
        if (!_ready || AppearancePage.ThemeSelectorControl.SelectedItem?.Value is not string mode) return;
        await ViewModel.SaveThemeAsync(mode);
    }

    private void LogLevelChanged(object? sender, SelectionChangedEventArgs e)
    {
        if (!_ready) return;
        RefreshLoggingDirtyState();
    }
    
    private void LoggingDirectoryChanged(object? sender, TextChangedEventArgs e)
    {
        if (!_ready) return;
        RefreshLoggingDirtyState();
    }
    
    private void LoggingMaxMegabytesChanged(object? sender, AvaloniaPropertyChangedEventArgs e)
    {
        if (!_ready) return;
        RefreshLoggingDirtyState();
    }
    
    private void LoggingRetentionChanged(object? sender, AvaloniaPropertyChangedEventArgs e)
    {
        if (!_ready) return;
        RefreshLoggingDirtyState();
    }
    
    private void ImageDirectoryChanged(object? sender, TextChangedEventArgs e)
    {
        if (!_ready) return;
        RefreshImageDirectoryDirtyState();
    }
    
    private void ToolCallLimitValueChanged(object? sender, AvaloniaPropertyChangedEventArgs e)
    {
        if (!_ready) return;
        RefreshDefaultsDirtyState();
    }

    private async void SaveLogging(object? sender, RoutedEventArgs e)
    {
        var level = LoggingPage.LogLevelSelectorControl.SelectedItem?.Value as string ?? ViewModel.LogLevel;
        if (LoggingPage.LogMaxMegabytesInputControl.Value is not { } megabytesValue
            || megabytesValue is < 1 or > 1000
            || decimal.Truncate(megabytesValue) != megabytesValue
            || LoggingPage.LogRetentionInputControl.Value is not { } retentionValue
            || retentionValue is < 0 or > 100
            || decimal.Truncate(retentionValue) != retentionValue)
        {
            LoggingPage.LoggingStatusText.Text = "Log size must be 1–1000 MB and retained backups must be 0–100";
            LoggingPage.LoggingStatusText.Foreground = this.FindResource("DangerBrush") as IBrush;
            return;
        }
        var saved = await ViewModel.SaveLoggingSettingsAsync(
            level,
            LoggingPage.LogDirectoryInputControl.Text,
            checked(decimal.ToInt64(megabytesValue) * 1024 * 1024).ToString(System.Globalization.CultureInfo.InvariantCulture),
            decimal.ToInt32(retentionValue).ToString(System.Globalization.CultureInfo.InvariantCulture));
        LoggingPage.LoggingStatusText.Text = ViewModel.StatusText;
        LoggingPage.LoggingStatusText.Foreground = this.FindResource(saved ? "SuccessBrush" : "DangerBrush") as IBrush;
        if (saved)
        {
            _baselineLogLevel = ViewModel.LogLevel;
            _baselineLogDirectory = NormalizeDirectory(ViewModel.EffectiveLogDirectory);
            _baselineLogMaxBytes = ViewModel.LogMaxBytes;
            _baselineLogRetention = ViewModel.LogRetention;
        }
        RefreshLoggingDirtyState();
    }

    private async void SaveImageDirectory(object? sender, RoutedEventArgs e)
    {
        var saved = await ViewModel.SaveImageDirectoryAsync(LoggingPage.ImageDirectoryInputControl.Text);
        LoggingPage.ImageDirectoryStatusText.Text = ViewModel.StatusText;
        LoggingPage.ImageDirectoryStatusText.Foreground = this.FindResource(saved ? "SuccessBrush" : "DangerBrush") as IBrush;
        if (saved)
        {
            _baselineImageDirectory = NormalizeDirectory(ViewModel.EffectiveImageDirectory);
        }
        RefreshImageDirectoryDirtyState();
    }

    private void HttpsCertificateVerificationChanged(object? sender, RoutedEventArgs e)
    {
        RefreshCertificateTrustPresentation();
        if (_ready) RefreshHttpsDirtyState();
    }

    private void SystemCertificatesChanged(object? sender, RoutedEventArgs e)
    {
        ViewModel.UseSystemCertificates = NetworkPage.UseSystemCertificatesToggleControl.IsChecked == true;
        NetworkPage.CertificatePathInputControl.IsEnabled = !ViewModel.UseSystemCertificates;
        RefreshCertificateTrustPresentation();
        if (_ready) RefreshHttpsDirtyState();
    }
    
    private void CertificatePathChanged(object? sender, TextChangedEventArgs e)
    {
        if (_ready) RefreshHttpsDirtyState();
    }

    private void ProxyModeChanged(object? sender, SelectionChangedEventArgs e)
    {
        NetworkPage.ProxyStatusText.Text = string.Empty;
        RefreshProxyPresentation();
        if (_ready) RefreshProxyDirtyState();
    }

    private void ProxyTextChanged(object? sender, TextChangedEventArgs e)
    {
        if (!string.IsNullOrEmpty(NetworkPage.ProxyPasswordInputControl.Text))
        {
            _clearProxyPassword = false;
            RefreshProxyPresentation();
        }
        if (_ready) RefreshProxyDirtyState();
    }

    private void RemoveProxyPassword(object? sender, RoutedEventArgs e)
    {
        _clearProxyPassword = true;
        NetworkPage.ProxyPasswordInputControl.Text = string.Empty;
        RefreshProxyPresentation();
        RefreshProxyDirtyState();
    }

    private async void SaveProxy(object? sender, RoutedEventArgs e)
    {
        var mode = NetworkPage.ProxyModeSelectorControl.SelectedItem?.Value as string ?? "system";
        var saved = await ViewModel.SaveProxyConfigurationAsync(
            mode,
            NetworkPage.ProxyUrlInputControl.Text,
            NetworkPage.ProxyUsernameInputControl.Text,
            NetworkPage.ProxyPasswordInputControl.Text,
            _clearProxyPassword,
            NetworkPage.ProxyBypassInputControl.Text);
        NetworkPage.ProxyStatusText.Text = ViewModel.StatusText;
        NetworkPage.ProxyStatusText.Foreground = this.FindResource(saved ? "SuccessBrush" : "DangerBrush") as IBrush;
        if (saved)
        {
            _baselineProxyMode = ViewModel.ProxyMode;
            _baselineProxyUrl = ViewModel.ProxyUrl;
            _baselineProxyUsername = ViewModel.ProxyUsername;
            _baselineProxyBypass = NormalizeProxyBypass(ViewModel.ProxyBypassRules);
            _baselineProxyPasswordConfigured = ViewModel.ProxyPasswordConfigured;
            _clearProxyPassword = false;
            NetworkPage.ProxyPasswordInputControl.Text = string.Empty;
            NetworkPage.ProxyBypassInputControl.Text = ViewModel.ProxyBypassRules;
        }
        RefreshProxyPresentation();
        RefreshProxyDirtyState();
    }

    private void RefreshProxyPresentation()
    {
        var mode = NetworkPage.ProxyModeSelectorControl.SelectedItem?.Value as string ?? "system";
        NetworkPage.CustomProxySectionControl.IsVisible = mode == "custom";
        var passwordConfigured = _baselineProxyPasswordConfigured && !_clearProxyPassword;
        NetworkPage.RemoveProxyPasswordButtonControl.IsVisible = passwordConfigured;
        NetworkPage.ProxyPasswordInputControl.PlaceholderText = passwordConfigured ? "Password stored" : "Optional";
        NetworkPage.ProxyPasswordHintText.Text = passwordConfigured
            ? "Password stored. Leave empty to keep it or remove it explicitly."
            : "Optional Basic proxy authentication password.";
        if (string.IsNullOrWhiteSpace(NetworkPage.ProxyStatusText.Text))
        {
            NetworkPage.ProxyStatusText.Text = mode switch
            {
                "no_proxy" => "Direct connections",
                "custom" => "Custom proxy",
                _ => "System proxy"
            };
        }
    }

    private async void SaveHttpsCertificateVerification(object? sender, RoutedEventArgs e)
    {
        var enabled = NetworkPage.VerifyHttpsCertificatesToggleControl.IsChecked == true;
        var saved = await ViewModel.SaveHttpsCertificateVerificationAsync(enabled);
        saved = await ViewModel.SaveCertificateTrustAsync(NetworkPage.UseSystemCertificatesToggleControl.IsChecked == true, NetworkPage.CertificatePathInputControl.Text) && saved;
        NetworkPage.HttpsCertificateStatusText.Text = ViewModel.StatusText;
        NetworkPage.HttpsCertificateStatusText.Foreground = this.FindResource(saved ? "SuccessBrush" : "DangerBrush") as IBrush;
        if (!saved)
        {
            NetworkPage.VerifyHttpsCertificatesToggleControl.IsChecked = ViewModel.VerifyHttpsCertificates;
        }
        else
        {
            _baselineVerifyHttpsCertificates = ViewModel.VerifyHttpsCertificates;
            _baselineUseSystemCertificates = ViewModel.UseSystemCertificates;
            _baselineCertificatePath = ViewModel.CertificatePath ?? string.Empty;
        }
        RefreshHttpsCertificateWarning();
        RefreshHttpsDirtyState();
    }

    private void RefreshHttpsCertificateWarning()
    {
        var verify = NetworkPage.VerifyHttpsCertificatesToggleControl.IsChecked == true;
        NetworkPage.HttpsCertificateWarningControl.IsVisible = !verify;
        NetworkPage.CertificateTrustSectionControl.IsVisible = verify;
    }

    private void RefreshCertificateTrustPresentation()
    {
        var useSystemCertificates = NetworkPage.UseSystemCertificatesToggleControl.IsChecked == true;
        NetworkPage.CertificatePathHintText.Text = useSystemCertificates
            ? "Disable system certificates to provide a custom certificate file."
            : "Choose a PEM, CRT, CER, or DER certificate file for custom trust.";
        RefreshHttpsCertificateWarning();
        NetworkPage.HttpsCertificateStatusText.Text = NetworkPage.VerifyHttpsCertificatesToggleControl.IsChecked == true
            ? useSystemCertificates
                ? "System trust store"
                : "Custom certificate required"
            : "Review required";
    }

    private async void SaveToolCallLimit(object? sender, RoutedEventArgs e)
    {
        if (DefaultsPage.ToolCallLimit.Value is not { } value) return;
        var saved = await ViewModel.SaveProjectToolCallLimitAsync(decimal.ToInt32(value));
        DefaultsPage.ToolCallLimitStatusText.Text = ViewModel.StatusText;
        DefaultsPage.ToolCallLimitStatusText.Foreground = this.FindResource(saved ? "SuccessBrush" : "DangerBrush") as IBrush;
        if (saved) _baselineToolCallLimit = ViewModel.ToolCallLimit;
        RefreshDefaultsDirtyState();
    }

    private async void SaveCredential(object? sender, RoutedEventArgs e)
    {
        var value = ProviderManager.ApiKeyText?.Trim();
        if (string.IsNullOrWhiteSpace(value)) return;
        await ViewModel.SaveCredentialAsync(_provider, value);
        ProviderManager.ApiKeyText = string.Empty;
        RefreshProvider();
    }

    private void ProviderApiKeyChanged(object? sender, TextChangedEventArgs e) =>
        ProviderManager.CanSaveCredential = !string.IsNullOrWhiteSpace(ProviderManager.ApiKeyText);

    private void ProviderEndpointChanged(object? sender, TextChangedEventArgs e) =>
        RefreshProviderEndpointActions();

    private async void SaveProviderEndpoint(object? sender, RoutedEventArgs e)
    {
        var saved = await ViewModel.SaveProviderEndpointAsync(_provider, ProviderManager.EndpointText);
        ProviderManager.EndpointStatusText = ViewModel.StatusText;
        ProviderManager.EndpointStatusBrush = this.FindResource(saved ? "SuccessBrush" : "DangerBrush") as IBrush;
        if (saved) ProviderManager.EndpointText = ViewModel.ProviderEndpoint(_provider);
        RefreshProviderEndpointActions();
    }

    private async void ResetProviderEndpoint(object? sender, RoutedEventArgs e)
    {
        var defaultEndpoint = ViewModel.Providers.FirstOrDefault(item => item.Id == _provider)?.DefaultApiBase ?? string.Empty;
        if (string.IsNullOrWhiteSpace(defaultEndpoint))
        {
            defaultEndpoint = ProviderManager.EndpointText?.Trim() ?? string.Empty;
        }
        if (string.IsNullOrWhiteSpace(defaultEndpoint)) return;
        ProviderManager.EndpointText = defaultEndpoint;
        var saved = await ViewModel.SaveProviderEndpointAsync(_provider, defaultEndpoint);
        ProviderManager.EndpointStatusText = ViewModel.StatusText;
        ProviderManager.EndpointStatusBrush = this.FindResource(saved ? "SuccessBrush" : "DangerBrush") as IBrush;
        if (saved) ProviderManager.EndpointText = ViewModel.ProviderEndpoint(_provider);
        RefreshProviderEndpointActions();
    }

    private async void RemoveCredential(object? sender, RoutedEventArgs e)
    {
        await ViewModel.RemoveCredentialAsync(_provider);
        RefreshProvider();
    }

    private void RefreshProvider()
    {
        var configured = ViewModel.IsProviderConfigured(_provider);
        ProviderManager.CredentialConfigured = configured;
        ProviderManager.CredentialStatusText = configured
            ? "API key configured"
            : "No API key configured";
        ProviderManager.CanRemoveCredential = configured;
        ProviderManager.CanSaveCredential = !string.IsNullOrWhiteSpace(ProviderManager.ApiKeyText);
        RefreshProviderEndpointActions();
        ProviderManager.ProviderModels = ViewModel.Models
            .Where(item => item.Provider == _provider)
            .Select(item => new ProviderModelItem(item.Display, configured))
            .ToArray();
    }

    private void RefreshProviderEndpointActions()
    {
        var endpoint = ProviderManager.EndpointText?.Trim() ?? string.Empty;
        var savedEndpoint = ViewModel.ProviderEndpoint(_provider).Trim();
        var defaultEndpoint = ViewModel.Providers.FirstOrDefault(item => item.Id == _provider)?.DefaultApiBase ?? string.Empty;
        ProviderManager.CanSaveEndpoint = endpoint.Length > 0
            && !string.Equals(endpoint, savedEndpoint, StringComparison.Ordinal);
        ProviderManager.CanResetEndpoint = defaultEndpoint.Length > 0
            && !string.Equals(endpoint.TrimEnd('/'), defaultEndpoint.TrimEnd('/'), StringComparison.Ordinal);
    }

    private void RebindViewModelSubscriptions()
    {
        if (_subscribedViewModel is not null)
        {
            _subscribedViewModel.Models.CollectionChanged -= ModelsCollectionChanged;
        }

        _subscribedViewModel = DataContext as DesktopViewModel;
        if (_subscribedViewModel is not null)
        {
            _subscribedViewModel.Models.CollectionChanged += ModelsCollectionChanged;
        }
    }

    private void ModelsCollectionChanged(object? sender, NotifyCollectionChangedEventArgs e)
    {
        RefreshModelSelector();
        if (!string.IsNullOrWhiteSpace(ProviderManager.SelectedProviderId))
        {
            RefreshProvider();
        }
    }

    private void RefreshModelSelector()
    {
        if (DataContext is not DesktopViewModel viewModel) return;
        var items = viewModel.Models.Select(model => new SCComboBoxItem(model.Id, model)).ToArray();
        DefaultsPage.ModelSelector.ItemsSource = items;
        DefaultsPage.ModelSelector.SelectedItem = items.FirstOrDefault(item => item.Value is ModelItem model && model.Id == viewModel.SelectedModel?.Id);
    }
    
    private static string NormalizeDirectory(string? directory) =>
        directory?.Trim().TrimEnd(Path.DirectorySeparatorChar, Path.AltDirectorySeparatorChar) ?? string.Empty;

    private static string NormalizeProxyBypass(string? value) => string.Join('\n',
        (value ?? string.Empty)
            .Split(['\r', '\n'], StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries)
            .Where(item => item.Length > 0));

    private void RefreshDefaultsDirtyState()
    {
        if (!ViewModel.IsProjectOpen)
        {
            DefaultsPage.SaveToolCallLimitButtonControl.IsEnabled = false;
            return;
        }

        var current = (int?)DefaultsPage.ToolCallLimit.Value;
        DefaultsPage.SaveToolCallLimitButtonControl.IsEnabled = 
            current.HasValue && current.Value != _baselineToolCallLimit;
    }

    private void RefreshLoggingDirtyState()
    {
        var level = LoggingPage.LogLevelSelectorControl.SelectedItem?.Value as string ?? ViewModel.LogLevel;
        var directory = NormalizeDirectory(LoggingPage.LogDirectoryInputControl.Text);
        var megabytes = LoggingPage.LogMaxMegabytesInputControl.Value is { } mbValue 
            ? checked(decimal.ToInt64(mbValue) * 1024 * 1024) 
            : (long?)null;
        var retention = LoggingPage.LogRetentionInputControl.Value is { } retentionValue 
            ? decimal.ToInt32(retentionValue) 
            : (int?)null;
        LoggingPage.SaveLoggingButtonControl.IsEnabled =
            !string.Equals(level, _baselineLogLevel, StringComparison.Ordinal)
            || !string.Equals(directory, _baselineLogDirectory, StringComparison.Ordinal)
            || (megabytes.HasValue && megabytes.Value != _baselineLogMaxBytes)
            || (retention.HasValue && retention.Value != _baselineLogRetention);
    }
    
    private void RefreshImageDirectoryDirtyState()
    {
        var directory = NormalizeDirectory(LoggingPage.ImageDirectoryInputControl.Text);
        LoggingPage.SaveImageDirectoryButtonControl.IsEnabled =
            !string.Equals(directory, _baselineImageDirectory, StringComparison.Ordinal);
    }
    
    private void RefreshHttpsDirtyState()
    {
        var verify = NetworkPage.VerifyHttpsCertificatesToggleControl.IsChecked == true;
        var useSystem = NetworkPage.UseSystemCertificatesToggleControl.IsChecked == true;
        var path = (NetworkPage.CertificatePathInputControl.Text ?? string.Empty).Trim();
        NetworkPage.SaveHttpsCertificateButtonControl.IsEnabled =
            verify != _baselineVerifyHttpsCertificates
            || useSystem != _baselineUseSystemCertificates
            || !string.Equals(path, _baselineCertificatePath, StringComparison.Ordinal);
    }

    private void RefreshProxyDirtyState()
    {
        var mode = NetworkPage.ProxyModeSelectorControl.SelectedItem?.Value as string ?? "system";
        var url = NetworkPage.ProxyUrlInputControl.Text?.Trim() ?? string.Empty;
        var username = NetworkPage.ProxyUsernameInputControl.Text?.Trim() ?? string.Empty;
        var bypass = NormalizeProxyBypass(NetworkPage.ProxyBypassInputControl.Text);
        var passwordChanged = !string.IsNullOrEmpty(NetworkPage.ProxyPasswordInputControl.Text);
        NetworkPage.SaveProxyButtonControl.IsEnabled =
            !string.Equals(mode, _baselineProxyMode, StringComparison.Ordinal)
            || !string.Equals(url, _baselineProxyUrl, StringComparison.Ordinal)
            || !string.Equals(username, _baselineProxyUsername, StringComparison.Ordinal)
            || !string.Equals(bypass, _baselineProxyBypass, StringComparison.Ordinal)
            || passwordChanged
            || _clearProxyPassword;
    }
}
