using System.Collections.Specialized;
using System.IO;
using System.Linq;
using Avalonia.Controls;
using Avalonia;
using Avalonia.Input;
using Avalonia.Input.Platform;
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
    private readonly DispatcherTimer _computerPollTimer = new() { Interval = TimeSpan.FromSeconds(2) };
    private readonly DispatcherTimer _browserPollTimer = new() { Interval = TimeSpan.FromSeconds(2) };
    private bool _refreshingComputerPage;
    private bool _refreshingBrowserPage;
    
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
        ComputerPage.EnabledChanged += ComputerEnabledChanged;
        ComputerPage.CapturePermissionRequested += RequestComputerCapturePermission;
        ComputerPage.InputPermissionRequested += RequestComputerInputPermission;
        ComputerPage.TakeControlRequested += TakeComputerControl;
        ComputerPage.ReturnControlRequested += ReturnComputerControl;
        ComputerPage.EmergencyStopRequested += EmergencyStopComputerUse;
        BrowserPage.EnabledChanged += BrowserEnabledChanged;
        BrowserPage.VerifyRequested += VerifyBrowserRuntime;
        BrowserPage.StartRequested += StartBrowserRuntime;
        BrowserPage.TakeControlRequested += TakeBrowserControl;
        BrowserPage.ReturnControlRequested += ReturnBrowserControl;
        BrowserPage.RestartRequested += RestartBrowserRuntime;
        BrowserPage.StopRequested += StopBrowserRuntime;
        BrowserPage.ClearRequested += ClearBrowserProfile;
        BrowserPage.CopyNodePathRequested += CopyBrowserNodePath;
        BrowserPage.CopyChromiumPathRequested += CopyBrowserChromiumPath;
        BrowserPage.CopyProfilePathRequested += CopyBrowserProfilePath;
        McpPage.AddRequested += AddMcpServer;
        McpPage.EditRequested += EditMcpServer;
        McpPage.DeleteRequested += DeleteMcpServer;
        LanguageServersPage.AddRequested += AddLanguageServer;
        LanguageServersPage.EditRequested += EditLanguageServer;
        LanguageServersPage.DeleteRequested += DeleteLanguageServer;
        _mcpPollTimer.Tick += McpPollTick;
        _languageServerPollTimer.Tick += LanguageServerPollTick;
        _computerPollTimer.Tick += ComputerPollTick;
        _browserPollTimer.Tick += BrowserPollTick;
        WindowDecorations = Avalonia.Controls.WindowDecorations.Full;
        Icon = new WindowIcon(Avalonia.Platform.AssetLoader.Open(new Uri("avares://SunCode/Assets/logo/suncode-logo-128.png")));
        AddHandler(KeyDownEvent, WindowKeyDown, RoutingStrategies.Tunnel);
        DataContextChanged += (_, _) => RebindViewModelSubscriptions();
        Opened += async (_, _) =>
        {
            RebindViewModelSubscriptions();
            await ViewModel.LoadProjectToolCallLimitAsync();
            await ViewModel.LoadComputerRuntimeAsync();
            RefreshComputerPresentation();
            await ViewModel.LoadBrowserRuntimeAsync();
            RefreshBrowserPresentation();
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
            _computerPollTimer.Stop();
            _browserPollTimer.Stop();
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
    private void ShowComputer(object? sender, RoutedEventArgs e) => SelectPage("computer", sender as Button);
    private void ShowBrowser(object? sender, RoutedEventArgs e) => SelectPage("browser", sender as Button);
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
        ComputerPage.IsVisible = page == "computer";
        BrowserPage.IsVisible = page == "browser";
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
        if (page == "computer")
        {
            _ = LoadAndRefreshComputerAsync();
            _computerPollTimer.Start();
        }
        else
        {
            _computerPollTimer.Stop();
        }
        if (page == "browser")
        {
            _ = LoadAndRefreshBrowserAsync();
            _browserPollTimer.Start();
        }
        else
        {
            _browserPollTimer.Stop();
        }
        foreach (var button in this.GetVisualDescendants().OfType<Button>().Where(button => button.Classes.Contains("navigation")))
            button.Classes.Set("selected", button == selected);
        if (page == "defaults") DefaultsNavigation.Classes.Set("selected", true);
        if (page == "appearance") AppearanceNavigation.Classes.Set("selected", true);
        if (page == "shortcuts") ShortcutsNavigation.Classes.Set("selected", true);
        if (page == "network") NetworkNavigation.Classes.Set("selected", true);
        if (page == "computer") ComputerNavigation.Classes.Set("selected", true);
        if (page == "browser") BrowserNavigation.Classes.Set("selected", true);
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
        : ComputerPage.IsVisible ? "computer"
        : BrowserPage.IsVisible ? "browser"
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

    private async void ComputerPollTick(object? sender, EventArgs e)
    {
        if (ComputerPage.IsVisible) await LoadAndRefreshComputerAsync();
    }

    private async Task LoadAndRefreshComputerAsync()
    {
        await ViewModel.LoadComputerRuntimeAsync();
        RefreshComputerPresentation();
    }

    private async void ComputerEnabledChanged(object? sender, RoutedEventArgs e)
    {
        if (!_ready || _refreshingComputerPage) return;
        await ViewModel.SetComputerUseEnabledAsync(ComputerPage.EnabledToggleControl.IsChecked == true);
        RefreshComputerPresentation();
    }

    private void RefreshComputerPresentation()
    {
        var runtime = ViewModel.ComputerRuntime;
        if (runtime is null) return;
        _refreshingComputerPage = true;
        try
        {
            var model = ViewModel.SelectedModel;
            var modelSupported = model?.SupportsComputerUse == true;
            ComputerPage.EnabledToggleControl.IsChecked = runtime.Enabled;
            ComputerPage.ModelSupportTextControl.Text = model is null
                ? "No model selected"
                : modelSupported ? "Supported" : "Not supported";
            ComputerPage.ModelNameTextControl.Text = model?.Id ?? "No model selected";
            ComputerPage.ModelStatusDotControl.Fill = this.FindResource(
                modelSupported ? "SuccessBrush" : model is null ? "TextMutedBrush" : "WarningBrush") as IBrush;
            ComputerPage.BackendStateTextControl.Text = !runtime.Enabled
                ? "Disabled"
                : runtime.BackendAvailable ? "Ready" : "Unavailable";
            ComputerPage.BackendStatusDotControl.Fill = this.FindResource(
                !runtime.Enabled ? "TextMutedBrush" : runtime.BackendAvailable ? "SuccessBrush" : "DangerBrush") as IBrush;
            ComputerPage.TargetDisplayTextControl.Text = runtime.PixelWidth is { } width && runtime.PixelHeight is { } height
                ? $"{FormatComputerValue(runtime.TargetDisplay)} · {width} × {height} px"
                : FormatComputerValue(runtime.TargetDisplay);
            ComputerPage.CapturePermissionTextControl.Text = FormatComputerValue(runtime.CapturePermission);
            ComputerPage.CapturePermissionDotControl.Fill = ComputerPermissionBrush(runtime.CapturePermission);
            ComputerPage.CapturePermissionButtonControl.IsEnabled = runtime.CapturePermission == "denied";
            ComputerPage.CapturePermissionButtonControl.Content = runtime.CapturePermission == "allowed"
                ? "Screen capture allowed"
                : "Request screen capture";
            ComputerPage.InputPermissionTextControl.Text = FormatComputerValue(runtime.InputPermission);
            ComputerPage.InputPermissionDotControl.Fill = ComputerPermissionBrush(runtime.InputPermission);
            ComputerPage.InputPermissionButtonControl.IsEnabled = runtime.InputPermission == "denied";
            ComputerPage.InputPermissionButtonControl.Content = runtime.InputPermission == "allowed"
                ? "Input control allowed"
                : "Request input control";
            ComputerPage.ControlOwnerTextControl.Text = FormatComputerValue(runtime.ControlOwner);
            var agentControlled = runtime.ControlOwner == "agent";
            ComputerPage.TakeControlButtonControl.IsVisible = runtime.Enabled && agentControlled;
            ComputerPage.TakeControlButtonControl.IsEnabled = runtime.BackendAvailable;
            ComputerPage.ReturnControlButtonControl.IsVisible = runtime.Enabled && !agentControlled;
            ComputerPage.ReturnControlButtonControl.IsEnabled = runtime.BackendAvailable;
            ComputerPage.StatusTextControl.Text = string.IsNullOrWhiteSpace(ViewModel.ComputerStatusText)
                ? runtime.Error ?? string.Empty
                : ViewModel.ComputerStatusText;
            ComputerPage.EmergencyStopButtonControl.IsEnabled = runtime.Enabled && runtime.BackendAvailable;
        }
        finally
        {
            _refreshingComputerPage = false;
        }
    }

    private static string FormatComputerValue(string value) => string.IsNullOrWhiteSpace(value)
        ? "—"
        : string.Join(
            " ",
            value.Split('_', StringSplitOptions.RemoveEmptyEntries)
                .Select((part, index) => index == 0
                    ? char.ToUpperInvariant(part[0]) + part[1..]
                    : part));

    private IBrush? ComputerPermissionBrush(string permission) => permission switch
    {
        "allowed" => this.FindResource("SuccessBrush") as IBrush,
        "denied" => this.FindResource("DangerBrush") as IBrush,
        "unsupported" => this.FindResource("WarningBrush") as IBrush,
        _ => this.FindResource("TextMutedBrush") as IBrush
    };

    private async void EmergencyStopComputerUse(object? sender, RoutedEventArgs e)
    {
        await ViewModel.EmergencyStopComputerUseAsync();
        RefreshComputerPresentation();
    }

    private async void RequestComputerCapturePermission(object? sender, RoutedEventArgs e)
    {
        await ViewModel.RequestComputerCapturePermissionAsync();
        RefreshComputerPresentation();
    }

    private async void RequestComputerInputPermission(object? sender, RoutedEventArgs e)
    {
        await ViewModel.RequestComputerInputPermissionAsync();
        RefreshComputerPresentation();
    }

    private async void TakeComputerControl(object? sender, RoutedEventArgs e)
    {
        await ViewModel.TakeComputerControlAsync();
        RefreshComputerPresentation();
    }

    private async void ReturnComputerControl(object? sender, RoutedEventArgs e)
    {
        await ViewModel.ReturnComputerControlAsync();
        RefreshComputerPresentation();
    }

    private async void BrowserPollTick(object? sender, EventArgs e)
    {
        if (BrowserPage.IsVisible) await LoadAndRefreshBrowserAsync();
    }

    private async Task LoadAndRefreshBrowserAsync()
    {
        await ViewModel.LoadBrowserRuntimeAsync();
        RefreshBrowserPresentation();
    }

    private async void BrowserEnabledChanged(object? sender, RoutedEventArgs e)
    {
        if (!_ready || _refreshingBrowserPage) return;
        await ViewModel.SetBrowserUseEnabledAsync(BrowserPage.EnabledToggleControl.IsChecked == true);
        RefreshBrowserPresentation();
    }

    private async void VerifyBrowserRuntime(object? sender, RoutedEventArgs e)
    {
        await ViewModel.VerifyBrowserRuntimeAsync();
        RefreshBrowserPresentation();
    }

    private async void StartBrowserRuntime(object? sender, RoutedEventArgs e)
    {
        await ViewModel.StartBrowserProjectAsync();
        RefreshBrowserPresentation();
    }

    private async void TakeBrowserControl(object? sender, RoutedEventArgs e)
    {
        await ViewModel.TakeBrowserControlAsync();
        RefreshBrowserPresentation();
    }

    private async void ReturnBrowserControl(object? sender, RoutedEventArgs e)
    {
        await ViewModel.ReturnBrowserControlAsync();
        RefreshBrowserPresentation();
    }

    private async void RestartBrowserRuntime(object? sender, RoutedEventArgs e)
    {
        await ViewModel.RestartBrowserRuntimeAsync();
        RefreshBrowserPresentation();
    }

    private async void StopBrowserRuntime(object? sender, RoutedEventArgs e)
    {
        await ViewModel.StopBrowserRuntimeAsync();
        RefreshBrowserPresentation();
    }

    private void ClearBrowserProfile(object? sender, RoutedEventArgs e)
    {
        if (ViewModel.SelectedProject is not { } project) return;
        var dialog = new ConfirmationWindow(
            "Clear browser data?",
            "Saved logins, cookies, site storage, and browsing state for this project will be removed. Project files are unchanged.",
            project.DisplayName,
            () => _ = ClearBrowserProfileConfirmedAsync(),
            "PROJECT BROWSER PROFILE",
            "Clear browser data");
        IsEnabled = false;
        dialog.Closed += (_, _) => IsEnabled = true;
        _ = dialog.ShowDialog(this);
    }

    private async Task ClearBrowserProfileConfirmedAsync()
    {
        await ViewModel.ClearBrowserProfileAsync();
        RefreshBrowserPresentation();
    }

    private async void CopyBrowserNodePath(object? sender, RoutedEventArgs e) =>
        await CopyBrowserValueAsync(ViewModel.BrowserRuntime?.NodePath);

    private async void CopyBrowserChromiumPath(object? sender, RoutedEventArgs e) =>
        await CopyBrowserValueAsync(ViewModel.BrowserRuntime?.ChromiumPath);

    private async void CopyBrowserProfilePath(object? sender, RoutedEventArgs e) =>
        await CopyBrowserValueAsync(ViewModel.BrowserRuntime?.ProfilePath);

    private async Task CopyBrowserValueAsync(string? value)
    {
        if (string.IsNullOrWhiteSpace(value) || TopLevel.GetTopLevel(this)?.Clipboard is not { } clipboard) return;
        await clipboard.SetTextAsync(value);
        BrowserPage.StatusTextControl.Text = "Copied to clipboard.";
    }

    private void RefreshBrowserPresentation()
    {
        var runtime = ViewModel.BrowserRuntime;
        if (runtime is null) return;
        _refreshingBrowserPage = true;
        try
        {
            BrowserPage.EnabledToggleControl.IsChecked = runtime.Enabled;
            BrowserPage.InstallationStateTextControl.Text = FormatBrowserState(runtime.InstallationState);
            BrowserPage.RuntimeStateTextControl.Text = FormatBrowserState(runtime.RuntimeState);
            BrowserPage.InstallationStatusDotControl.Fill = BrowserStateBrush(runtime.InstallationState);
            BrowserPage.RuntimeStatusDotControl.Fill = BrowserStateBrush(runtime.RuntimeState);
            BrowserPage.RuntimeScopeTextControl.Text = ViewModel.SelectedProject is { } project
                ? $"Project: {project.DisplayName}"
                : "No project selected";
            BrowserPage.TargetTextControl.Text = EmptyAsDash(runtime.Target);
            BrowserPage.NodeVersionTextControl.Text = EmptyAsDash(runtime.NodeVersion);
            BrowserPage.NodePathTextControl.Text = EmptyAsDash(runtime.NodePath);
            ToolTip.SetTip(BrowserPage.NodePathTextControl, runtime.NodePath);
            BrowserPage.CopyNodePathButtonControl.IsEnabled = !string.IsNullOrWhiteSpace(runtime.NodePath);
            BrowserPage.PlaywrightVersionTextControl.Text = EmptyAsDash(runtime.PlaywrightVersion);
            BrowserPage.ChromiumVersionTextControl.Text = string.IsNullOrWhiteSpace(runtime.ChromiumRevision)
                ? EmptyAsDash(runtime.ChromiumVersion)
                : $"{runtime.ChromiumVersion} · revision {runtime.ChromiumRevision}";
            BrowserPage.ChromiumPathTextControl.Text = EmptyAsDash(runtime.ChromiumPath);
            ToolTip.SetTip(BrowserPage.ChromiumPathTextControl, runtime.ChromiumPath);
            BrowserPage.CopyChromiumPathButtonControl.IsEnabled = !string.IsNullOrWhiteSpace(runtime.ChromiumPath);
            BrowserPage.WorkerProtocolTextControl.Text = runtime.WorkerProtocolVersion.ToString();
            BrowserPage.IntegrityTextControl.Text = runtime.IntegrityState switch
            {
                "verified" => "Integrity verified",
                "unverified" => "Integrity not yet verified",
                _ => "Integrity unavailable"
            };
            var hasProject = ViewModel.SelectedProject is not null;
            BrowserPage.ProjectSectionControl.IsVisible = hasProject;
            BrowserPage.NoProjectSectionControl.IsVisible = !hasProject;
            BrowserPage.ProfilePathTextControl.Text = EmptyAsDash(runtime.ProfilePath);
            ToolTip.SetTip(BrowserPage.ProfilePathTextControl, runtime.ProfilePath);
            BrowserPage.CopyProfilePathButtonControl.IsEnabled = !string.IsNullOrWhiteSpace(runtime.ProfilePath);
            BrowserPage.ProfileUsageTextControl.Text = runtime.ProfileSizeBytes is { } bytes
                ? $"{FormatByteSize(bytes)} · {runtime.ActivePageCount} active pages"
                : "—";
            BrowserPage.VisibilityTextControl.Text = FormatBrowserState(runtime.VisibilityCapability);
            var userControlled = runtime.RuntimeState == "user_controlled";
            BrowserPage.ControlTitleTextControl.Text = userControlled
                ? "You control Chromium"
                : "Agent control is active";
            BrowserPage.ControlHintTextControl.Text = userControlled
                ? "Browser tools are paused. Returning control invalidates previous element references."
                : "Showing the browser transfers exclusive control to you and pauses browser tools.";
            BrowserPage.ErrorTextControl.Text = runtime.Error ?? string.Empty;
            BrowserPage.ErrorTextControl.IsVisible = !string.IsNullOrWhiteSpace(runtime.Error);
            BrowserPage.StatusTextControl.Text = ViewModel.BrowserStatusText;
            var installationReady = runtime.Enabled && runtime.InstallationState == "ready";
            var notStarted = runtime.RuntimeState == "not_started";
            BrowserPage.VerifyButtonControl.IsEnabled = runtime.Enabled;
            BrowserPage.StartButtonControl.IsVisible = hasProject && notStarted;
            BrowserPage.StartButtonControl.IsEnabled = installationReady;
            BrowserPage.TakeControlButtonControl.IsVisible = hasProject && runtime.RuntimeState == "background";
            BrowserPage.TakeControlButtonControl.IsEnabled = installationReady;
            BrowserPage.ReturnControlButtonControl.IsVisible = hasProject && userControlled;
            BrowserPage.RestartButtonControl.IsEnabled = installationReady && !notStarted && !userControlled;
            BrowserPage.StopButtonControl.IsEnabled = !notStarted;
            BrowserPage.ClearButtonControl.IsEnabled = hasProject && notStarted;
        }
        finally
        {
            _refreshingBrowserPage = false;
        }
    }

    private static string FormatBrowserState(string value) => string.Join(
        " ",
        value.Split('_', StringSplitOptions.RemoveEmptyEntries)
            .Select((part, index) => index == 0
                ? char.ToUpperInvariant(part[0]) + part[1..]
                : part));

    private static string EmptyAsDash(string? value) => string.IsNullOrWhiteSpace(value) ? "—" : value;

    private IBrush? BrowserStateBrush(string state) => state switch
    {
        "ready" or "background" => this.FindResource("SuccessBrush") as IBrush,
        "verifying" or "starting" or "stopping" or "user_controlled" => this.FindResource("WarningBrush") as IBrush,
        "missing" or "invalid" or "unsupported" or "failed" => this.FindResource("DangerBrush") as IBrush,
        _ => this.FindResource("TextMutedBrush") as IBrush
    };

    private static string FormatByteSize(ulong value)
    {
        if (value >= 1024UL * 1024UL * 1024UL) return $"{value / (1024d * 1024d * 1024d):0.0} GB";
        if (value >= 1024UL * 1024UL) return $"{value / (1024d * 1024d):0.0} MB";
        if (value >= 1024UL) return $"{value / 1024d:0.0} KB";
        return $"{value} B";
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
        if (_ready && DefaultsPage.ModelSelector.SelectedItem?.Value is ModelItem model)
        {
            await ViewModel.SaveDefaultModelAsync(model);
            RefreshComputerPresentation();
        }
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
        RefreshComputerPresentation();
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
