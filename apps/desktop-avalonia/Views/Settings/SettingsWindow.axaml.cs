using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Styling;
using Avalonia.VisualTree;
using SvgControl = Avalonia.Svg.Skia.Svg;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.Settings;

public sealed partial class SettingsWindow : Window
{
    private static readonly IReadOnlyDictionary<string, (string Title, string Description, string Placeholder)> Providers =
        new Dictionary<string, (string, string, string)>
        {
            ["deepseek"] = ("DeepSeek", "Configure the credential used by the local DeepSeek provider.", "Paste DeepSeek API key"),
            ["zhipu"] = ("Zhipu GLM", "Configure the credential used by the local Zhipu GLM provider.", "Paste Zhipu API key"),
            ["openai"] = ("OpenAI", "Configure the credential used by the local OpenAI provider.", "Paste OpenAI API key"),
            ["kimi"] = ("Kimi", "Configure the credential used by the local Kimi provider.", "Paste Kimi API key"),
            ["claude"] = ("Claude", "Configure the credential used by the local Claude provider.", "Paste Anthropic API key"),
            ["gemini"] = ("Gemini", "Configure the credential used by the local Gemini provider.", "Paste Gemini API key")
        };

    private bool _ready;
    private bool _providersExpanded = true;
    private string _provider = "deepseek";

    public SettingsWindow()
    {
        InitializeComponent();
        WindowDecorations = OperatingSystem.IsMacOS()
            ? Avalonia.Controls.WindowDecorations.BorderOnly
            : Avalonia.Controls.WindowDecorations.None;
        Icon = new WindowIcon(Avalonia.Platform.AssetLoader.Open(new Uri("avares://SunCode/Assets/logo/suncode-logo-128.png")));
        AddHandler(KeyDownEvent, WindowKeyDown, RoutingStrategies.Tunnel);
        Opened += (_, _) =>
        {
            ThemeSelector.SelectedIndex = ViewModel.ThemeMode == "light" ? 1 : 0;
            DefaultModelSelector.SelectedItem = ViewModel.SelectedModel;
            LogLevelSelector.SelectedItem = LogLevelSelector.Items
                .OfType<ComboBoxItem>()
                .FirstOrDefault(item => item.Tag as string == ViewModel.LogLevel);
            LogDirectoryInput.Text = ViewModel.LogDirectory;
            LogMaxBytesInput.Text = ViewModel.LogMaxBytes.ToString(System.Globalization.CultureInfo.InvariantCulture);
            LogRetentionInput.Text = ViewModel.LogRetention.ToString(System.Globalization.CultureInfo.InvariantCulture);
            ProvidersChevron.RenderTransform = new Avalonia.Media.RotateTransform(_providersExpanded ? 90 : 0);
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
    private void MinimizeSettings(object? sender, RoutedEventArgs e) => WindowState = WindowState.Minimized;
    private void ToggleSettingsMaximized(object? sender, RoutedEventArgs e) =>
        WindowState = WindowState == WindowState.Maximized ? WindowState.Normal : WindowState.Maximized;

    private void SettingsTitleBarPressed(object? sender, PointerPressedEventArgs e)
    {
        if (e.GetCurrentPoint(this).Properties.IsLeftButtonPressed && !OriginatesFromButton(e.Source)) BeginMoveDrag(e);
    }

    private void SettingsTitleBarDoubleTapped(object? sender, TappedEventArgs e)
    {
        if (OriginatesFromButton(e.Source)) return;
        ToggleSettingsMaximized(sender, new RoutedEventArgs());
        e.Handled = true;
    }

    private static bool OriginatesFromButton(object? source) =>
        source is Button || source is Avalonia.Visual visual && visual.FindAncestorOfType<Button>() is not null;

    private void TrafficLightEntered(object? sender, PointerEventArgs e) => SetTrafficLightState(sender, "hover");
    private void TrafficLightExited(object? sender, PointerEventArgs e) => SetTrafficLightState(sender, "normal");
    private void TrafficLightPressed(object? sender, PointerPressedEventArgs e) => SetTrafficLightState(sender, "press");
    private void TrafficLightReleased(object? sender, PointerReleasedEventArgs e) => SetTrafficLightState(sender, "hover");

    private static void SetTrafficLightState(object? sender, string state)
    {
        if (sender is not Button button || button.GetVisualDescendants().OfType<SvgControl>().FirstOrDefault() is not { } icon) return;
        var kind = button.Name?.Contains("Close", StringComparison.Ordinal) == true
            ? "close"
            : button.Name?.Contains("Minimize", StringComparison.Ordinal) == true ? "minimize" : "maximize";
        var file = (kind, state) switch
        {
            ("close", "hover") => "2-close-2-hover.svg",
            ("close", "press") => "2-close-3-press.svg",
            ("close", _) => "1-close-1-normal.svg",
            ("minimize", "hover") => "2-minimize-2-hover.svg",
            ("minimize", "press") => "2-minimize-3-press.svg",
            ("minimize", _) => "2-minimize-1-normal.svg",
            ("maximize", "hover") => "3-maximize-2-hover.svg",
            ("maximize", "press") => "3-maximize-3-press.svg",
            _ => "3-maximize-1-normal.svg"
        };
        icon.Path = $"/Assets/traffic-lights/{file}";
    }
    private void ShowDefaults(object? sender, RoutedEventArgs e) => SelectPage("defaults", sender as Button);
    private void ShowAppearance(object? sender, RoutedEventArgs e) => SelectPage("appearance", sender as Button);
    private void ShowLogging(object? sender, RoutedEventArgs e) => SelectPage("logging", sender as Button);

    private void ToggleProviders(object? sender, RoutedEventArgs e)
    {
        _providersExpanded = !_providersExpanded;
        ProviderNavigation.IsVisible = _providersExpanded;
        ProvidersChevron.RenderTransform = new Avalonia.Media.RotateTransform(_providersExpanded ? 90 : 0);
    }

    private void ShowProvider(object? sender, RoutedEventArgs e)
    {
        if (sender is not Button { Tag: string provider } || !Providers.TryGetValue(provider, out var copy)) return;
        _provider = provider;
        ProviderTitle.Text = copy.Title;
        ProviderDescription.Text = copy.Description;
        ProviderApiKey.PlaceholderText = copy.Placeholder;
        ProviderApiKey.Text = string.Empty;
        RefreshProvider();
        SelectPage("provider", sender as Button);
    }

    private void SelectPage(string page, Button? selected)
    {
        DefaultsPage.IsVisible = page == "defaults";
        AppearancePage.IsVisible = page == "appearance";
        LoggingPage.IsVisible = page == "logging";
        ProviderPage.IsVisible = page == "provider";
        foreach (var button in this.GetVisualDescendants().OfType<Button>().Where(button => button.Classes.Contains("navigation")))
            button.Classes.Set("selected", button == selected);
        if (page == "defaults") DefaultsNavigation.Classes.Set("selected", true);
        if (page == "appearance") AppearanceNavigation.Classes.Set("selected", true);
        if (page == "logging") LoggingNavigation.Classes.Set("selected", true);
    }

    private async void DefaultModelChanged(object? sender, SelectionChangedEventArgs e)
    {
        if (_ready && DefaultModelSelector.SelectedItem is ModelItem model) await ViewModel.SaveDefaultModelAsync(model);
    }

    private async void ThemeChanged(object? sender, SelectionChangedEventArgs e)
    {
        if (!_ready || ThemeSelector.SelectedItem is not ComboBoxItem { Tag: string mode }) return;
        await ViewModel.SaveThemeAsync(mode);
    }

    private async void SaveLogging(object? sender, RoutedEventArgs e)
    {
        var level = (LogLevelSelector.SelectedItem as ComboBoxItem)?.Tag as string ?? ViewModel.LogLevel;
        var saved = await ViewModel.SaveLoggingSettingsAsync(
            level,
            LogDirectoryInput.Text,
            LogMaxBytesInput.Text ?? string.Empty,
            LogRetentionInput.Text ?? string.Empty);
        LoggingStatus.Text = ViewModel.StatusText;
        LoggingStatus.Foreground = this.FindResource(saved ? "SuccessBrush" : "DangerBrush") as Avalonia.Media.IBrush;
    }

    private async void SaveCredential(object? sender, RoutedEventArgs e)
    {
        var value = ProviderApiKey.Text?.Trim();
        if (string.IsNullOrWhiteSpace(value)) return;
        await ViewModel.SaveCredentialAsync(_provider, value);
        ProviderApiKey.Text = string.Empty;
        RefreshProvider();
    }

    private void ProviderApiKeyChanged(object? sender, TextChangedEventArgs e) =>
        SaveCredentialButton.IsEnabled = !string.IsNullOrWhiteSpace(ProviderApiKey.Text);

    private async void RemoveCredential(object? sender, RoutedEventArgs e)
    {
        await ViewModel.RemoveCredentialAsync(_provider);
        RefreshProvider();
    }

    private void RefreshProvider()
    {
        var configured = ViewModel.IsProviderConfigured(_provider);
        CredentialStatus.Text = configured
            ? "API key configured in the local runtime credential store."
            : "No API key configured.";
        CredentialStatus.Foreground = this.FindResource(configured ? "SuccessBrush" : "WarningBrush") as Avalonia.Media.IBrush;
        RemoveCredentialButton.IsEnabled = configured;
        SaveCredentialButton.IsEnabled = !string.IsNullOrWhiteSpace(ProviderApiKey.Text);
        ProviderModelsText.Text = ViewModel.ProviderModels(_provider);
    }
}
