using Avalonia;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Media;
using SunCode.Desktop.Models;

namespace SunCode.Desktop.Controls;

public sealed partial class SCModelProvider : UserControl
{
    public static readonly StyledProperty<IEnumerable<ProviderItem>?> ProvidersProperty =
        AvaloniaProperty.Register<SCModelProvider, IEnumerable<ProviderItem>?>(nameof(Providers));

    public static readonly StyledProperty<string?> SelectedProviderIdProperty =
        AvaloniaProperty.Register<SCModelProvider, string?>(nameof(SelectedProviderId));

    public static readonly StyledProperty<string?> EndpointTextProperty =
        AvaloniaProperty.Register<SCModelProvider, string?>(nameof(EndpointText), defaultBindingMode: Avalonia.Data.BindingMode.TwoWay);

    public static readonly StyledProperty<string?> EndpointStatusTextProperty =
        AvaloniaProperty.Register<SCModelProvider, string?>(nameof(EndpointStatusText));

    public static readonly StyledProperty<string?> CredentialStatusTextProperty =
        AvaloniaProperty.Register<SCModelProvider, string?>(nameof(CredentialStatusText));

    public static readonly StyledProperty<string?> ApiKeyPlaceholderTextProperty =
        AvaloniaProperty.Register<SCModelProvider, string?>(nameof(ApiKeyPlaceholderText));

    public static readonly StyledProperty<string?> ApiKeyTextProperty =
        AvaloniaProperty.Register<SCModelProvider, string?>(nameof(ApiKeyText), defaultBindingMode: Avalonia.Data.BindingMode.TwoWay);

    public static readonly StyledProperty<IEnumerable<ProviderModelItem>?> ProviderModelsProperty =
        AvaloniaProperty.Register<SCModelProvider, IEnumerable<ProviderModelItem>?>(nameof(ProviderModels));

    public static readonly StyledProperty<IBrush?> EndpointStatusBrushProperty =
        AvaloniaProperty.Register<SCModelProvider, IBrush?>(nameof(EndpointStatusBrush));

    public static readonly StyledProperty<bool> CanSaveEndpointProperty =
        AvaloniaProperty.Register<SCModelProvider, bool>(nameof(CanSaveEndpoint));

    public static readonly StyledProperty<bool> CanResetEndpointProperty =
        AvaloniaProperty.Register<SCModelProvider, bool>(nameof(CanResetEndpoint));

    public static readonly StyledProperty<bool> CanSaveCredentialProperty =
        AvaloniaProperty.Register<SCModelProvider, bool>(nameof(CanSaveCredential));

    public static readonly StyledProperty<bool> CanRemoveCredentialProperty =
        AvaloniaProperty.Register<SCModelProvider, bool>(nameof(CanRemoveCredential));

    public static readonly StyledProperty<bool> CredentialConfiguredProperty =
        AvaloniaProperty.Register<SCModelProvider, bool>(nameof(CredentialConfigured));

    public event EventHandler<string>? ProviderSelected;
    public event EventHandler<RoutedEventArgs>? SaveEndpointRequested;
    public event EventHandler<RoutedEventArgs>? ResetEndpointRequested;
    public event EventHandler<RoutedEventArgs>? SaveCredentialRequested;
    public event EventHandler<RoutedEventArgs>? RemoveCredentialRequested;
    public event EventHandler<TextChangedEventArgs>? EndpointTextChanged;
    public event EventHandler<TextChangedEventArgs>? ApiKeyTextChanged;

    private bool _syncingEndpoint;
    private bool _syncingApiKey;

    public SCModelProvider()
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

    protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
    {
        base.OnPropertyChanged(change);
        if (change.Property == ProvidersProperty
            || change.Property == SelectedProviderIdProperty
            || change.Property == EndpointTextProperty
            || change.Property == EndpointStatusTextProperty
            || change.Property == CredentialStatusTextProperty
            || change.Property == ApiKeyPlaceholderTextProperty
            || change.Property == ApiKeyTextProperty
            || change.Property == ProviderModelsProperty
            || change.Property == EndpointStatusBrushProperty
            || change.Property == CanSaveEndpointProperty
            || change.Property == CanResetEndpointProperty
            || change.Property == CanSaveCredentialProperty
            || change.Property == CanRemoveCredentialProperty
            || change.Property == CredentialConfiguredProperty)
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

        var metadata = SCProviderCatalog.GetOrDefault(selectedId!);
        ProviderTitleText.Text = metadata.Title;
        ProviderDescriptionText.Text = metadata.Description;

        _syncingEndpoint = true;
        EndpointInput.Text = EndpointText ?? string.Empty;
        _syncingEndpoint = false;

        _syncingApiKey = true;
        ApiKeyInput.Text = ApiKeyText ?? string.Empty;
        _syncingApiKey = false;

        ApiKeyInput.PlaceholderText = ApiKeyPlaceholderText ?? metadata.ApiKeyPlaceholder;
        EndpointStatusTextBlock.Text = EndpointStatusText ?? string.Empty;
        EndpointStatusTextBlock.Foreground = EndpointStatusBrush ?? this.FindResource("TextSecondaryBrush") as IBrush;
        var models = ProviderModels?.Where(model => !string.IsNullOrWhiteSpace(model.Display)).ToArray() ?? [];
        ProviderModelsItemsControl.ItemsSource = models;
        ProviderModelsItemsControl.IsVisible = models.Length > 0;
        NoProviderModelsText.IsVisible = models.Length == 0;
        CredentialStatusTextBlock.Text = CredentialStatusText ?? string.Empty;
        CredentialStatusTextBlock.Foreground = this.FindResource("TextBrush") as IBrush;
        CredentialWarningDot.IsVisible = !CredentialConfigured;
        CredentialSuccessDot.IsVisible = CredentialConfigured;
        CredentialStoredBorder.IsVisible = CredentialConfigured;
        CredentialMissingHint.IsVisible = !CredentialConfigured;
        var isZhipu = string.Equals(selectedId, "zhipu", StringComparison.Ordinal);
        ProviderUnconfiguredBanner.IsVisible = isZhipu && !CredentialConfigured;
        ProviderUnconfiguredTitle.Text = isZhipu
            ? "Try Zhipu GLM after adding a key"
            : "Add an API key to use this provider";
        SaveCredentialButton.Content = CredentialConfigured ? "Replace key" : "Save key";
        SaveEndpointButton.IsEnabled = CanSaveEndpoint;
        ResetEndpointButton.IsEnabled = CanResetEndpoint;
        SaveCredentialButton.IsEnabled = CanSaveCredential;
        RemoveCredentialButton.IsEnabled = CanRemoveCredential;
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

    private void SaveEndpoint(object? sender, RoutedEventArgs e) =>
        SaveEndpointRequested?.Invoke(this, e);

    private void ResetEndpoint(object? sender, RoutedEventArgs e) =>
        ResetEndpointRequested?.Invoke(this, e);

    private void SaveCredential(object? sender, RoutedEventArgs e) =>
        SaveCredentialRequested?.Invoke(this, e);

    private void RemoveCredential(object? sender, RoutedEventArgs e) =>
        RemoveCredentialRequested?.Invoke(this, e);
}
