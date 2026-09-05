using Avalonia.Controls;
using Avalonia.Interactivity;
using SunCode.Desktop.Controls;

namespace SunCode.Desktop.Views.Settings.Controls;

public sealed partial class NetworkSettingsControl : UserControl
{
    public event EventHandler<RoutedEventArgs>? HttpsCertificateVerificationChanged;
    public event EventHandler<RoutedEventArgs>? SystemCertificatesChanged;
    public event EventHandler<RoutedEventArgs>? SaveHttpsCertificateVerificationRequested;
    public ToggleSwitch VerifyHttpsCertificatesToggleControl => VerifyHttpsCertificatesToggle;
    public ToggleSwitch UseSystemCertificatesToggleControl => UseSystemCertificatesToggle;
    public SCFileSelector CertificatePathInputControl => CertificatePathInput;
    public Border CertificateTrustSectionControl => CertificateTrustSection;
    public Border HttpsCertificateWarningControl => HttpsCertificateWarning;
    public TextBlock CertificatePathHintText => CertificatePathHint;
    public TextBlock HttpsCertificateStatusText => HttpsCertificateStatus;
    public NetworkSettingsControl() => InitializeComponent();
    private void OnHttpsCertificateVerificationChanged(object? sender, RoutedEventArgs e) => HttpsCertificateVerificationChanged?.Invoke(this, e);
    private void OnSystemCertificatesChanged(object? sender, RoutedEventArgs e) => SystemCertificatesChanged?.Invoke(this, e);
    private void OnSaveHttpsCertificateVerification(object? sender, RoutedEventArgs e) => SaveHttpsCertificateVerificationRequested?.Invoke(this, e);
}
