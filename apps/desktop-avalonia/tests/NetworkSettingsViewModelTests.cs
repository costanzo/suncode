using System.Text.Json;
using SunCode.Desktop.ViewModels;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Tests;

public sealed class NetworkSettingsViewModelTests
{
    [Fact]
    public void ApplyReadsPersistedNetworkSettings()
    {
        var viewModel = new NetworkSettingsViewModel(new FakeViewModelHost());

        viewModel.Apply(Snapshot(
            ("verify_https_certificates", false),
            ("use_system_certificates", false),
            ("certificate_path", "/tmp/ca.pem"),
            ("proxy_mode", "custom"),
            ("proxy_url", "http://proxy.example:8080"),
            ("proxy_username", "user"),
            ("proxy_password_configured", true),
            ("proxy_bypass", new[] { "localhost", "", "*.internal" })));

        Assert.False(viewModel.VerifyHttpsCertificates);
        Assert.False(viewModel.UseSystemCertificates);
        Assert.Equal("/tmp/ca.pem", viewModel.CertificatePath);
        Assert.Equal("custom", viewModel.ProxyMode);
        Assert.Equal("http://proxy.example:8080", viewModel.ProxyUrl);
        Assert.Equal("user", viewModel.ProxyUsername);
        Assert.True(viewModel.ProxyPasswordConfigured);
        Assert.Equal($"localhost{Environment.NewLine}*.internal", viewModel.ProxyBypassRules);
    }

    [Fact]
    public void ApplyFallsBackForMissingOrInvalidValues()
    {
        var viewModel = new NetworkSettingsViewModel(new FakeViewModelHost());

        viewModel.Apply(Snapshot(
            ("proxy_mode", "socks"),
            ("verify_https_certificates", "yes"),
            ("proxy_bypass", "localhost")));

        Assert.True(viewModel.VerifyHttpsCertificates);
        Assert.True(viewModel.UseSystemCertificates);
        Assert.Equal(string.Empty, viewModel.CertificatePath);
        Assert.Equal("system", viewModel.ProxyMode);
        Assert.False(viewModel.ProxyPasswordConfigured);
        Assert.Equal(string.Empty, viewModel.ProxyBypassRules);
    }

    [Fact]
    public async Task SavesFailWithoutSdkAndDoNotToggleBusy()
    {
        var host = new FakeViewModelHost();
        var viewModel = new NetworkSettingsViewModel(host);

        Assert.False(await viewModel.SaveHttpsCertificateVerificationAsync(false));
        Assert.False(await viewModel.SaveCertificateTrustAsync(false, "/tmp/ca.pem"));
        Assert.False(await viewModel.SaveProxyConfigurationAsync("custom", "http://p", null, null, false, null));

        Assert.True(viewModel.VerifyHttpsCertificates);
        Assert.Equal(string.Empty, viewModel.CertificatePath);
        Assert.Equal("system", viewModel.ProxyMode);
        Assert.Empty(host.BusyChanges);
        Assert.Empty(host.Successes);
    }

    private static SettingsSnapshot Snapshot(params (string Key, object Value)[] values) =>
        new(values
            .Select(item => new SettingRecord(item.Key, JsonSerializer.SerializeToElement(item.Value), "global", string.Empty))
            .ToArray());
}
