using System.Collections.Specialized;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Media;
using Avalonia.VisualTree;
using SunCode.Desktop.Controls;
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
    private string _provider = "deepseek";
    private DesktopViewModel? _subscribedViewModel;

    public SettingsWindow()
    {
        InitializeComponent();
        WindowDecorations = Avalonia.Controls.WindowDecorations.Full;
        Icon = new WindowIcon(Avalonia.Platform.AssetLoader.Open(new Uri("avares://SunCode/Assets/logo/suncode-logo-128.png")));
        AddHandler(KeyDownEvent, WindowKeyDown, RoutingStrategies.Tunnel);
        DataContextChanged += (_, _) => RebindViewModelSubscriptions();
        Opened += async (_, _) =>
        {
            RebindViewModelSubscriptions();
            await ViewModel.LoadProjectToolCallLimitAsync();
            RefreshModelSelector();
            ThemeSelector.ItemsSource = ThemeOptions;
            ThemeSelector.SelectedItem = ThemeOptions.FirstOrDefault(item => Equals(item.Value, ViewModel.ThemeMode));
            LogLevelSelector.ItemsSource = LogLevelOptions;
            LogLevelSelector.SelectedItem = LogLevelOptions.FirstOrDefault(item => Equals(item.Value, ViewModel.LogLevel));
            ToolCallLimitInput.Value = ViewModel.ToolCallLimit;
            ToolCallLimitInput.IsEnabled = ViewModel.IsProjectOpen;
            SaveToolCallLimitButton.IsEnabled = ViewModel.IsProjectOpen;
            ToolCallLimitScope.Text = ViewModel.SelectedProject is { } project
                ? $"Project: {project.DisplayName}"
                : "Open a project to configure this setting.";
            LogDirectoryInput.Text = ViewModel.LogDirectory;
            ImageDirectoryInput.Text = ViewModel.ImageDirectory;
            LogMaxMegabytesInput.Text = Math.Max(1, ViewModel.LogMaxBytes / (1024 * 1024))
                .ToString(System.Globalization.CultureInfo.InvariantCulture);
            LogRetentionInput.Text = ViewModel.LogRetention.ToString(System.Globalization.CultureInfo.InvariantCulture);
            VerifyHttpsCertificatesToggle.IsChecked = ViewModel.VerifyHttpsCertificates;
            UseSystemCertificatesToggle.IsChecked = ViewModel.UseSystemCertificates;
            CertificatePathInput.Text = ViewModel.CertificatePath;
            CertificatePathInput.IsEnabled = ViewModel.UseSystemCertificates == false;
            RefreshHttpsCertificateWarning();
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
    }

    private void ToggleProviders(object? sender, RoutedEventArgs e)
    {
        _providersExpanded = !_providersExpanded;
        ProviderNavigation.IsVisible = _providersExpanded;
        ProvidersChevron.RenderTransform = new Avalonia.Media.RotateTransform(_providersExpanded ? 90 : 0);
    }

    private void ShowProvider(object? sender, RoutedEventArgs e)
    {
        if (sender is not Button { Tag: string provider }) return;
        ShowProviderPanel(provider);
        SelectPage("providers", ProvidersNavigation);
    }

    private void ProviderSelected(object? sender, string provider)
    {
        ShowProviderPanel(provider);
        SelectPage("providers", ProvidersNavigation);
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
        ProviderManager.ApiKeyPlaceholderText = SCProviderCatalog.GetOrDefault(provider).ApiKeyPlaceholder;
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
        if (page == "providers") ProvidersNavigation.Classes.Set("selected", true);
    }

    private async void DefaultModelChanged(object? sender, SelectionChangedEventArgs e)
    {
        if (_ready && DefaultModelSelector.SelectedItem?.Value is ModelItem model) await ViewModel.SaveDefaultModelAsync(model);
    }

    private async void ThemeChanged(object? sender, SelectionChangedEventArgs e)
    {
        if (!_ready || ThemeSelector.SelectedItem?.Value is not string mode) return;
        await ViewModel.SaveThemeAsync(mode);
    }

    private void LogLevelChanged(object? sender, SelectionChangedEventArgs e)
    {
    }

    private async void SaveLogging(object? sender, RoutedEventArgs e)
    {
        var level = LogLevelSelector.SelectedItem?.Value as string ?? ViewModel.LogLevel;
        if (!long.TryParse(LogMaxMegabytesInput.Text?.Trim(), System.Globalization.NumberStyles.Integer, System.Globalization.CultureInfo.InvariantCulture, out var megabytes)
            || megabytes is < 1 or > 1000)
        {
            LoggingStatus.Text = "Maximum log size must be between 1 and 1000 MB";
            LoggingStatus.Foreground = this.FindResource("DangerBrush") as IBrush;
            return;
        }
        var saved = await ViewModel.SaveLoggingSettingsAsync(
            level,
            LogDirectoryInput.Text,
            checked(megabytes * 1024 * 1024).ToString(System.Globalization.CultureInfo.InvariantCulture),
            LogRetentionInput.Text ?? string.Empty);
        LoggingStatus.Text = ViewModel.StatusText;
        LoggingStatus.Foreground = this.FindResource(saved ? "SuccessBrush" : "DangerBrush") as IBrush;
    }

    private async void SaveImageDirectory(object? sender, RoutedEventArgs e)
    {
        var saved = await ViewModel.SaveImageDirectoryAsync(ImageDirectoryInput.Text);
        ImageDirectoryStatus.Text = ViewModel.StatusText;
        ImageDirectoryStatus.Foreground = this.FindResource(saved ? "SuccessBrush" : "DangerBrush") as IBrush;
    }

    private void HttpsCertificateVerificationChanged(object? sender, RoutedEventArgs e) =>
        RefreshHttpsCertificateWarning();

    private void SystemCertificatesChanged(object? sender, RoutedEventArgs e)
    {
        ViewModel.UseSystemCertificates = UseSystemCertificatesToggle.IsChecked == true;
        CertificatePathInput.IsEnabled = !ViewModel.UseSystemCertificates;
    }

    private async void SaveHttpsCertificateVerification(object? sender, RoutedEventArgs e)
    {
        var enabled = VerifyHttpsCertificatesToggle.IsChecked == true;
        var saved = await ViewModel.SaveHttpsCertificateVerificationAsync(enabled);
        saved = await ViewModel.SaveCertificateTrustAsync(UseSystemCertificatesToggle.IsChecked == true, CertificatePathInput.Text) && saved;
        HttpsCertificateStatus.Text = ViewModel.StatusText;
        HttpsCertificateStatus.Foreground = this.FindResource(saved ? "SuccessBrush" : "DangerBrush") as IBrush;
        if (!saved)
        {
            VerifyHttpsCertificatesToggle.IsChecked = ViewModel.VerifyHttpsCertificates;
        }
        RefreshHttpsCertificateWarning();
    }

    private void RefreshHttpsCertificateWarning() =>
        HttpsCertificateWarning.IsVisible = VerifyHttpsCertificatesToggle.IsChecked != true;

    private async void SaveToolCallLimit(object? sender, RoutedEventArgs e)
    {
        if (ToolCallLimitInput.Value is not { } value) return;
        var saved = await ViewModel.SaveProjectToolCallLimitAsync(decimal.ToInt32(value));
        ToolCallLimitStatus.Text = ViewModel.StatusText;
        ToolCallLimitStatus.Foreground = this.FindResource(saved ? "SuccessBrush" : "DangerBrush") as IBrush;
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
        ProviderManager.CanSaveEndpoint = !string.IsNullOrWhiteSpace(ProviderManager.EndpointText)
            && !string.Equals(ProviderManager.EndpointText?.Trim(), ViewModel.ProviderEndpoint(_provider), StringComparison.Ordinal);

    private async void SaveProviderEndpoint(object? sender, RoutedEventArgs e)
    {
        var saved = await ViewModel.SaveProviderEndpointAsync(_provider, ProviderManager.EndpointText);
        ProviderManager.EndpointStatusText = ViewModel.StatusText;
        ProviderManager.EndpointStatusBrush = this.FindResource(saved ? "SuccessBrush" : "DangerBrush") as IBrush;
        if (saved) ProviderManager.EndpointText = ViewModel.ProviderEndpoint(_provider);
        ProviderManager.CanSaveEndpoint = !saved;
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
            ? "API key configured in the local agent credential store."
            : "No API key configured.";
        ProviderManager.CanRemoveCredential = configured;
        ProviderManager.CanSaveCredential = !string.IsNullOrWhiteSpace(ProviderManager.ApiKeyText);
        ProviderManager.CanSaveEndpoint = !string.IsNullOrWhiteSpace(ProviderManager.EndpointText)
            && !string.Equals(ProviderManager.EndpointText?.Trim(), ViewModel.ProviderEndpoint(_provider), StringComparison.Ordinal);
        ProviderManager.ProviderModelsText = ViewModel.ProviderModels(_provider);
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
        DefaultModelSelector.ItemsSource = items;
        DefaultModelSelector.SelectedItem = items.FirstOrDefault(item => item.Value is ModelItem model && model.Id == viewModel.SelectedModel?.Id);
    }
}
