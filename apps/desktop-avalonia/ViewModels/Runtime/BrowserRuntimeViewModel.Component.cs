using System.Windows.Input;
using SunCode.Desktop.Controls;
using SunCode.Desktop.Infrastructure;
using SunCode.Sdk.Models;
using static SunCode.Desktop.ViewModels.SettingsFormatting;

namespace SunCode.Desktop.ViewModels;

public sealed record BrowserComponentIdentity(string Label, string Value, ICommand CopyCommand)
{
    public string CopyLabel => string.Format(System.Globalization.CultureInfo.CurrentCulture,
        LocalizationService.GetString("LocBrowserCopyIdentity", "Copy {0}"), Label);
}

public sealed partial class BrowserRuntimeViewModel
{
    private readonly BrowserComponentService _component;
    private BrowserComponentStatus _componentStatus = new("missing");
    private BrowserComponentInspection _componentInspection = new("missing");
    private IReadOnlyList<SCComboBoxItem> _downloadSources = [];
    public IReadOnlyList<SCComboBoxItem> DownloadSources => _downloadSources;
    public SCComboBoxItem? SelectedDownloadSource
    {
        get => _downloadSources.FirstOrDefault(item => Equals(item.Value, _component.Source));
        set
        {
            if (value?.Value is not string source || source == _component.Source) return;
            try { _component.SelectSource(source); }
            catch (Exception exception) { StatusText = exception.Message; }
            RefreshPresentation();
        }
    }
    public string ComponentStateText => L("LocBrowserComponentState_" + _componentStatus.State, _componentStatus.State);
    public StatusTone ComponentTone => _componentStatus.State switch
    {
        "ready" => StatusTone.Success,
        "missing" or "unsupported" => StatusTone.Muted,
        "downloading" or "verifying" or "verifying_files" or "installing" or "uninstalling" or "restart_required" or "uninstall_restart_required" => StatusTone.Warning,
        _ => StatusTone.Danger,
    };
    public bool ComponentBusy => _component.Busy;
    public bool CanCancelComponent => ComponentBusy && _componentStatus.State != "uninstalling";
    public bool ComponentIndeterminate => _componentStatus.State != "downloading" || _componentStatus.Total is null;
    public double ComponentProgress => _componentStatus.Total is > 0 ? Math.Clamp(_componentStatus.Downloaded * 100.0 / _componentStatus.Total.Value, 0, 100) : 0;
    public string ComponentProgressText => _componentStatus.State == "downloading"
        ? _componentStatus.Total is > 0 ? $"{ComponentProgress:0}% · {ByteSize((ulong)_componentStatus.Downloaded)} / {ByteSize((ulong)_componentStatus.Total.Value)}" : ByteSize((ulong)_componentStatus.Downloaded)
        : ComponentStateText;
    public string ComponentMessage
    {
        get
        {
            if (_componentStatus.Error == "browser_active" || CefPreviewRuntime.ActiveViews != 0) return L("LocBrowserStopBeforeRepair", "Stop all project browsers and built-in previews before repairing or uninstalling Chromium.");
            var message = L("LocBrowserComponentMessage_" + _componentStatus.State, "");
            if (_componentInspection.State == "ready" && _componentStatus.State is "hash_mismatch" or "download_failed" or "source_missing")
                message += " " + L("LocBrowserPreviousRetained", "The previously installed component is still available.");
            return message;
        }
    }
    public string RequiredComponentText => $"{BrowserComponentCatalog.Target} · {BrowserComponentCatalog.Version}";
    public string DownloadSourceUrl => BrowserComponentCatalog.SourceUrl(_component.Source);
    public bool CanSelectDownloadSource => !ComponentBusy && BrowserComponentCatalog.Current is not null && !ComponentRestartRequired && !_component.UninstallPending;
    public bool ShowComponentInstall => !ComponentBusy && !ComponentRestartRequired && !_component.UninstallPending && BrowserComponentCatalog.Current is not null;
    public bool CanInstallComponent => ShowComponentInstall && CefPreviewRuntime.ActiveViews == 0;
    public bool ShowComponentUninstall => !ComponentBusy && _componentInspection.State != "uninstall_restart_required" && _component.HasInstallation;
    public bool CanUninstallComponent => ShowComponentUninstall && CefPreviewRuntime.ActiveViews == 0;
    public bool InstallComponentPrimary => _componentInspection.State != "ready";
    public bool ComponentRestartRequired => _componentInspection.State is "restart_required" or "uninstall_restart_required";
    public string UninstallComponentLabel => _componentStatus.State == "uninstall_failed"
        ? L("LocBrowserRetryUninstall", "Retry uninstall")
        : L("LocBrowserUninstall", "Uninstall Chromium");
    public string InstallComponentLabel => _componentStatus.State switch
    {
        "incompatible" => L("LocBrowserInstallCompatible", "Install compatible version"),
        "download_failed" or "hash_mismatch" or "source_missing" => L("LocBrowserRetryDownload", "Retry download"),
        _ => _componentInspection.State == "missing" ? L("LocBrowserDownload", "Download Chromium") : L("LocBrowserRepair", "Repair Chromium"),
    };
    public IReadOnlyList<BrowserComponentIdentity> ComponentDiagnostics
    {
        get
        {
            var package = BrowserComponentCatalog.Current;
            BrowserComponentIdentity Row(string key, string fallback, string? value)
            {
                var text = value ?? "—";
                return new(L(key, fallback), text, new AsyncRelayCommand(() => CopyValueAsync(text)));
            }
            return [
                Row("LocTarget", "Target", BrowserComponentCatalog.Target),
                Row("LocBrowserRequiredVersion", "Required version", package?.Version),
                Row("LocBrowserInstalledVersion", "Installed version", _componentInspection.State is "ready" or "restart_required" ? package?.Version : null),
                Row("LocBrowserNuGetPackage", "NuGet package", package?.Id),
                Row("LocBrowserPackageSha256", "Package SHA-256", package?.Sha256),
                Row("LocBrowserManifest", "Manifest", L("LocBrowserTrustedManifest", "SunCode trusted component manifest")),
                Row("LocBrowserComponentPath", "Component path", _componentInspection.Directory),
                Row("LocBrowserCdpEndpoint", "CDP endpoint", CefPreviewRuntime.Available ? "127.0.0.1:9222" : L("LocNotStarted", "Not started")),
            ];
        }
    }

    public ICommand InstallComponentCommand { get; private set; } = null!;
    public ICommand CancelComponentCommand { get; private set; } = null!;
    public ICommand RestartComponentCommand { get; private set; } = null!;
    public ICommand UninstallComponentCommand { get; private set; } = null!;
    private void InitializeComponentCommands()
    {
        InstallComponentCommand = new AsyncRelayCommand(async () =>
        {
            if (!CanInstallComponent) return;
            try
            {
                var network = await _host.EnsureSdkReadyAsync()
                    ? await _host.Sdk!.GetDownloadNetworkConfigurationAsync()
                    : new DownloadNetworkConfiguration(true, true, null, new("system", "", "", "", []));
                var installation = _component.InstallAsync(network);
                RefreshPresentation();
                await installation;
            }
            catch (Exception exception)
            {
                StatusText = L("LocBrowserComponentMessage_download_failed", "The download failed. Retry or choose another download source.");
                DiagnosticLog.Error("browser.install", exception, "download_configuration_failed=true");
            }
            finally { RefreshPresentation(); }
        });
        CancelComponentCommand = new RelayCommand(() => _component.Cancel());
        UninstallComponentCommand = new RelayCommand(() =>
        {
            if (!CanUninstallComponent) return;
            _dialogs?.Confirm(new(
                L("LocBrowserUninstallTitle", "Uninstall Chromium?"),
                L("LocBrowserUninstallMessage", "Remove the downloaded Chromium runtime and temporary packages. Saved browser data is retained. Download Chromium again to use browser features. A loaded runtime is removed after restarting SunCode."),
                "Chromium", "", L("LocBrowserUninstall", "Uninstall Chromium")), () => _ = UninstallComponentAsync());
        });
        RestartComponentCommand = new RelayCommand(() => _dialogs?.Confirm(new(
            L("LocBrowserRestartTitle", "Restart SunCode?"),
            _component.UninstallPending
                ? L("LocBrowserUninstallRestartMessage", "Restart to finish removing Chromium. Active turns must finish or be stopped first.")
                : L("LocBrowserRestartMessage", "Restart to activate the verified Chromium component. Active turns must finish or be stopped first."),
            "Chromium", "", L("LocBrowserRestartAction", "Restart SunCode"), Destructive: false), () => _ = RestartComponentAsync()));
        RefreshComponent();
    }
    internal async Task UninstallComponentAsync()
    {
        if (!CanUninstallComponent) return;
        var uninstall = _component.UninstallAsync();
        RefreshPresentation();
        await uninstall;
        RefreshPresentation();
    }
    private async Task RestartComponentAsync()
    {
        try { if (Avalonia.Application.Current is App app) await app.RestartBrowserComponentAsync(); }
        catch (Exception exception) { StatusText = exception.Message; }
    }
    private void RefreshComponent()
    {
        _componentStatus = _component.Status;
        _componentInspection = _component.Inspect();
        var mirror = L("LocBrowserMirror", "Huawei Cloud (China)");
        if (_downloadSources.Count == 0 || _downloadSources[1].Label != mirror)
        {
            _downloadSources = [new("NuGet.org", "official"), new(mirror, "mirror")];
            OnPropertyChanged(nameof(DownloadSources));
        }
        foreach (var name in new[] { nameof(ComponentStateText), nameof(ComponentTone), nameof(ComponentBusy), nameof(ComponentIndeterminate),
            nameof(ComponentProgress), nameof(ComponentProgressText), nameof(ComponentMessage), nameof(RequiredComponentText), nameof(DownloadSourceUrl),
            nameof(CanSelectDownloadSource), nameof(ShowComponentInstall), nameof(CanInstallComponent), nameof(ComponentRestartRequired),
            nameof(CanCancelComponent), nameof(ShowComponentUninstall), nameof(CanUninstallComponent), nameof(UninstallComponentLabel),
            nameof(InstallComponentLabel), nameof(InstallComponentPrimary), nameof(ComponentDiagnostics), nameof(SelectedDownloadSource) }) OnPropertyChanged(name);
    }
}
