using Avalonia;
using Avalonia.Controls;
using SunCode.Desktop.Infrastructure;
using Avalonia.Interactivity;
using Avalonia.Media;
using SunCode.Desktop.Models;

namespace SunCode.Desktop.Views.Settings.Controls;

public sealed partial class ModelProviderSettingsControl : UserControl
{
    public static readonly StyledProperty<IEnumerable<ProviderItem>?> ProvidersProperty =
        AvaloniaProperty.Register<ModelProviderSettingsControl, IEnumerable<ProviderItem>?>(nameof(Providers));

    public static readonly StyledProperty<string?> SelectedProviderIdProperty =
        AvaloniaProperty.Register<ModelProviderSettingsControl, string?>(nameof(SelectedProviderId));

    public static readonly StyledProperty<string?> EndpointTextProperty =
        AvaloniaProperty.Register<ModelProviderSettingsControl, string?>(nameof(EndpointText), defaultBindingMode: Avalonia.Data.BindingMode.TwoWay);

    public static readonly StyledProperty<string?> EndpointStatusTextProperty =
        AvaloniaProperty.Register<ModelProviderSettingsControl, string?>(nameof(EndpointStatusText));

    public static readonly StyledProperty<string?> CredentialStatusTextProperty =
        AvaloniaProperty.Register<ModelProviderSettingsControl, string?>(nameof(CredentialStatusText));

    public static readonly StyledProperty<string> CredentialStatusProperty =
        AvaloniaProperty.Register<ModelProviderSettingsControl, string>(nameof(CredentialStatus), "missing");

    public static readonly StyledProperty<string?> ApiKeyPlaceholderTextProperty =
        AvaloniaProperty.Register<ModelProviderSettingsControl, string?>(nameof(ApiKeyPlaceholderText));

    public static readonly StyledProperty<string?> ApiKeyTextProperty =
        AvaloniaProperty.Register<ModelProviderSettingsControl, string?>(nameof(ApiKeyText), defaultBindingMode: Avalonia.Data.BindingMode.TwoWay);

    public static readonly StyledProperty<IEnumerable<ProviderModelItem>?> ProviderModelsProperty =
        AvaloniaProperty.Register<ModelProviderSettingsControl, IEnumerable<ProviderModelItem>?>(nameof(ProviderModels));

    public static readonly StyledProperty<IBrush?> EndpointStatusBrushProperty =
        AvaloniaProperty.Register<ModelProviderSettingsControl, IBrush?>(nameof(EndpointStatusBrush));

    public static readonly StyledProperty<bool> CanSaveEndpointProperty =
        AvaloniaProperty.Register<ModelProviderSettingsControl, bool>(nameof(CanSaveEndpoint));

    public static readonly StyledProperty<bool> CanResetEndpointProperty =
        AvaloniaProperty.Register<ModelProviderSettingsControl, bool>(nameof(CanResetEndpoint));

    public static readonly StyledProperty<bool> CanSaveCredentialProperty =
        AvaloniaProperty.Register<ModelProviderSettingsControl, bool>(nameof(CanSaveCredential));

    public static readonly StyledProperty<bool> CanRemoveCredentialProperty =
        AvaloniaProperty.Register<ModelProviderSettingsControl, bool>(nameof(CanRemoveCredential));

    public static readonly StyledProperty<bool> CredentialConfiguredProperty =
        AvaloniaProperty.Register<ModelProviderSettingsControl, bool>(nameof(CredentialConfigured));

    public static readonly StyledProperty<string> CredentialModeProperty =
        AvaloniaProperty.Register<ModelProviderSettingsControl, string>(nameof(CredentialMode), "keychain");

    public static readonly StyledProperty<string?> CredentialMaskProperty =
        AvaloniaProperty.Register<ModelProviderSettingsControl, string?>(nameof(CredentialMask));

    public event EventHandler<string>? ProviderSelected;
    public event EventHandler<RoutedEventArgs>? SaveEndpointRequested;
    public event EventHandler<RoutedEventArgs>? ResetEndpointRequested;
    public event EventHandler<RoutedEventArgs>? SaveCredentialRequested;
    public event EventHandler<RoutedEventArgs>? RemoveCredentialRequested;
    public event EventHandler<TextChangedEventArgs>? EndpointTextChanged;
    public event EventHandler<TextChangedEventArgs>? ApiKeyTextChanged;
    public event EventHandler<string>? CredentialModeChangedRequest;
    public event EventHandler<RoutedEventArgs>? CancelCredentialRequested;
    public event EventHandler<RoutedEventArgs>? RetryCredentialRequested;
    private bool _credentialBusy;
    public string SavedCredentialMode { get; set; } = "keychain";

    public void SetCredentialBusy(bool busy)
    {
        _credentialBusy = busy;
        SyncView();
    }

    public void SetCredentialFeedback(string message, bool isError = false)
    {
        CredentialFeedbackTextBlock.Text = message;
        CredentialFeedbackTextBlock.IsVisible = !string.IsNullOrEmpty(message);
        CredentialFeedbackTextBlock.Foreground = this.FindResource(isError ? "DangerBrush" : "TextSecondaryBrush") as IBrush;
    }

    private void CancelCredential(object? sender, RoutedEventArgs e) => CancelCredentialRequested?.Invoke(this, e);
    private void RetryCredential(object? sender, RoutedEventArgs e) => RetryCredentialRequested?.Invoke(this, e);

    private bool _syncingEndpoint;
    private bool _syncingApiKey;
    private bool _syncingCredentialMode;

    public ModelProviderSettingsControl()
    {
        InitializeComponent();
        SyncView();
    }

    public IEnumerable<ProviderItem>? Providers
    {
        get => GetValue(ProvidersProperty);
        set => SetValue(ProvidersProperty, value);
    }

    public string? SelectedProviderId
    {
        get => GetValue(SelectedProviderIdProperty);
        set => SetValue(SelectedProviderIdProperty, value);
    }

    public string? EndpointText
    {
        get => GetValue(EndpointTextProperty);
        set => SetValue(EndpointTextProperty, value);
    }

    public string? EndpointStatusText
    {
        get => GetValue(EndpointStatusTextProperty);
        set => SetValue(EndpointStatusTextProperty, value);
    }

    public string? CredentialStatusText
    {
        get => GetValue(CredentialStatusTextProperty);
        set => SetValue(CredentialStatusTextProperty, value);
    }

    public string CredentialStatus
    {
        get => GetValue(CredentialStatusProperty);
        set => SetValue(CredentialStatusProperty, value);
    }

    public string? ApiKeyPlaceholderText
    {
        get => GetValue(ApiKeyPlaceholderTextProperty);
        set => SetValue(ApiKeyPlaceholderTextProperty, value);
    }

    public string? ApiKeyText
    {
        get => GetValue(ApiKeyTextProperty);
        set => SetValue(ApiKeyTextProperty, value);
    }

    public IEnumerable<ProviderModelItem>? ProviderModels
    {
        get => GetValue(ProviderModelsProperty);
        set => SetValue(ProviderModelsProperty, value);
    }

    public IBrush? EndpointStatusBrush
    {
        get => GetValue(EndpointStatusBrushProperty);
        set => SetValue(EndpointStatusBrushProperty, value);
    }

    public bool CanSaveEndpoint
    {
        get => GetValue(CanSaveEndpointProperty);
        set => SetValue(CanSaveEndpointProperty, value);
    }

    public bool CanResetEndpoint
    {
        get => GetValue(CanResetEndpointProperty);
        set => SetValue(CanResetEndpointProperty, value);
    }

    public bool CanSaveCredential
    {
        get => GetValue(CanSaveCredentialProperty);
        set => SetValue(CanSaveCredentialProperty, value);
    }

    public bool CanRemoveCredential
    {
        get => GetValue(CanRemoveCredentialProperty);
        set => SetValue(CanRemoveCredentialProperty, value);
    }

    public bool CredentialConfigured
    {
        get => GetValue(CredentialConfiguredProperty);
        set => SetValue(CredentialConfiguredProperty, value);
    }

    public string CredentialMode
    {
        get => GetValue(CredentialModeProperty);
        set => SetValue(CredentialModeProperty, value);
    }

    public string? CredentialMask
    {
        get => GetValue(CredentialMaskProperty);
        set => SetValue(CredentialMaskProperty, value);
    }

    protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
    {
        base.OnPropertyChanged(change);
        if (change.Property == ProvidersProperty
            || change.Property == SelectedProviderIdProperty
            || change.Property == EndpointTextProperty
            || change.Property == EndpointStatusTextProperty
            || change.Property == CredentialStatusTextProperty
            || change.Property == CredentialStatusProperty
            || change.Property == ApiKeyPlaceholderTextProperty
            || change.Property == ApiKeyTextProperty
            || change.Property == ProviderModelsProperty
            || change.Property == EndpointStatusBrushProperty
            || change.Property == CanSaveEndpointProperty
            || change.Property == CanResetEndpointProperty
            || change.Property == CanSaveCredentialProperty
            || change.Property == CanRemoveCredentialProperty
            || change.Property == CredentialConfiguredProperty
            || change.Property == CredentialModeProperty
            || change.Property == CredentialMaskProperty)
        {
            SyncView();
        }
    }

    public void FocusCredentialInput() => ApiKeyInput.Focus();

    private void SyncView()
    {
        var selectedId = SelectedProviderId?.Trim();
        var hasSelection = !string.IsNullOrWhiteSpace(selectedId);
        OverviewPanel.IsVisible = !hasSelection;
        DetailPanel.IsVisible = hasSelection;
        ProviderItemsControl.ItemsSource = Providers;
        if (!hasSelection)
        {
            return;
        }

        var provider = Providers?.FirstOrDefault(item => item.Id == selectedId);
        ProviderTitleText.Text = provider?.DisplayName ?? selectedId;
        ProviderDescriptionText.Text = LocalizationService.GetString("LocProviderDescription", "Configure the provider URL and credential used by the local agent.");

        _syncingEndpoint = true;
        EndpointInput.Text = EndpointText ?? string.Empty;
        _syncingEndpoint = false;

        _syncingApiKey = true;
        ApiKeyInput.Text = ApiKeyText ?? string.Empty;
        _syncingApiKey = false;

        var hasSavedKey = !string.IsNullOrWhiteSpace(CredentialMask);
        ApiKeyInput.PlaceholderText = hasSavedKey && CredentialStatus != "missing"
            ? LocalizationService.GetString("LocCredentialKeepSavedKey", "Leave empty to keep the saved key")
            : ApiKeyPlaceholderText ?? string.Format(
                System.Globalization.CultureInfo.CurrentCulture,
                LocalizationService.GetString("LocPasteApiKey", "Paste {0} API key"),
                provider?.DisplayName ?? selectedId);
        EndpointStatusTextBlock.Text = EndpointStatusText ?? string.Empty;
        EndpointStatusTextBlock.Foreground = EndpointStatusBrush ?? this.FindResource("TextSecondaryBrush") as IBrush;
        var models = ProviderModels?.Where(model => !string.IsNullOrWhiteSpace(model.Display)).ToArray() ?? [];
        ProviderModelsItemsControl.ItemsSource = models;
        ProviderModelsItemsControl.IsVisible = models.Length > 0;
        NoProviderModelsText.IsVisible = models.Length == 0;
        CredentialStatusTextBlock.Text = CredentialConfigured
            ? LocalizationService.GetString("LocCredentialSavedStatus", "API key saved")
            : CredentialStatus switch
            {
                "locked" => LocalizationService.GetString("LocCredentialLocked", "Unlock your system keychain, then try again."),
                "access_denied" => LocalizationService.GetString("LocCredentialDenied", "Allow SunCode to access your system keychain, then try again."),
                "unavailable" => LocalizationService.GetString("LocCredentialUnavailable", "System keychain unavailable. Check your credential service, then try again."),
                _ => hasSavedKey
                    ? LocalizationService.GetString("LocCredentialMissing", "Saved key not found. Enter a replacement API key.")
                    : LocalizationService.GetString("LocCredentialEmptyStatus", "No API key saved"),
            };
        _syncingCredentialMode = true;
        CredentialModeKeychainInput.IsChecked = CredentialMode != "plaintext";
        CredentialModePlaintextInput.IsChecked = CredentialMode == "plaintext";
        _syncingCredentialMode = false;
        var storeName = OperatingSystem.IsMacOS() ? "macOS Keychain"
            : OperatingSystem.IsWindows() ? "Windows Credential Manager" : "Linux Secret Service";
        CredentialKeychainDescriptionTextBlock.Text = string.Format(
            LocalizationService.GetString("LocCredentialKeychainDescription", "Protected by {0}. The key stays outside the database."),
            storeName);
        CredentialInputLabelTextBlock.Text = LocalizationService.GetString(
            hasSavedKey ? "LocCredentialReplaceLabel" : "LocCredentialInputLabel",
            hasSavedKey ? "Replace API key" : "API key");
        CredentialHelpTextBlock.Text = LocalizationService.GetString(
            "LocCredentialHelp",
            "Storage applies only to this provider. Saved keys are shown masked. Your operating system may ask you to allow access.");
        CredentialPlaintextWarning.IsVisible = CredentialMode == "plaintext";
        CredentialLocationTextBlock.Text = !hasSavedKey
            ? LocalizationService.GetString("LocCredentialEmptyHint", "Add a key to use this provider. System keychain is the default.")
            : string.Format(LocalizationService.GetString("LocCredentialSavedIn", "Saved in {0}"),
                SavedCredentialMode == "plaintext" ? LocalizationService.GetString("LocCredentialPlaintextLocation", "the local database · Plaintext") : storeName);
        CredentialMaskTextBlock.Text = CredentialMask ?? string.Empty;
        CredentialMaskBorder.IsVisible = !string.IsNullOrWhiteSpace(CredentialMask);
        CredentialWarningDot.IsVisible = !CredentialConfigured;
        CredentialSuccessDot.IsVisible = CredentialConfigured;
        SaveCredentialButton.Content = _credentialBusy
            ? LocalizationService.GetString("LocCredentialSaving", "Saving API key…")
            : hasSavedKey
            ? LocalizationService.GetString("LocSaveChanges", "Save changes")
            : LocalizationService.GetString("LocSaveKey", "Save key");
        SaveEndpointButton.IsEnabled = CanSaveEndpoint;
        ResetEndpointButton.IsEnabled = CanResetEndpoint;
        SaveCredentialButton.IsEnabled = CanSaveCredential && !_credentialBusy;
        CancelCredentialButton.IsVisible = !string.IsNullOrEmpty(ApiKeyText) || CredentialMode != SavedCredentialMode;
        CredentialFields.IsEnabled = !_credentialBusy;
        CancelCredentialButton.IsEnabled = !_credentialBusy;
        RetryCredentialButton.IsVisible = !CredentialConfigured && !string.IsNullOrEmpty(CredentialMask) && CredentialStatus != "missing";
        RetryCredentialButton.IsEnabled = !_credentialBusy;
        RemoveCredentialButton.IsEnabled = CanRemoveCredential && !_credentialBusy;
    }

    private void SelectProvider(object? sender, RoutedEventArgs e)
    {
        if (sender is not Button { Tag: string providerId }) return;
        ProviderSelected?.Invoke(this, providerId);
    }

    private void EndpointInputChanged(object? sender, TextChangedEventArgs e)
    {
        if (!_syncingEndpoint)
        {
            EndpointText = EndpointInput.Text;
        }

        EndpointTextChanged?.Invoke(this, e);
    }

    private void ApiKeyInputChanged(object? sender, TextChangedEventArgs e)
    {
        if (!_syncingApiKey)
        {
            ApiKeyText = ApiKeyInput.Text;
        }

        ApiKeyTextChanged?.Invoke(this, e);
    }

    private void CredentialModeChanged(object? sender, RoutedEventArgs e)
    {
        if (_syncingCredentialMode) return;
        if (sender is not RadioButton radio || radio.Tag is not string mode || radio.IsChecked != true) return;
        if (mode == CredentialMode) return;
        CredentialMode = mode;
        CredentialModeChangedRequest?.Invoke(this, mode);
    }

    private void SaveEndpoint(object? sender, RoutedEventArgs e) =>
        SaveEndpointRequested?.Invoke(this, e);

    private void ResetEndpoint(object? sender, RoutedEventArgs e) =>
        ResetEndpointRequested?.Invoke(this, e);

    private void SaveCredential(object? sender, RoutedEventArgs e) =>
        SaveCredentialRequested?.Invoke(this, e);

    private void RemoveCredential(object? sender, RoutedEventArgs e) =>
        RemoveCredentialRequested?.Invoke(this, e);
}
