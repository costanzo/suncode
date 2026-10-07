using System.Windows.Input;
using SunCode.Desktop.Controls;
using SunCode.Desktop.Infrastructure;
using static SunCode.Desktop.ViewModels.SettingsFormatting;

namespace SunCode.Desktop.ViewModels;

// Edit state of the Network settings page. The proxy and HTTPS sections each
// compare their editors with the last saved NetworkSettingsViewModel values,
// so each Save command is enabled only while its section has changes.
public sealed class NetworkSettingsEditorViewModel : ObservableObject
{
    private readonly IViewModelHost _host;
    private readonly AsyncRelayCommand _saveProxy;
    private readonly AsyncRelayCommand _saveHttps;
    private readonly RelayCommand _removeProxyPassword;
    private IReadOnlyList<SCComboBoxItem> _proxyModeOptions = [];
    private SCComboBoxItem? _selectedProxyMode;
    private string? _proxyUrl;
    private string? _proxyUsername;
    private string? _proxyPassword;
    private string? _proxyBypass;
    private bool _clearProxyPassword;
    private string _proxyStatusText = string.Empty;
    private StatusTone _proxyStatusTone;
    private bool _verifyHttpsCertificates = true;
    private bool _useSystemCertificates = true;
    private string? _certificatePath;
    private string _httpsStatusText = string.Empty;
    private StatusTone _httpsStatusTone;
    private bool _baselineVerifyHttps = true;
    private bool _baselineUseSystem = true;
    private string _baselineCertificatePath = string.Empty;
    private string _baselineProxyMode = "system";
    private string _baselineProxyUrl = string.Empty;
    private string _baselineProxyUsername = string.Empty;
    private string _baselineProxyBypass = string.Empty;
    private bool _baselinePasswordConfigured;

    internal NetworkSettingsEditorViewModel(IViewModelHost host, NetworkSettingsViewModel saved)
    {
        _host = host;
        Saved = saved;
        _saveProxy = new AsyncRelayCommand(SaveProxyAsync, () => IsProxyDirty);
        _saveHttps = new AsyncRelayCommand(SaveHttpsAsync, () => IsHttpsDirty);
        _removeProxyPassword = new RelayCommand(RemoveProxyPassword);
    }

    public NetworkSettingsViewModel Saved { get; }

    public ICommand SaveProxyCommand => _saveProxy;
    public ICommand SaveHttpsCommand => _saveHttps;
    public ICommand RemoveProxyPasswordCommand => _removeProxyPassword;

    public static IReadOnlyList<SCComboBoxItem> CreateProxyModeOptions() =>
    [
        new(L("LocNoProxyMode", "No proxy"), "no_proxy"),
        new(L("LocSystemProxyMode", "System proxy"), "system"),
        new(L("LocCustomProxyMode", "Custom proxy"), "custom")
    ];

    public IReadOnlyList<SCComboBoxItem> ProxyModeOptions { get => _proxyModeOptions; private set => SetProperty(ref _proxyModeOptions, value); }

    public SCComboBoxItem? SelectedProxyMode
    {
        get => _selectedProxyMode;
        set
        {
            if (!SetProperty(ref _selectedProxyMode, value)) return;
            // A mode change replaces the save result text with the mode
            // summary; the last save color stays, as before the migration.
            ProxyStatusText = ProxyModeSummary;
            OnPropertyChanged(nameof(IsCustomProxy));
            RefreshProxyDirty();
        }
    }

    public string? ProxyUrl { get => _proxyUrl; set { if (SetProperty(ref _proxyUrl, value)) RefreshProxyDirty(); } }
    public string? ProxyUsername { get => _proxyUsername; set { if (SetProperty(ref _proxyUsername, value)) RefreshProxyDirty(); } }
    public string? ProxyBypass { get => _proxyBypass; set { if (SetProperty(ref _proxyBypass, value)) RefreshProxyDirty(); } }

    // Write-only: typing a new password cancels a pending removal.
    public string? ProxyPassword
    {
        get => _proxyPassword;
        set
        {
            if (!SetProperty(ref _proxyPassword, value)) return;
            if (!string.IsNullOrEmpty(value)) ClearProxyPassword = false;
            RefreshProxyDirty();
        }
    }

    public bool ClearProxyPassword
    {
        get => _clearProxyPassword;
        private set
        {
            if (!SetProperty(ref _clearProxyPassword, value)) return;
            OnPropertyChanged(nameof(IsProxyPasswordStored));
            OnPropertyChanged(nameof(ProxyPasswordPlaceholder));
            OnPropertyChanged(nameof(ProxyPasswordHint));
        }
    }

    private string CurrentProxyMode => SelectedProxyMode?.Value as string ?? "system";
    public bool IsCustomProxy => CurrentProxyMode == "custom";
    public bool IsProxyPasswordStored => _baselinePasswordConfigured && !ClearProxyPassword;
    public string ProxyPasswordPlaceholder => IsProxyPasswordStored
        ? L("LocPasswordStored", "Password stored")
        : L("LocOptional", "Optional");
    public string ProxyPasswordHint => IsProxyPasswordStored
        ? L("LocPasswordStoredHint", "Password stored. Leave empty to keep it or remove it explicitly.")
        : L("LocProxyPasswordHint", "Optional Basic proxy authentication password.");

    public string ProxyStatusText { get => _proxyStatusText; private set => SetProperty(ref _proxyStatusText, value); }
    public StatusTone ProxyStatusTone { get => _proxyStatusTone; private set => SetProperty(ref _proxyStatusTone, value); }

    private string ProxyModeSummary => CurrentProxyMode switch
    {
        "no_proxy" => L("LocDirectConnections", "Direct connections"),
        "custom" => L("LocCustomProxyStatus", "Custom proxy"),
        _ => L("LocSystemProxyStatus", "System proxy")
    };

    public bool IsProxyDirty =>
        !string.Equals(CurrentProxyMode, _baselineProxyMode, StringComparison.Ordinal)
        || !string.Equals(ProxyUrl?.Trim() ?? string.Empty, _baselineProxyUrl, StringComparison.Ordinal)
        || !string.Equals(ProxyUsername?.Trim() ?? string.Empty, _baselineProxyUsername, StringComparison.Ordinal)
        || !string.Equals(NormalizeProxyBypass(ProxyBypass), _baselineProxyBypass, StringComparison.Ordinal)
        || !string.IsNullOrEmpty(ProxyPassword)
        || ClearProxyPassword;

    public bool VerifyHttpsCertificates
    {
        get => _verifyHttpsCertificates;
        set
        {
            if (!SetProperty(ref _verifyHttpsCertificates, value)) return;
            OnPropertyChanged(nameof(IsCertificateVerificationOff));
            HttpsStatusText = HttpsSummary;
            RefreshHttpsDirty();
        }
    }

    public bool UseSystemCertificates
    {
        get => _useSystemCertificates;
        set
        {
            if (!SetProperty(ref _useSystemCertificates, value)) return;
            // Kept in sync for compatibility with existing readers of the saved model.
            Saved.UseSystemCertificates = value;
            OnPropertyChanged(nameof(CertificatePathHint));
            HttpsStatusText = HttpsSummary;
            RefreshHttpsDirty();
        }
    }

    public string? CertificatePath { get => _certificatePath; set { if (SetProperty(ref _certificatePath, value)) RefreshHttpsDirty(); } }

    public bool IsCertificateVerificationOff => !VerifyHttpsCertificates;
    public string CertificatePathHint => UseSystemCertificates
        ? L("LocCertificatePathHint", "Disable system certificates to provide a custom certificate file.")
        : L("LocCustomCertificatePathHint", "Choose a PEM, CRT, CER, or DER certificate file for custom trust.");

    public string HttpsStatusText { get => _httpsStatusText; private set => SetProperty(ref _httpsStatusText, value); }
    public StatusTone HttpsStatusTone { get => _httpsStatusTone; private set => SetProperty(ref _httpsStatusTone, value); }

    private string HttpsSummary => VerifyHttpsCertificates
        ? UseSystemCertificates
            ? L("LocSystemTrustStore", "System trust store")
            : L("LocCustomCertificateRequired", "Custom certificate required")
        : L("LocReviewRequired", "Review required");

    public bool IsHttpsDirty =>
        VerifyHttpsCertificates != _baselineVerifyHttps
        || UseSystemCertificates != _baselineUseSystem
        || !string.Equals(CertificatePath?.Trim() ?? string.Empty, _baselineCertificatePath, StringComparison.Ordinal);

    // Loads saved values into the editors and makes them the baseline.
    public void Load()
    {
        ProxyModeOptions = CreateProxyModeOptions();
        _selectedProxyMode = ProxyModeOptions.FirstOrDefault(item => Equals(item.Value, Saved.ProxyMode));
        _proxyUrl = Saved.ProxyUrl;
        _proxyUsername = Saved.ProxyUsername;
        _proxyPassword = string.Empty;
        _proxyBypass = Saved.ProxyBypassRules;
        _clearProxyPassword = false;
        _verifyHttpsCertificates = Saved.VerifyHttpsCertificates;
        _useSystemCertificates = Saved.UseSystemCertificates;
        _certificatePath = Saved.CertificatePath;
        ResetProxyBaseline();
        ResetHttpsBaseline();
        ProxyStatusText = ProxyModeSummary;
        ProxyStatusTone = StatusTone.Neutral;
        HttpsStatusText = HttpsSummary;
        HttpsStatusTone = StatusTone.Neutral;
        foreach (var name in new[]
                 {
                     nameof(SelectedProxyMode), nameof(ProxyUrl), nameof(ProxyUsername), nameof(ProxyPassword),
                     nameof(ProxyBypass), nameof(ClearProxyPassword), nameof(IsProxyPasswordStored), nameof(IsCustomProxy), nameof(VerifyHttpsCertificates),
                     nameof(UseSystemCertificates), nameof(CertificatePath), nameof(IsCertificateVerificationOff)
                 })
            OnPropertyChanged(name);
        RefreshLocalizedText();
        RefreshProxyDirty();
        RefreshHttpsDirty();
    }

    // Rebuilds localized option labels and hints after a language switch,
    // keeping the selected proxy mode and any save result text.
    public void RefreshLocalizedText()
    {
        var mode = CurrentProxyMode;
        ProxyModeOptions = CreateProxyModeOptions();
        _selectedProxyMode = ProxyModeOptions.FirstOrDefault(item => Equals(item.Value, mode));
        OnPropertyChanged(nameof(SelectedProxyMode));
        OnPropertyChanged(nameof(ProxyPasswordPlaceholder));
        OnPropertyChanged(nameof(ProxyPasswordHint));
        OnPropertyChanged(nameof(CertificatePathHint));
        HttpsStatusText = HttpsSummary;
        if (string.IsNullOrWhiteSpace(ProxyStatusText)) ProxyStatusText = ProxyModeSummary;
    }

    private void RemoveProxyPassword()
    {
        ClearProxyPassword = true;
        _proxyPassword = string.Empty;
        OnPropertyChanged(nameof(ProxyPassword));
        RefreshProxyDirty();
    }

    private async Task SaveProxyAsync()
    {
        var saved = await Saved.SaveProxyConfigurationAsync(
            CurrentProxyMode, ProxyUrl, ProxyUsername, ProxyPassword, ClearProxyPassword, ProxyBypass);
        ProxyStatusText = _host.StatusText;
        ProxyStatusTone = SaveResultTone(saved);
        if (saved)
        {
            ResetProxyBaseline();
            ClearProxyPassword = false;
            _proxyPassword = string.Empty;
            _proxyBypass = Saved.ProxyBypassRules;
            OnPropertyChanged(nameof(ProxyPassword));
            OnPropertyChanged(nameof(ProxyBypass));
        }
        OnPropertyChanged(nameof(IsProxyPasswordStored));
        OnPropertyChanged(nameof(ProxyPasswordPlaceholder));
        OnPropertyChanged(nameof(ProxyPasswordHint));
        RefreshProxyDirty();
    }

    private async Task SaveHttpsAsync()
    {
        var saved = await Saved.SaveHttpsCertificateVerificationAsync(VerifyHttpsCertificates);
        saved = await Saved.SaveCertificateTrustAsync(UseSystemCertificates, CertificatePath) && saved;
        HttpsStatusText = _host.StatusText;
        HttpsStatusTone = SaveResultTone(saved);
        if (saved)
        {
            ResetHttpsBaseline();
        }
        else
        {
            // Restore the persisted verification state and keep the failure text.
            _verifyHttpsCertificates = Saved.VerifyHttpsCertificates;
            OnPropertyChanged(nameof(VerifyHttpsCertificates));
            OnPropertyChanged(nameof(IsCertificateVerificationOff));
        }
        RefreshHttpsDirty();
    }

    private void ResetProxyBaseline()
    {
        _baselineProxyMode = Saved.ProxyMode;
        _baselineProxyUrl = Saved.ProxyUrl;
        _baselineProxyUsername = Saved.ProxyUsername;
        _baselineProxyBypass = NormalizeProxyBypass(Saved.ProxyBypassRules);
        _baselinePasswordConfigured = Saved.ProxyPasswordConfigured;
    }

    private void ResetHttpsBaseline()
    {
        _baselineVerifyHttps = Saved.VerifyHttpsCertificates;
        _baselineUseSystem = Saved.UseSystemCertificates;
        _baselineCertificatePath = Saved.CertificatePath ?? string.Empty;
    }

    private void RefreshProxyDirty()
    {
        OnPropertyChanged(nameof(IsProxyDirty));
        _saveProxy.RaiseCanExecuteChanged();
    }

    private void RefreshHttpsDirty()
    {
        OnPropertyChanged(nameof(IsHttpsDirty));
        _saveHttps.RaiseCanExecuteChanged();
    }
}
