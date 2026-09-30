using System.Text.Json;
using SunCode.Desktop.Infrastructure;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

public sealed class NetworkSettingsViewModel : ObservableObject
{
    private readonly IViewModelHost _host;
    private bool _verifyHttpsCertificates = true;
    private bool _useSystemCertificates = true;
    private string _certificatePath = string.Empty;
    private string _proxyMode = "system";
    private string _proxyUrl = string.Empty;
    private string _proxyUsername = string.Empty;
    private bool _proxyPasswordConfigured;
    private string _proxyBypassRules = string.Empty;

    internal NetworkSettingsViewModel(IViewModelHost host) => _host = host;

    public bool VerifyHttpsCertificates { get => _verifyHttpsCertificates; private set => SetProperty(ref _verifyHttpsCertificates, value); }
    public bool UseSystemCertificates { get => _useSystemCertificates; set => SetProperty(ref _useSystemCertificates, value); }
    public string CertificatePath { get => _certificatePath; set => SetProperty(ref _certificatePath, value); }
    public string ProxyMode { get => _proxyMode; private set => SetProperty(ref _proxyMode, value); }
    public string ProxyUrl { get => _proxyUrl; private set => SetProperty(ref _proxyUrl, value); }
    public string ProxyUsername { get => _proxyUsername; private set => SetProperty(ref _proxyUsername, value); }
    public bool ProxyPasswordConfigured { get => _proxyPasswordConfigured; private set => SetProperty(ref _proxyPasswordConfigured, value); }
    public string ProxyBypassRules { get => _proxyBypassRules; private set => SetProperty(ref _proxyBypassRules, value); }

    internal void Apply(SettingsSnapshot settings)
    {
        VerifyHttpsCertificates = settings.Bool("verify_https_certificates", true);
        UseSystemCertificates = settings.Bool("use_system_certificates", true);
        CertificatePath = settings.String("certificate_path", string.Empty);
        var proxyMode = settings.String("proxy_mode", "system");
        ProxyMode = proxyMode is "no_proxy" or "system" or "custom" ? proxyMode : "system";
        ProxyUrl = settings.String("proxy_url", string.Empty);
        ProxyUsername = settings.String("proxy_username", string.Empty);
        ProxyPasswordConfigured = settings.Bool("proxy_password_configured", false);
        ProxyBypassRules = string.Join(Environment.NewLine, settings.StringArray("proxy_bypass"));
    }

    public async Task<bool> SaveHttpsCertificateVerificationAsync(bool enabled)
    {
        if (!_host.EnsureSdk()) return false;

        _host.SetBusy(true);
        try
        {
            await _host.Sdk!.SetSettingAsync(new SetSettingRequest("global", null, null, "verify_https_certificates", JsonSerializer.SerializeToElement(enabled)));
            VerifyHttpsCertificates = enabled;
            _host.ReportSuccess(enabled
                ? "HTTPS certificate verification enabled"
                : "HTTPS certificate verification disabled");
            return true;
        }
        catch (Exception exception)
        {
            _host.ReportError(exception);
            return false;
        }
        finally
        {
            _host.SetBusy(false);
        }
    }

    public async Task<bool> SaveCertificateTrustAsync(bool useSystem, string? certificatePath)
    {
        if (!_host.EnsureSdk()) return false;
        _host.SetBusy(true);
        try
        {
            certificatePath = certificatePath?.Trim() ?? string.Empty;
            await _host.Sdk!.SetSettingAsync(new SetSettingRequest("global", null, null, "use_system_certificates", JsonSerializer.SerializeToElement(useSystem)));
            await _host.Sdk.SetSettingAsync(new SetSettingRequest("global", null, null, "certificate_path", JsonSerializer.SerializeToElement(certificatePath)));
            UseSystemCertificates = useSystem;
            CertificatePath = certificatePath;
            _host.ReportSuccess("Certificate trust settings saved");
            return true;
        }
        catch (Exception exception)
        {
            _host.ReportError(exception);
            return false;
        }
        finally { _host.SetBusy(false); }
    }

    public async Task<bool> SaveProxyConfigurationAsync(
        string mode,
        string? url,
        string? username,
        string? password,
        bool clearPassword,
        string? bypassRules)
    {
        if (!_host.EnsureSdk()) return false;
        _host.SetBusy(true);
        try
        {
            var bypass = (bypassRules ?? string.Empty)
                .Split(['\r', '\n'], StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries)
                .Where(value => value.Length > 0)
                .ToArray();
            var result = await _host.Sdk!.SetProxyConfigurationAsync(new ProxyConfigurationRequest(
                mode,
                url?.Trim() ?? string.Empty,
                username?.Trim() ?? string.Empty,
                string.IsNullOrEmpty(password) ? null : password,
                clearPassword,
                bypass));
            ProxyMode = result.Mode;
            ProxyUrl = result.Url;
            ProxyUsername = result.Username;
            ProxyPasswordConfigured = result.PasswordConfigured;
            ProxyBypassRules = string.Join(Environment.NewLine, result.Bypass);
            _host.ReportSuccess("Proxy settings applied");
            return true;
        }
        catch (Exception exception)
        {
            _host.ReportError(exception);
            return false;
        }
        finally
        {
            _host.SetBusy(false);
        }
    }
}
