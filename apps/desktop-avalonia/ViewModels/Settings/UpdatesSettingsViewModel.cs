using System.Windows.Input;
using Avalonia.Threading;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Infrastructure.Updates;
using SunCode.Desktop.Models;
using static SunCode.Desktop.ViewModels.SettingsFormatting;

namespace SunCode.Desktop.ViewModels;

public sealed class UpdatesSettingsViewModel : ObservableObject, IPolledPage, IDisposable
{
    private readonly UpdateCoordinator _coordinator;
    private readonly AsyncRelayCommand _primary;
    private readonly AsyncRelayCommand _check;
    private readonly RelayCommand _cancel;
    private readonly Action<Action> _post;
    private readonly Func<string> _locale;
    private ISettingsDialogs? _dialogs;
    private string _sdkVersion = "";
    private bool? _nativeAvailable;
    private bool _busy = true;
    private bool _disposed;

    internal UpdatesSettingsViewModel(UpdateCoordinator coordinator, Action<Action>? post = null, Func<string>? locale = null)
    {
        _coordinator = coordinator;
        _locale = locale ?? (() => LocalizationService.DefaultLocale);
        _post = post ?? (action => { if (Dispatcher.UIThread.CheckAccess()) action(); else Dispatcher.UIThread.Post(action); });
        _primary = new(PrimaryAsync, () => !IsWorking && Snapshot.Phase != UpdatePhase.Unconfigured && (!IsReady || !_busy));
        _check = new(coordinator.CheckAsync, () => !IsWorking);
        _cancel = new(coordinator.CancelDownload, () => CanCancel);
        coordinator.Changed += Changed;
    }
    private UpdateSnapshot Snapshot => _coordinator.Snapshot;
    private UpdatePlan? Plan => Snapshot.Plan;
    public ICommand PrimaryCommand => _primary;
    public ICommand CheckCommand => _check;
    public ICommand CancelCommand => _cancel;
    public bool IsWorking => Snapshot.Phase is UpdatePhase.Checking or UpdatePhase.Downloading or UpdatePhase.Verifying or UpdatePhase.Restarting;
    public bool IsReady => Snapshot.Phase == UpdatePhase.Ready;
    public bool CanCancel => Snapshot.Phase is UpdatePhase.Downloading or UpdatePhase.Verifying;
    public bool ShowPrimary => !CanCancel;
    public bool ShowCheckAgain => Snapshot.Phase == UpdatePhase.Available;
    public bool ShowProgress => CanCancel;
    public bool IsIndeterminate => Snapshot.Phase == UpdatePhase.Verifying;
    public double Progress => Snapshot.Total == 0 ? 0 : Snapshot.Downloaded * 100d / Snapshot.Total;
    public bool ShowPlanSummary => Plan is not null && !CanCancel;
    public bool ShowNotes => Plan is not null;
    public bool IsRestartBlocked => IsReady && _busy;
    public bool AutomaticChecks { get => _coordinator.AutomaticChecks; set { if (value != AutomaticChecks) _coordinator.SetAutomaticChecks(value); } }
    public string DesktopVersion => AppInfo.DisplayVersion;
    private bool SdkUnavailable => Plan?.Repair == true || _coordinator.SdkHealthy == false || _nativeAvailable == false;
    public string SdkVersion => SdkUnavailable ? L("LocUnavailable", "Unavailable") : _coordinator.Installation?.Sdk is { } sdk ? $"v{sdk.Version}" : _sdkVersion.Length > 0 ? _sdkVersion : L("LocUnavailable", "Unavailable");
    public bool ShowDesktopTarget => Plan?.Desktop == true;
    public bool ShowSdkTarget => Plan?.Sdk == true;
    public string DesktopTarget => ShowDesktopTarget ? $"→  v{Plan!.DesktopVersion}" : "";
    public string SdkTarget => ShowSdkTarget ? $"→  v{Plan!.SdkVersion}" : "";
    private string ComponentSummary => Plan is { Desktop: true, Sdk: true } ? L("LocUpdateBothNames", "Desktop and Agent SDK")
        : Plan?.Desktop == true ? L("LocDesktop", "Desktop") : L("LocAgentSdk", "Agent SDK");
    public string Heading => Snapshot.Phase switch
    {
        UpdatePhase.Unconfigured => L("LocUpdatesNotConfigured", "Updates are not configured for this build"),
        UpdatePhase.Idle => L("LocUpdatesNotChecked", "Check for a newer version"),
        UpdatePhase.Checking => L("LocUpdatesChecking", "Checking for updates…"),
        UpdatePhase.Current => L("LocUpdatesCurrent", "SunCode is up to date"),
        UpdatePhase.Available => LF("LocUpdatesAvailable", "An update for {0} is available", ComponentSummary),
        UpdatePhase.Downloading => L("LocUpdatesDownloading", "Downloading update…"),
        UpdatePhase.Verifying => L("LocUpdatesVerifying", "Verifying update…"),
        UpdatePhase.Ready => L("LocUpdatesReady", "Your update is ready"),
        UpdatePhase.Restarting => L("LocUpdatesRestarting", "Restarting SunCode…"),
        _ => ErrorTitle
    };
    private string ErrorTitle => Snapshot.ErrorCode switch
    {
        "check_timed_out" => L("LocUpdatesCheckTimedOut", "Checking for updates timed out"),
        "download_timed_out" => L("LocUpdatesDownloadTimedOut", "The download timed out"),
        "download_stalled" => L("LocUpdatesDownloadStalled", "The download stopped responding"),
        "integrity_failed" or "signature_invalid" or "platform_signature_invalid" => L("LocUpdatesVerificationFailed", "The update couldn’t be verified"),
        "archive_invalid" or "package_layout" => L("LocUpdatesVerificationFailed", "The update couldn’t be verified"),
        "download_failed" => L("LocUpdatesDownloadFailed", "The download was interrupted"),
        "network_failed" when Snapshot.FailedOperation == UpdateOperation.Download => L("LocUpdatesDownloadFailed", "The download was interrupted"),
        "install_failed" or "activation_invalid" or "helper_failed" => L("LocUpdatesInstallFailed", "The update couldn’t be installed"),
        "installation_unmanaged" => L("LocUpdatesUnmanaged", "Automatic installation is unavailable for this build"),
        "desktop_damaged" or "state_invalid" => L("LocUpdatesRepairDesktop", "This installation needs repair"),
        _ when Snapshot.FailedOperation == UpdateOperation.Installation => L("LocUpdatesInstallFailed", "The update couldn’t be installed"),
        _ when Snapshot.FailedOperation == UpdateOperation.Verification => L("LocUpdatesVerificationFailed", "The update couldn’t be verified"),
        _ => L("LocUpdatesCheckFailed", "Couldn’t check for updates")
    };
    public string Description => Snapshot.Phase switch
    {
        UpdatePhase.Unconfigured => L("LocUpdatesDistributionPending", "Update delivery is not enabled in this build. You can keep using SunCode."),
        UpdatePhase.Idle => L("LocUpdatesInstalledBelow", "Your installed versions are shown below."),
        UpdatePhase.Checking => L("LocUpdatesCheckingBoth", "Looking for updates for both components."),
        UpdatePhase.Current => L("LocUpdatesCurrentHint", "Both components are on the latest compatible version."),
        UpdatePhase.Available => Plan?.Repair == true ? L("LocUpdatesRepairSdkHint", "Download a compatible Agent SDK to repair SunCode, then restart.")
            : Plan is { Desktop: true, Sdk: true } ? L("LocUpdatesPairedHint", "These versions need to be installed together.")
            : L("LocUpdatesAvailableHint", "Download now and restart when you’re ready."),
        UpdatePhase.Downloading => L("LocUpdatesDownloadHint", "You can keep working while the download finishes."),
        UpdatePhase.Verifying => L("LocUpdatesVerifyHint", "The download will be checked before it can be installed."),
        UpdatePhase.Ready => IsRestartBlocked ? L("LocUpdatesBusyHint", "Finish active turns and close running terminals before restarting.") : L("LocUpdatesRestartHint", "Restart SunCode to finish installing."),
        UpdatePhase.Restarting => L("LocUpdatesInstallingHint", "Preparing the downloaded update for a safe restart."),
        _ => ErrorDescription
    };
    private string ErrorDescription => Snapshot.ErrorCode switch
    {
        "check_timed_out" => L("LocUpdatesCheckTimedOutHint", "The update service took too long to respond. Check your connection and try again."),
        "download_timed_out" => L("LocUpdatesDownloadTimedOutHint", "The download took too long. Your installed versions have not changed. Try downloading again."),
        "download_stalled" => L("LocUpdatesDownloadStalledHint", "No data was received for 30 seconds. Your installed versions have not changed. Try downloading again."),
        "configuration_missing" => L("LocUpdatesDistributionPending", "Update delivery is not enabled in this build. You can keep using SunCode."),
        "integrity_failed" or "signature_invalid" or "platform_signature_invalid" => L("LocUpdatesInvalidDownloadHint", "It won’t be installed. Check for updates and download a fresh copy."),
        "archive_invalid" or "package_layout" => L("LocUpdatesInvalidDownloadHint", "It won’t be installed. Check for updates and download a fresh copy."),
        "download_failed" => L("LocUpdatesDownloadFailedHint", "Your installed versions have not changed. Try downloading again."),
        "network_failed" when Snapshot.FailedOperation == UpdateOperation.Download => L("LocUpdatesDownloadFailedHint", "Your installed versions have not changed. Try downloading again."),
        "installation_unmanaged" => L("LocUpdatesUnmanagedHint", "This build wasn’t installed with automatic update support. You can keep using SunCode."),
        "desktop_damaged" or "state_invalid" => L("LocUpdatesReinstallHint", "Reinstall SunCode from an official distribution to repair update support. Your project files are not changed."),
        "no_compatible_update" or "target_mismatch" or "schema_unsupported" => L("LocUpdatesIncompatibleHint", "No compatible update is available for this installation. Check again later or reinstall SunCode."),
        "clock_invalid" => L("LocUpdatesClockHint", "Check your system clock and try again."),
        "catalog_expired" => L("LocUpdatesExpiredHint", "The update information has expired. Check again before installing."),
        "install_failed" or "activation_invalid" or "helper_failed" => L("LocUpdatesInstallationFailedHint", "Your previous installation is still available. Check for updates and try again."),
        _ => L("LocUpdatesConnectionHint", "Check your connection and try again. Your installed versions have not changed.")
    };
    public StatusTone Tone => Snapshot.ErrorCode == "installation_unmanaged" ? StatusTone.Warning : Snapshot.Phase == UpdatePhase.Failed ? StatusTone.Danger : IsRestartBlocked || Snapshot.Phase == UpdatePhase.Unconfigured ? StatusTone.Warning : Snapshot.Phase is UpdatePhase.Current or UpdatePhase.Ready ? StatusTone.Success : StatusTone.Neutral;
    public string PrimaryText => IsReady ? L("LocUpdatesRestartAction", "Restart SunCode")
        : Snapshot.Phase == UpdatePhase.Checking ? L("LocUpdatesChecking", "Checking for updates…")
        : Snapshot.Phase == UpdatePhase.Restarting ? L("LocUpdatesRestarting", "Restarting SunCode…")
        : Plan?.Repair == true ? L("LocUpdatesRepairSdk", "Repair Agent SDK")
        : Plan is not null ? Snapshot.Phase == UpdatePhase.Failed ? L("LocUpdatesDownloadAgain", "Download again") : L("LocUpdatesDownload", "Download update")
        : L("LocUpdatesCheck", "Check for updates");
    public string CancelText => IsIndeterminate ? L("LocUpdatesCancelUpdate", "Cancel update") : L("LocUpdatesCancelDownload", "Cancel download");
    public string PlanSummary => IsReady ? LF("LocUpdatesDownloadedSummary", "Downloaded · {0}", ComponentSummary)
        : Plan is null ? "" : $"{ByteSize((ulong)Plan.SizeBytes)} · {(Plan.Desktop && Plan.Sdk ? L("LocUpdatesBoth", "Both components") : LF("LocUpdatesOnly", "{0} only", ComponentSummary))}";
    public string ProgressText => IsIndeterminate ? L("LocUpdatesCheckingIntegrity", "Checking download integrity")
        : $"{ByteSize((ulong)Math.Max(0, Snapshot.Downloaded))} / {ByteSize((ulong)Math.Max(0, Snapshot.Total))} · {Progress:0}%";
    public string DesktopStatus => ComponentStatus(ShowDesktopTarget);
    public string SdkStatus => SdkUnavailable || _coordinator.Installation?.Sdk is null && _nativeAvailable != true ? L("LocUnavailable", "Unavailable")
        : _coordinator.SdkHealthy is null && _coordinator.Installation?.Sdk is not null ? L("LocChecking", "Checking…") : ComponentStatus(ShowSdkTarget);
    private string ComponentStatus(bool affected) => affected ? Snapshot.Phase == UpdatePhase.Failed ? L("LocUpdateNotInstalled", "Update not installed")
        : IsReady ? L("LocUpdatesReadyToInstall", "Ready to install")
        : Snapshot.Phase == UpdatePhase.Downloading ? L("LocDownloading", "Downloading")
        : IsIndeterminate ? L("LocVerifying", "Verifying") : L("LocUpdateAvailable", "Update available")
        : Snapshot.Phase is UpdatePhase.Available or UpdatePhase.Current or UpdatePhase.Ready ? L("LocUpToDate", "Up to date") : L("LocInstalled", "Installed");
    public string Notes => Plan is null ? "" : string.Join("\n\n", new (bool Included, string Name, ComponentRelease Release)[]
    {
        (Plan.Desktop, L("LocDesktop", "Desktop"), Plan.Verified.Catalog.Components.Desktop),
        (Plan.Sdk, L("LocAgentSdk", "Agent SDK"), Plan.Verified.Catalog.Components.Sdk)
    }.Where(x => x.Included).Select(x => $"{x.Name} v{x.Release.Version}\n{LocalizedNotes(x.Release.ReleaseNotes)}"));
    private string LocalizedNotes(Dictionary<string, string> notes)
    {
        var locale = _locale();
        return notes.FirstOrDefault(pair => pair.Key.Equals(locale, StringComparison.OrdinalIgnoreCase)).Value
            ?? notes.FirstOrDefault(pair => pair.Key.Equals(locale.Split('-')[0], StringComparison.OrdinalIgnoreCase)).Value ?? notes["en-US"];
    }
    internal void AttachDialogs(ISettingsDialogs? dialogs) => _dialogs = dialogs;
    internal async Task LoadIdentityAsync()
    {
        if (_coordinator.Installation?.Sdk is { } installed)
            _coordinator.RecordSdkHealth(await Task.Run(() => UpdatePlanner.Healthy(installed)));
        else
        {
            try
            {
                var running = await _coordinator.ReadRunningVersionsAsync();
                _nativeAvailable = running.SdkAbi == running.DesktopAbi && !string.IsNullOrWhiteSpace(running.SdkVersion);
                _sdkVersion = _nativeAvailable == true ? AppInfo.FormatVersion(running.SdkVersion) : "";
                _coordinator.RecordSdkRuntimeAvailability(_nativeAvailable == true);
            }
            catch (Exception) { _sdkVersion = ""; _nativeAvailable = false; _coordinator.RecordSdkRuntimeAvailability(false); }
        }
        RefreshLocalizedText();
    }
    private Task PrimaryAsync()
    {
        if (IsReady)
        {
            _dialogs?.Confirm(new(
                L("LocUpdatesRestartTitle", "Restart SunCode to update?"),
                L("LocUpdatesRestartMessage", "The downloaded update will be installed. Save your work in every SunCode window before continuing."),
                ComponentSummary, L("LocUpdates", "Updates"), L("LocUpdatesRestartConfirm", "Restart and install"),
                L("LocUpdatesLater", "Later"), Destructive: false), () => _ = _coordinator.RestartAsync());
            return Task.CompletedTask;
        }
        return Plan is null ? _coordinator.CheckAsync() : _coordinator.DownloadAsync();
    }
    private void Changed() => _post(() => { if (!_disposed) RefreshLocalizedText(); });
    internal void RefreshLocalizedText()
    {
        foreach (var name in new[] { nameof(Heading), nameof(Description), nameof(Tone), nameof(PrimaryText), nameof(CancelText), nameof(IsWorking), nameof(IsReady), nameof(CanCancel), nameof(ShowPrimary), nameof(ShowCheckAgain), nameof(ShowProgress), nameof(IsIndeterminate), nameof(Progress), nameof(ProgressText), nameof(ShowPlanSummary), nameof(PlanSummary), nameof(DesktopVersion), nameof(SdkVersion), nameof(ShowDesktopTarget), nameof(ShowSdkTarget), nameof(DesktopTarget), nameof(SdkTarget), nameof(DesktopStatus), nameof(SdkStatus), nameof(ShowNotes), nameof(Notes), nameof(AutomaticChecks), nameof(IsRestartBlocked) }) OnPropertyChanged(name);
        _primary.RaiseCanExecuteChanged(); _check.RaiseCanExecuteChanged(); _cancel.RaiseCanExecuteChanged();
    }
    public async Task PollAsync()
    {
        if (!IsReady) return;
        try { _busy = await _coordinator.IsBusyAsync(); }
        catch (Exception) { _busy = true; }
        RefreshLocalizedText();
    }
    public async Task ActivateAsync() { await LoadIdentityAsync(); await PollAsync(); }
    public void Deactivate() { }
    public void Dispose() { _disposed = true; _coordinator.Changed -= Changed; _dialogs = null; }
}
