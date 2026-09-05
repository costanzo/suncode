using System.Collections.Specialized;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Media;
using Avalonia.VisualTree;
using SunCode.Desktop.Controls;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

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

    private bool _ready;
    private bool _providersExpanded = true;
    private string _provider = string.Empty;
    private DesktopViewModel? _subscribedViewModel;

    public SettingsWindow()
    {
        InitializeComponent();
        DefaultsPage.DefaultModelChanged += DefaultModelChanged;
        DefaultsPage.SaveToolCallLimitRequested += SaveToolCallLimit;
        AppearancePage.ThemeChanged += ThemeChanged;
        LoggingPage.LogLevelChanged += LogLevelChanged;
        LoggingPage.SaveLoggingRequested += SaveLogging;
        LoggingPage.SaveImageDirectoryRequested += SaveImageDirectory;
        NetworkPage.HttpsCertificateVerificationChanged += HttpsCertificateVerificationChanged;
        NetworkPage.SystemCertificatesChanged += SystemCertificatesChanged;
        NetworkPage.SaveHttpsCertificateVerificationRequested += SaveHttpsCertificateVerification;
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
            DefaultsPage.SaveToolCallLimitButtonControl.IsEnabled = ViewModel.IsProjectOpen;
            DefaultsPage.ToolCallLimitScopeText.Text = ViewModel.SelectedProject is { } project
                ? $"Project: {project.DisplayName}"
                : "Open a project to configure this setting.";
            LoggingPage.LogDirectoryInputControl.Text = ViewModel.EffectiveLogDirectory;
            LoggingPage.ImageDirectoryInputControl.Text = ViewModel.EffectiveImageDirectory;
            LoggingPage.LogMaxMegabytesInputControl.Value = Math.Max(1, ViewModel.LogMaxBytes / (1024 * 1024));
            LoggingPage.LogRetentionInputControl.Value = ViewModel.LogRetention;
            NetworkPage.VerifyHttpsCertificatesToggleControl.IsChecked = ViewModel.VerifyHttpsCertificates;
            NetworkPage.UseSystemCertificatesToggleControl.IsChecked = ViewModel.UseSystemCertificates;
            NetworkPage.CertificatePathInputControl.Text = ViewModel.CertificatePath;
            NetworkPage.CertificatePathInputControl.IsEnabled = ViewModel.UseSystemCertificates == false;
            RefreshHttpsCertificateWarning();
            RefreshCertificateTrustPresentation();
            LoggingPage.LoggingStatusText.Text = "Local settings";
            LoggingPage.ImageDirectoryStatusText.Text = "Local settings";
            ProvidersChevron.RenderTransform = new Avalonia.Media.RotateTransform(_providersExpanded ? 90 : 0);
            ShowProviderPanel(null);
            _ready = true;
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
    private void ShowNetwork(object? sender, RoutedEventArgs e) => SelectPage("network", sender as Button);
    private void ShowLogging(object? sender, RoutedEventArgs e) => SelectPage("logging", sender as Button);

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
        NetworkPage.IsVisible = page == "network";
        LoggingPage.IsVisible = page == "logging";
        ProvidersPage.IsVisible = page == "providers";
        foreach (var button in this.GetVisualDescendants().OfType<Button>().Where(button => button.Classes.Contains("navigation")))
            button.Classes.Set("selected", button == selected);
        if (page == "defaults") DefaultsNavigation.Classes.Set("selected", true);
        if (page == "appearance") AppearanceNavigation.Classes.Set("selected", true);
        if (page == "network") NetworkNavigation.Classes.Set("selected", true);
        if (page == "logging") LoggingNavigation.Classes.Set("selected", true);
        if (page == "providers" && selected is null) ProvidersNavigation.Classes.Set("selected", true);
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
    }

    private async void SaveImageDirectory(object? sender, RoutedEventArgs e)
    {
        var saved = await ViewModel.SaveImageDirectoryAsync(LoggingPage.ImageDirectoryInputControl.Text);
        LoggingPage.ImageDirectoryStatusText.Text = ViewModel.StatusText;
        LoggingPage.ImageDirectoryStatusText.Foreground = this.FindResource(saved ? "SuccessBrush" : "DangerBrush") as IBrush;
    }

    private void HttpsCertificateVerificationChanged(object? sender, RoutedEventArgs e) =>
        RefreshCertificateTrustPresentation();

    private void SystemCertificatesChanged(object? sender, RoutedEventArgs e)
    {
        ViewModel.UseSystemCertificates = NetworkPage.UseSystemCertificatesToggleControl.IsChecked == true;
        NetworkPage.CertificatePathInputControl.IsEnabled = !ViewModel.UseSystemCertificates;
        RefreshCertificateTrustPresentation();
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
        RefreshHttpsCertificateWarning();
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
}
