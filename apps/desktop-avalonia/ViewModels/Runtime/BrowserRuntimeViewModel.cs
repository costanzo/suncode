using System.Windows.Input;
using SunCode.Desktop.Infrastructure;
using SunCode.Sdk.Models;
using static SunCode.Desktop.ViewModels.SettingsFormatting;

namespace SunCode.Desktop.ViewModels;

public sealed partial class BrowserRuntimeViewModel : ObservableObject, IPolledPage
{
    private static readonly string[] PresentationProperties =
    [
        nameof(InstallationStateText), nameof(InstallationTone), nameof(RuntimeStateText), nameof(RuntimeTone),
        nameof(RuntimeScopeText), nameof(TargetText), nameof(ChromiumVersionText), nameof(ChromiumPathText),
        nameof(ChromiumPathTip), nameof(CanCopyChromiumPath), nameof(WorkerProtocolText), nameof(IntegrityText),
        nameof(HasProject), nameof(HasNoProject), nameof(ProfilePathText), nameof(ProfilePathTip),
        nameof(CanCopyProfilePath), nameof(ProfileUsageText), nameof(VisibilityText), nameof(ControlTitleText),
        nameof(ControlHintText), nameof(ErrorText), nameof(HasError), nameof(CanVerify), nameof(IsStartVisible),
        nameof(CanStart), nameof(IsTakeControlVisible), nameof(CanTakeControl), nameof(IsReturnControlVisible),
        nameof(IsRestartVisible), nameof(CanRestart), nameof(CanStop), nameof(CanClear)
    ];

    private readonly IViewModelHost _host;
    private readonly AppSettingsViewModel? _appSettings;
    private BrowserRuntimeInfo? _runtime;
    private bool _enabledChecked;
    private bool _loading;
    private string _statusText = string.Empty;
    private ISettingsDialogs? _dialogs;

    internal BrowserRuntimeViewModel(IViewModelHost host, BrowserComponentService? component = null, AppSettingsViewModel? appSettings = null)
    {
        _host = host;
        _appSettings = appSettings;
        _component = component ?? BrowserComponentService.Current;
        InitializeComponentCommands();
        SetEnabledCommand = Command(() => SetBrowserUseEnabledAsync(EnabledChecked));
        VerifyCommand = Command(VerifyBrowserRuntimeAsync);
        StartCommand = Command(StartBrowserProjectAsync);
        TakeControlCommand = Command(TakeBrowserControlAsync);
        ReturnControlCommand = Command(ReturnBrowserControlAsync);
        RestartCommand = Command(RestartBrowserRuntimeAsync);
        StopCommand = Command(StopBrowserRuntimeAsync);
        ClearCommand = new RelayCommand(RequestClearBrowserProfile);
        CopyChromiumPathCommand = new AsyncRelayCommand(() => CopyValueAsync(Runtime?.ChromiumPath));
        CopyProfilePathCommand = new AsyncRelayCommand(() => CopyValueAsync(Runtime?.ProfilePath));
    }

    public BrowserRuntimeInfo? Runtime
    {
        get => _runtime;
        internal set
        {
            SetProperty(ref _runtime, value);
            RefreshPresentation();
        }
    }

    public string StatusText
    {
        get => _statusText;
        private set => SetProperty(ref _statusText, value);
    }
    public ICommand SetEnabledCommand { get; }
    public ICommand VerifyCommand { get; }
    public ICommand StartCommand { get; }
    public ICommand TakeControlCommand { get; }
    public ICommand ReturnControlCommand { get; }
    public ICommand RestartCommand { get; }
    public ICommand StopCommand { get; }
    public ICommand ClearCommand { get; }
    public ICommand CopyChromiumPathCommand { get; }
    public ICommand CopyProfilePathCommand { get; }

    // Two-way toggle state; see ComputerRuntimeViewModel.EnabledChecked.
    public bool EnabledChecked
    {
        get => _enabledChecked;
        set => SetProperty(ref _enabledChecked, value);
    }

    // Until the first runtime load, values match the page's static defaults.
    private bool Loaded => Runtime is not null;
    private bool UserControlled => Runtime?.RuntimeState == "user_controlled";
    private bool NotStarted => Runtime?.RuntimeState == "not_started";
    private bool BrowserUseEnabled => _appSettings?.BrowserUseEnabled ?? Runtime?.Enabled == true;
    private bool InstallationReady => BrowserUseEnabled && Runtime?.InstallationState == "ready" && _componentInspection.State == "ready" && !ComponentBusy;

    public string InstallationStateText => Runtime is null ? L("LocDisabled", "Disabled") : BrowserState(Runtime.InstallationState);
    public StatusTone InstallationTone => BrowserStateTone(Runtime?.InstallationState);
    public string RuntimeStateText => Runtime is null ? L("LocNotStarted", "Not started") : BrowserState(Runtime.RuntimeState);
    public StatusTone RuntimeTone => BrowserStateTone(Runtime?.RuntimeState);
    public string RuntimeScopeText => Loaded && _host.SelectedProject is { } project
        ? LF("LocProjectScope", "Project: {0}", project.DisplayName)
        : L("LocNoProjectSelected", "No project selected");

    public string TargetText => EmptyAsDash(Runtime?.Target);
    public string ChromiumVersionText => EmptyAsDash(Runtime?.ChromiumVersion);
    public string ChromiumPathText => Loaded ? EmptyAsDash(Runtime!.ChromiumPath) : string.Empty;
    public string? ChromiumPathTip => Runtime?.ChromiumPath;
    public bool CanCopyChromiumPath => !Loaded || !string.IsNullOrWhiteSpace(Runtime!.ChromiumPath);
    public string WorkerProtocolText => Runtime?.WorkerProtocolVersion.ToString(System.Globalization.CultureInfo.CurrentCulture) ?? Dash;
    public string IntegrityText => Runtime?.IntegrityState switch
    {
        "verified" => L("LocIntegrityVerified", "Integrity verified"),
        "unverified" => L("LocIntegrityNotVerified", "Integrity not yet verified"),
        _ => L("LocIntegrityUnavailable", "Integrity unavailable")
    };

    public bool HasProject => Loaded && _host.SelectedProject is not null;
    public bool HasNoProject => !HasProject;
    public string ProfilePathText => Loaded ? EmptyAsDash(Runtime!.ProfilePath) : string.Empty;
    public string? ProfilePathTip => Runtime?.ProfilePath;
    public bool CanCopyProfilePath => !Loaded || !string.IsNullOrWhiteSpace(Runtime!.ProfilePath);
    public string ProfileUsageText => Runtime switch
    {
        null => string.Empty,
        { ProfileSizeBytes: { } bytes } runtime =>
            LF("LocProfileUsageValue", "{0} · {1} active pages", ByteSize(bytes), runtime.ActivePageCount),
        _ => Dash
    };
    public string VisibilityText => Runtime is null ? string.Empty : BrowserState(Runtime.VisibilityCapability);
    public string ControlTitleText => UserControlled
        ? L("LocYouControlChromium", "You control Chromium")
        : L("LocAgentControlActive", "Agent control is active");
    public string ControlHintText => !Loaded ? string.Empty
        : UserControlled
            ? L("LocBrowserToolsPaused", "Browser tools are paused. Returning control invalidates previous element references.")
            : L("LocShowingBrowserTransfers", "Showing the browser transfers exclusive control to you and pauses browser tools.");
    public string ErrorText => Runtime?.Error ?? string.Empty;
    public bool HasError => !string.IsNullOrWhiteSpace(Runtime?.Error);

    public bool CanVerify => _componentInspection.Directory is not null && !ComponentBusy && CefPreviewRuntime.ActiveViews == 0;
    public bool IsStartVisible => HasProject && Runtime?.RuntimeState is "not_started" or "failed";
    public bool CanStart => HasProject && InstallationReady;
    public bool IsTakeControlVisible => !Loaded || (HasProject && Runtime!.RuntimeState == "background");
    public bool CanTakeControl => !Loaded || InstallationReady;
    public bool IsReturnControlVisible => HasProject && UserControlled;
    public bool IsRestartVisible => !Loaded || HasProject;
    public bool CanRestart => !Loaded || (InstallationReady && !NotStarted && !UserControlled);
    public bool CanStop => HasProject && (!NotStarted || CefPreviewRuntime.HasProjectBrowser(_host.SelectedProject!.ProjectId));
    public bool CanClear => HasProject && NotStarted && !ComponentBusy && !CefPreviewRuntime.HasProjectBrowser(_host.SelectedProject!.ProjectId);

    Task IPolledPage.ActivateAsync() => LoadAsync();
    Task IPolledPage.PollAsync() => LoadAsync();
    void IPolledPage.Deactivate() { }

    internal void AttachDialogs(ISettingsDialogs? dialogs) => _dialogs = dialogs;

    // Re-reads derived state after a language or project change.
    internal void RefreshPresentation()
    {
        RefreshComponent();
        EnabledChecked = BrowserUseEnabled;
        foreach (var name in PresentationProperties) OnPropertyChanged(name);
    }

    private ICommand Command(Func<Task<bool>> operation) => new AsyncRelayCommand(async () =>
    {
        await operation();
        RefreshPresentation();
    });

    private void RequestClearBrowserProfile()
    {
        if (_host.SelectedProject is not { } project || _dialogs is null) return;
        _dialogs.Confirm(
            new ConfirmationRequest(
                L("LocClearBrowserDataTitle", "Clear browser data?"),
                L("LocClearBrowserDataMessage", "Saved logins, cookies, site storage, and browsing state for this project will be removed. Project files are unchanged."),
                project.DisplayName,
                L("LocProjectBrowserProfile", "PROJECT BROWSER PROFILE"),
                L("LocClearBrowserData", "Clear browser data")),
            () => _ = ClearConfirmedAsync());
    }

    private async Task ClearConfirmedAsync()
    {
        await ClearBrowserProfileAsync();
        RefreshPresentation();
    }

    private async Task CopyValueAsync(string? value)
    {
        if (string.IsNullOrWhiteSpace(value) || _dialogs is null) return;
        if (await _dialogs.CopyTextAsync(value)) StatusText = L("LocCopiedToClipboard", "Copied to clipboard.");
    }

    public async Task LoadAsync()
    {
        if (_loading) return;
        RefreshPresentation();
        if (!await _host.EnsureSdkReadyAsync()) return;
        _loading = true;
        try
        {
            Runtime = await _host.Sdk!.GetBrowserRuntimeInfoAsync(_host.SelectedProject?.ProjectId);
            StatusText = string.Empty;
        }
        catch (Exception exception)
        {
            StatusText = exception.Message;
            _host.ReportError(exception);
        }
        finally
        {
            _loading = false;
        }
    }

    public async Task<bool> SetBrowserUseEnabledAsync(bool enabled)
    {
        var result = await RunMutationAsync(
            async () =>
            {
                Runtime = await _host.Sdk!.SetBrowserUseEnabledAsync(enabled);
                _appSettings?.SetBrowserUseEnabled(Runtime.Enabled);
            },
            enabled
                ? LocalizationService.GetString("LocBrowserUseEnabledStatus", "Browser Use enabled. Chromium starts only when a project needs it.")
                : LocalizationService.GetString("LocBrowserUseDisabledStatus", "Browser Use disabled. Active browser runtimes were stopped."));
        return result;
    }

    public async Task<bool> VerifyBrowserRuntimeAsync()
    {
        await _component.VerifyAsync();
        RefreshPresentation();
        return _componentInspection.State == "ready";
    }

    public Task<bool> StartBrowserProjectAsync() => RunProjectMutationAsync(
        projectId => _host.Sdk!.StartBrowserProjectAsync(projectId),
        LocalizationService.GetString("LocProjectBrowserStarted", "Project browser started in the background."));

    public Task<bool> TakeBrowserControlAsync() => RunProjectMutationAsync(
        projectId => _host.Sdk!.TakeBrowserControlAsync(projectId),
        LocalizationService.GetString("LocBrowserControlPaused", "Browser tools paused while you control Chromium."));

    public Task<bool> ReturnBrowserControlAsync() => RunProjectMutationAsync(
        projectId => _host.Sdk!.ReturnBrowserControlAsync(projectId),
        LocalizationService.GetString("LocBrowserControlReturned", "Control returned. The agent must take a fresh page snapshot."));

    public Task<bool> RestartBrowserRuntimeAsync() => RunProjectMutationAsync(
        projectId => _host.Sdk!.RestartBrowserRuntimeAsync(projectId),
        LocalizationService.GetString("LocProjectBrowserRestarted", "Project browser restarted with its persistent profile."));

    public async Task<bool> StopBrowserRuntimeAsync()
    {
        var result = await RunProjectMutationAsync(projectId => _host.Sdk!.StopBrowserRuntimeAsync(projectId),
            LocalizationService.GetString("LocProjectBrowserStopped", "Project browser stopped. Its profile was preserved."));
        if (result && _host.SelectedProject is { } project)
            (Avalonia.Application.Current as App)?.CloseProjectBrowser(project.ProjectId);
        return result;
    }

    public async Task<bool> ClearBrowserProfileAsync()
    {
        if (_host.SelectedProject is null || !await _host.EnsureSdkReadyAsync()) return false;
        try
        {
            var project = _host.SelectedProject;
            await _host.Sdk!.ClearBrowserProfileAsync(project.ProjectId);
            Runtime = await _host.Sdk!.GetBrowserRuntimeInfoAsync(project.ProjectId);
            StatusText = string.Format(
                System.Globalization.CultureInfo.CurrentCulture,
                LocalizationService.GetString("LocBrowserDataCleared", "Browser data cleared for {0}."),
                project.DisplayName);
            return true;
        }
        catch (Exception exception)
        {
            StatusText = exception.Message;
            _host.ReportError(exception);
            return false;
        }
    }

    private async Task<bool> RunProjectMutationAsync(
        Func<string, Task<BrowserRuntimeInfo>> operation,
        string success)
    {
        if (_host.SelectedProject is null)
        {
            StatusText = LocalizationService.GetString("LocOpenProjectToManageBrowser", "Open a project to manage its browser runtime.");
            return false;
        }
        return await RunMutationAsync(
            async () => Runtime = await operation(_host.SelectedProject.ProjectId),
            success);
    }

    private async Task<bool> RunMutationAsync(Func<Task> operation, string success)
    {
        if (!await _host.EnsureSdkReadyAsync()) return false;
        try
        {
            await operation();
            StatusText = success;
            return true;
        }
        catch (Exception exception)
        {
            StatusText = exception.Message;
            _host.ReportError(exception);
            return false;
        }
    }
}
