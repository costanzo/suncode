using System.Text.Json;
using System.Windows.Input;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Tests;

public sealed class SettingsPageViewModelTests
{
    [Fact]
    public void LoggingSaveIsEnabledOnlyWhileEditsDifferFromSavedValues()
    {
        var logging = new LoggingSettingsViewModel(new FakeViewModelHost(), Settings());
        logging.Load();
        var canExecute = CanExecuteRecorder.For(logging.SaveLoggingCommand);

        Assert.False(logging.IsLoggingDirty);
        Assert.False(logging.SaveLoggingCommand.CanExecute(null));
        Assert.Equal("DEBUG", logging.SelectedLogLevel?.Value);
        Assert.Equal(20m, logging.LogMaxMegabytes);

        logging.LogMaxMegabytes = 30;
        Assert.True(logging.SaveLoggingCommand.CanExecute(null));
        logging.LogMaxMegabytes = 20;
        Assert.False(logging.IsLoggingDirty);

        logging.LogDirectory = logging.LogDirectory + Path.DirectorySeparatorChar + "  ";
        Assert.False(logging.IsLoggingDirty);

        logging.SelectedLogLevel = LoggingSettingsViewModel.LogLevelOptions[0];
        Assert.True(logging.IsLoggingDirty);
        Assert.True(canExecute.Count > 0);
    }

    [Fact]
    public async Task LoggingSaveRejectsOutOfRangeValuesWithDangerTone()
    {
        var host = new FakeViewModelHost();
        var logging = new LoggingSettingsViewModel(host, Settings());
        logging.Load();
        Assert.Equal(StatusTone.Neutral, logging.LoggingStatusTone);

        logging.LogRetention = 101;
        await ((AsyncRelayCommand)logging.SaveLoggingCommand).ExecuteAsync();

        Assert.Equal("Log size must be 1–1000 MB and retained backups must be 0–100", logging.LoggingStatusText);
        Assert.Equal(StatusTone.Danger, logging.LoggingStatusTone);
        Assert.True(logging.IsLoggingDirty);
    }

    [Fact]
    public async Task ImageDirectoryFailureKeepsEditDirtyAndReportsHostStatus()
    {
        var host = new FakeViewModelHost { StatusText = "Agent SDK is not connected" };
        var logging = new LoggingSettingsViewModel(host, Settings());
        logging.Load();

        logging.ImageDirectory = "/tmp/other-images";
        Assert.True(logging.SaveImageDirectoryCommand.CanExecute(null));
        await ((AsyncRelayCommand)logging.SaveImageDirectoryCommand).ExecuteAsync();

        Assert.Equal("Agent SDK is not connected", logging.ImageDirectoryStatusText);
        Assert.Equal(StatusTone.Danger, logging.ImageDirectoryStatusTone);
        Assert.True(logging.IsImageDirectoryDirty);
    }

    [Fact]
    public void NetworkProxyDirtyTrackingIgnoresWhitespaceAndTracksPassword()
    {
        var editor = NetworkEditor(("proxy_mode", "custom"), ("proxy_url", "http://proxy:8080"),
            ("proxy_bypass", new[] { "localhost", "*.internal" }), ("proxy_password_configured", true));

        Assert.True(editor.IsCustomProxy);
        Assert.False(editor.IsProxyDirty);
        Assert.True(editor.IsProxyPasswordStored);
        Assert.Equal("Password stored", editor.ProxyPasswordPlaceholder);
        Assert.Equal("Custom proxy", editor.ProxyStatusText);

        editor.ProxyUrl = "  http://proxy:8080 ";
        editor.ProxyBypass = "localhost\r\n\r\n *.internal ";
        Assert.False(editor.IsProxyDirty);

        editor.ProxyPassword = "secret";
        Assert.True(editor.SaveProxyCommand.CanExecute(null));
        editor.ProxyPassword = string.Empty;
        Assert.False(editor.IsProxyDirty);

        editor.RemoveProxyPasswordCommand.Execute(null);
        Assert.True(editor.ClearProxyPassword);
        Assert.False(editor.IsProxyPasswordStored);
        Assert.Equal("Optional", editor.ProxyPasswordPlaceholder);
        Assert.True(editor.IsProxyDirty);

        editor.ProxyPassword = "new";
        Assert.False(editor.ClearProxyPassword);
    }

    [Fact]
    public void NetworkProxyModeChangeShowsModeSummary()
    {
        var editor = NetworkEditor();

        Assert.Equal("System proxy", editor.ProxyStatusText);
        Assert.False(editor.IsCustomProxy);
        editor.SelectedProxyMode = editor.ProxyModeOptions.Single(item => Equals(item.Value, "no_proxy"));

        Assert.Equal("Direct connections", editor.ProxyStatusText);
        Assert.True(editor.IsProxyDirty);
        Assert.Equal(["No proxy", "System proxy", "Custom proxy"], editor.ProxyModeOptions.Select(item => item.Label));
    }

    [Fact]
    public void NetworkHttpsEditsDriveWarningHintAndDirtyState()
    {
        var editor = NetworkEditor(("certificate_path", "/tmp/ca.pem"));

        Assert.False(editor.IsHttpsDirty);
        Assert.Equal("System trust store", editor.HttpsStatusText);
        editor.VerifyHttpsCertificates = false;
        Assert.True(editor.IsCertificateVerificationOff);
        Assert.Equal("Review required", editor.HttpsStatusText);
        Assert.True(editor.SaveHttpsCommand.CanExecute(null));

        editor.VerifyHttpsCertificates = true;
        editor.UseSystemCertificates = false;
        Assert.Equal("Custom certificate required", editor.HttpsStatusText);
        Assert.StartsWith("Choose a PEM", editor.CertificatePathHint);
        Assert.True(editor.IsHttpsDirty);

        editor.UseSystemCertificates = true;
        editor.CertificatePath = " /tmp/ca.pem ";
        Assert.False(editor.IsHttpsDirty);
    }

    [Fact]
    public void RemoteEditorAppliesStatusToButtonsAndPairing()
    {
        using var remote = new RemoteServerViewModel(new FakeViewModelHost());
        using var editor = new RemoteServerEditorViewModel(remote, action => action());

        remote.ApplyStatus(new RemoteServerStatus(true, true, false, "host-1", "legacy", null,
            "2026-10-08T00:00:00Z", MobilePairingUrl: "https://remote.example/pair?code=1"));

        Assert.False(editor.IsSaveVisible);
        Assert.True(editor.IsDisconnectVisible);
        Assert.True(editor.HasPairing);
        Assert.Equal("https://remote.example/pair?code=1", editor.PairingUrl);
        Assert.Equal($"Host ID: host-1{Environment.NewLine}Access token expires: 2026-10-08T00:00:00Z", editor.PairingMetadata);
        Assert.Equal("Connected", editor.StatusText);

        remote.ApplyStatus(new RemoteServerStatus(true, false, false, null, "legacy", null));
        Assert.True(editor.IsSaveVisible);
        Assert.False(editor.IsDisconnectVisible);
        Assert.Equal("legacy", editor.PairingUrl);
        Assert.Equal("Disconnected", editor.StatusText);

        remote.ApplyStatus(new RemoteServerStatus(false, false, false, null, null, "Offline"));
        Assert.False(editor.HasPairing);
        Assert.Equal("Offline", editor.StatusText);
    }

    [Fact]
    public void ComputerPresentationFollowsRuntimeAndSelectedModel()
    {
        var computer = new ComputerRuntimeViewModel(new FakeViewModelHost());
        Assert.Equal("Unknown", computer.ModelSupportText);
        Assert.Equal("—", computer.TargetDisplayText);
        Assert.True(computer.CanEmergencyStop);

        computer.Runtime = new ComputerRuntimeInfo(true, true, "primary_display", null, null, 1920, 1080,
            "allowed", "denied", "agent", null);
        computer.SelectedModel = new ModelItem("model-a", "p", "P", "configured", false, SupportsComputerUse: true);

        Assert.Equal("Supported", computer.ModelSupportText);
        Assert.Equal(StatusTone.Success, computer.ModelTone);
        Assert.Equal("Ready", computer.BackendStateText);
        Assert.Equal("Primary display · 1920 × 1080 px", computer.TargetDisplayText);
        Assert.False(computer.CanRequestCapturePermission);
        Assert.Equal("Screen capture allowed", computer.CapturePermissionButtonText);
        Assert.True(computer.CanRequestInputPermission);
        Assert.Equal(StatusTone.Danger, computer.InputPermissionTone);
        Assert.True(computer.IsTakeControlVisible);
        Assert.False(computer.IsReturnControlVisible);
        Assert.True(computer.EnabledChecked);

        computer.Runtime = new ComputerRuntimeInfo(false, false, "primary_display", null, null, null, null,
            "unknown", "unknown", "user", "backend missing");
        Assert.Equal("Disabled", computer.BackendStateText);
        Assert.Equal(StatusTone.Muted, computer.BackendTone);
        Assert.False(computer.IsTakeControlVisible);
        Assert.False(computer.IsReturnControlVisible);
        Assert.False(computer.CanEmergencyStop);
        Assert.Equal("backend missing", computer.PageStatusText);
        Assert.False(computer.EnabledChecked);
    }

    [Fact]
    public void BrowserPresentationFollowsRuntimeAndProject()
    {
        var host = new FakeViewModelHost();
        var browser = new BrowserRuntimeViewModel(host);
        Assert.True(browser.HasNoProject);
        Assert.Equal("Not started", browser.RuntimeStateText);

        browser.Runtime = new BrowserRuntimeInfo(true, "ready", "not_started", "", "/opt/chromium", "140", "",
            2, "verified", null, "/tmp/profile", 2048, 1, "headless", null);
        Assert.True(browser.HasNoProject);
        Assert.Equal("No project selected", browser.RuntimeScopeText);
        Assert.Equal("—", browser.TargetText);

        host.SelectedProject = new ProjectItem("project", "Project", "/tmp/project");
        browser.RefreshPresentation();
        Assert.True(browser.HasProject);
        Assert.Equal("Project: Project", browser.RuntimeScopeText);
        Assert.Equal("Ready", browser.InstallationStateText);
        Assert.Equal(StatusTone.Success, browser.InstallationTone);
        Assert.Equal("Integrity verified", browser.IntegrityText);
        Assert.Equal("2.0 KB · 1 active pages", browser.ProfileUsageText);
        Assert.True(browser.IsStartVisible);
        Assert.True(browser.CanStart);
        Assert.True(browser.CanClear);
        Assert.False(browser.CanStop);
        Assert.False(browser.CanRestart);
        Assert.False(browser.HasError);

        browser.Runtime = new BrowserRuntimeInfo(true, "ready", "user_controlled", "", "", "140", "77",
            2, "unverified", null, null, null, 0, "visible", "lost focus");
        Assert.Equal("You control Chromium", browser.ControlTitleText);
        Assert.True(browser.IsReturnControlVisible);
        Assert.False(browser.IsStartVisible);
        Assert.False(browser.CanRestart);
        Assert.True(browser.CanStop);
        Assert.Equal("revision 77", browser.ChromiumVersionText);
        Assert.False(browser.CanCopyChromiumPath);
        Assert.Equal("—", browser.ProfileUsageText);
        Assert.True(browser.HasError);
    }

    private static AppSettingsViewModel Settings()
    {
        var settings = new AppSettingsViewModel();
        settings.Apply(Snapshot(
            ("log_level", "debug"),
            ("log_directory", "/tmp/logs"),
            ("image_directory", "/tmp/images"),
            ("log_max_bytes", 20L * 1024 * 1024),
            ("log_retention", 3)));
        return settings;
    }

    private static NetworkSettingsEditorViewModel NetworkEditor(params (string Key, object Value)[] values)
    {
        var saved = new NetworkSettingsViewModel(new FakeViewModelHost());
        saved.Apply(Snapshot(values));
        var editor = new NetworkSettingsEditorViewModel(new FakeViewModelHost(), saved);
        editor.Load();
        return editor;
    }

    private static SettingsSnapshot Snapshot(params (string Key, object Value)[] values) =>
        new(values
            .Select(item => new SettingRecord(item.Key, JsonSerializer.SerializeToElement(item.Value), "global", string.Empty))
            .ToArray());

    private sealed class CanExecuteRecorder
    {
        public int Count { get; private set; }

        public static CanExecuteRecorder For(ICommand command)
        {
            var recorder = new CanExecuteRecorder();
            command.CanExecuteChanged += (_, _) => recorder.Count++;
            return recorder;
        }
    }
}
