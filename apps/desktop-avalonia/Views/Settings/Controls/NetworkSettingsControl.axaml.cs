using Avalonia.Controls;
using Avalonia.Interactivity;
using SunCode.Desktop.Controls;

namespace SunCode.Desktop.Views.Settings.Controls;

public sealed partial class NetworkSettingsControl : UserControl
{
    public event EventHandler<SelectionChangedEventArgs>? ProxyModeChanged;
    public event EventHandler<TextChangedEventArgs>? ProxyTextChanged;
    public event EventHandler<RoutedEventArgs>? RemoveProxyPasswordRequested;
    public event EventHandler<RoutedEventArgs>? SaveProxyRequested;
    public event EventHandler<RoutedEventArgs>? HttpsCertificateVerificationChanged;
    public event EventHandler<RoutedEventArgs>? SystemCertificatesChanged;
    public event EventHandler<TextChangedEventArgs>? CertificatePathChanged;
    public event EventHandler<RoutedEventArgs>? SaveHttpsCertificateVerificationRequested;
    public ToggleSwitch VerifyHttpsCertificatesToggleControl => VerifyHttpsCertificatesToggle;
    public ToggleSwitch UseSystemCertificatesToggleControl => UseSystemCertificatesToggle;
    public SCFileSelector CertificatePathInputControl => CertificatePathInput;
    public Border CertificateTrustSectionControl => CertificateTrustSection;
    public Border HttpsCertificateWarningControl => HttpsCertificateWarning;
    public TextBlock CertificatePathHintText => CertificatePathHint;
    public TextBlock HttpsCertificateStatusText => HttpsCertificateStatus;
    public Button SaveHttpsCertificateButtonControl => SaveHttpsCertificateButton;
    public SCFlatComboBox ProxyModeSelectorControl => ProxyModeSelector;
    public TextBox ProxyUrlInputControl => ProxyUrlInput;
    public TextBox ProxyUsernameInputControl => ProxyUsernameInput;
    public TextBox ProxyPasswordInputControl => ProxyPasswordInput;
    public TextBox ProxyBypassInputControl => ProxyBypassInput;
    public TextBlock ProxyPasswordHintText => ProxyPasswordHint;
    public Button RemoveProxyPasswordButtonControl => RemoveProxyPasswordButton;
    public Button SaveProxyButtonControl => SaveProxyButton;
    public TextBlock ProxyStatusText => ProxyStatus;
    public Border CustomProxySectionControl => CustomProxySection;

    public NetworkSettingsControl()
    {
        InitializeComponent();
        CertificatePathInput.TextChanged += (_, e) => CertificatePathChanged?.Invoke(this, e);
    }
    private void OnHttpsCertificateVerificationChanged(object? sender, RoutedEventArgs e) => HttpsCertificateVerificationChanged?.Invoke(this, e);
    private void OnSystemCertificatesChanged(object? sender, RoutedEventArgs e) => SystemCertificatesChanged?.Invoke(this, e);
    private void OnSaveHttpsCertificateVerification(object? sender, RoutedEventArgs e) => SaveHttpsCertificateVerificationRequested?.Invoke(this, e);
    private void OnProxyModeChanged(object? sender, SelectionChangedEventArgs e) => ProxyModeChanged?.Invoke(this, e);
    private void OnProxyTextChanged(object? sender, TextChangedEventArgs e) => ProxyTextChanged?.Invoke(this, e);
    private void OnRemoveProxyPassword(object? sender, RoutedEventArgs e) => RemoveProxyPasswordRequested?.Invoke(this, e);
    private void OnSaveProxy(object? sender, RoutedEventArgs e) => SaveProxyRequested?.Invoke(this, e);
}
