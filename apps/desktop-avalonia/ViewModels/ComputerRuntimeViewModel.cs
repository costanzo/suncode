using System.Windows.Input;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Sdk;
using SunCode.Sdk.Models;
using static SunCode.Desktop.ViewModels.SettingsFormatting;

namespace SunCode.Desktop.ViewModels;

public sealed class ComputerRuntimeViewModel : ObservableObject, IPolledPage
{
    private static readonly string[] PresentationProperties =
    [
        nameof(ModelSupportText), nameof(ModelNameText), nameof(ModelTone),
        nameof(BackendStateText), nameof(BackendTone), nameof(TargetDisplayText),
        nameof(CapturePermissionText), nameof(CapturePermissionTone), nameof(CanRequestCapturePermission),
        nameof(CapturePermissionButtonText), nameof(InputPermissionText), nameof(InputPermissionTone),
        nameof(CanRequestInputPermission), nameof(InputPermissionButtonText), nameof(ControlOwnerText),
        nameof(IsTakeControlVisible), nameof(IsReturnControlVisible), nameof(CanTransferControl),
        nameof(PageStatusText), nameof(CanEmergencyStop)
    ];

    private readonly IViewModelHost _host;
    private ComputerRuntimeInfo? _runtime;
    private ModelItem? _selectedModel;
    private bool _enabledChecked;
    private bool _loading;
    private string _statusText = string.Empty;

    internal ComputerRuntimeViewModel(IViewModelHost host)
    {
        _host = host;
        SetEnabledCommand = Command(() => SetComputerUseEnabledAsync(EnabledChecked));
        RequestCapturePermissionCommand = Command(RequestComputerCapturePermissionAsync);
        RequestInputPermissionCommand = Command(RequestComputerInputPermissionAsync);
        TakeControlCommand = Command(TakeComputerControlAsync);
        ReturnControlCommand = Command(ReturnComputerControlAsync);
        EmergencyStopCommand = Command(EmergencyStopComputerUseAsync);
    }

    public ComputerRuntimeInfo? Runtime
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
        private set
        {
            if (SetProperty(ref _statusText, value)) OnPropertyChanged(nameof(PageStatusText));
        }
    }
    // The window's selected model; the Settings window view model keeps it current.
    public ModelItem? SelectedModel
    {
        get => _selectedModel;
        set
        {
            if (SetProperty(ref _selectedModel, value)) RefreshPresentation();
        }
    }

    public ICommand SetEnabledCommand { get; }
    public ICommand RequestCapturePermissionCommand { get; }
    public ICommand RequestInputPermissionCommand { get; }
    public ICommand TakeControlCommand { get; }
    public ICommand ReturnControlCommand { get; }
    public ICommand EmergencyStopCommand { get; }

    // Two-way toggle state. A click writes the requested value and runs
    // SetEnabledCommand; the resulting runtime then restores the real state.
    public bool EnabledChecked
    {
        get => _enabledChecked;
        set => SetProperty(ref _enabledChecked, value);
    }

    // Until the first runtime load, every value matches the page's static defaults.
    private bool HasRuntime => Runtime is not null;
    private bool ModelSupported => SelectedModel?.SupportsComputerUse == true;
    private static string NoModelText => L("LocNoModelSelected", "No model selected");
    private static string UnknownText => L("LocUnknown", "Unknown");

    public string ModelSupportText => !HasRuntime ? UnknownText
        : SelectedModel is null ? NoModelText
        : ModelSupported ? L("LocSupported", "Supported") : L("LocNotSupported", "Not supported");

    public string ModelNameText => HasRuntime ? SelectedModel?.Id ?? NoModelText : NoModelText;

    public StatusTone ModelTone => !HasRuntime || SelectedModel is null ? StatusTone.Muted
        : ModelSupported ? StatusTone.Success : StatusTone.Warning;

    public string BackendStateText => Runtime switch
    {
        { Enabled: false } => L("LocDisabled", "Disabled"),
        { BackendAvailable: true } => L("LocReady", "Ready"),
        _ => L("LocUnavailable", "Unavailable")
    };

    public StatusTone BackendTone => Runtime switch
    {
        null or { Enabled: false } => StatusTone.Muted,
        { BackendAvailable: true } => StatusTone.Success,
        _ => StatusTone.Danger
    };

    public string TargetDisplayText => Runtime switch
    {
        null => Dash,
        { PixelWidth: { } width, PixelHeight: { } height } runtime =>
            $"{ComputerValue(runtime.TargetDisplay)} · {width} × {height} px",
        var runtime => ComputerValue(runtime.TargetDisplay)
    };

    public string CapturePermissionText => Runtime is null ? UnknownText : ComputerValue(Runtime.CapturePermission);
    public StatusTone CapturePermissionTone => PermissionTone(Runtime?.CapturePermission);
    public bool CanRequestCapturePermission => Runtime is null || Runtime.CapturePermission == "denied";
    public string CapturePermissionButtonText => Runtime?.CapturePermission == "allowed"
        ? L("LocScreenCaptureAllowed", "Screen capture allowed")
        : L("LocRequestScreenCapture", "Request screen capture");

    public string InputPermissionText => Runtime is null ? UnknownText : ComputerValue(Runtime.InputPermission);
    public StatusTone InputPermissionTone => PermissionTone(Runtime?.InputPermission);
    public bool CanRequestInputPermission => Runtime is null || Runtime.InputPermission is "denied" or "unknown";
    public string InputPermissionButtonText => Runtime?.InputPermission == "allowed"
        ? L("LocInputControlAllowed", "Input control allowed")
        : L("LocRequestInputControl", "Request input control");

    public string ControlOwnerText => Runtime is null ? L("LocUser", "User") : ComputerValue(Runtime.ControlOwner);
    private bool AgentControlled => Runtime?.ControlOwner == "agent";
    public bool IsTakeControlVisible => Runtime is null || (Runtime.Enabled && AgentControlled);
    public bool IsReturnControlVisible => Runtime is null || (Runtime.Enabled && !AgentControlled);
    public bool CanTransferControl => Runtime is null || Runtime.BackendAvailable;
    public bool CanEmergencyStop => Runtime is null || (Runtime.Enabled && Runtime.BackendAvailable);

    public string PageStatusText => Runtime is null ? string.Empty
        : string.IsNullOrWhiteSpace(StatusText) ? Runtime.Error ?? string.Empty
        : StatusText;

    Task IPolledPage.ActivateAsync() => LoadAsync();
    Task IPolledPage.PollAsync() => LoadAsync();
    void IPolledPage.Deactivate() { }

    // Re-reads localized text after a language switch.
    internal void RefreshPresentation()
    {
        if (Runtime is { } runtime) EnabledChecked = runtime.Enabled;
        foreach (var name in PresentationProperties) OnPropertyChanged(name);
    }

    private ICommand Command(Func<Task<bool>> operation) => new AsyncRelayCommand(async () =>
    {
        await operation();
        RefreshPresentation();
    });

    public async Task LoadAsync()
    {
        if (_loading || !await _host.EnsureSdkReadyAsync()) return;
        _loading = true;
        try
        {
            Runtime = await _host.Sdk!.GetComputerRuntimeInfoAsync();
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

    public async Task<bool> SetComputerUseEnabledAsync(bool enabled)
    {
        if (!await _host.EnsureSdkReadyAsync()) return false;
        try
        {
            Runtime = await _host.Sdk!.SetComputerUseEnabledAsync(enabled);
            StatusText = LocalizationService.GetString(
                enabled ? "LocComputerUseEnabledStatus" : "LocComputerUseDisabledStatus",
                enabled ? "Computer Use enabled. Desktop input still requires approval." : "Computer Use disabled and held input was released.");
            return true;
        }
        catch (Exception exception)
        {
            StatusText = exception is SdkException { Code: "computer_input_permission_required" }
                ? LocalizationService.GetString(
                    "LocComputerInputPermissionRequired",
                    "Computer Use needs input control permission. Allow SunCode in System Settings > Privacy & Security > Accessibility, then try again.")
                : exception.Message;
            _host.ReportError(exception);
            return false;
        }
    }

    public async Task<bool> RequestComputerCapturePermissionAsync()
    {
        if (!await _host.EnsureSdkReadyAsync()) return false;
        try
        {
            Runtime = await _host.Sdk!.RequestComputerCapturePermissionAsync();
            StatusText = LocalizationService.GetString(
                Runtime.CapturePermission == "allowed" ? "LocScreenCapturePermissionAvailable" : "LocScreenCapturePermissionUnavailable",
                Runtime.CapturePermission == "allowed" ? "Screen capture permission is available." : "Screen capture permission is still unavailable. Review the operating-system prompt or privacy settings.");
            return Runtime.CapturePermission == "allowed";
        }
        catch (Exception exception)
        {
            StatusText = exception.Message;
            _host.ReportError(exception);
            return false;
        }
    }

    public async Task<bool> RequestComputerInputPermissionAsync()
    {
        if (!await _host.EnsureSdkReadyAsync()) return false;
        try
        {
            Runtime = await _host.Sdk!.RequestComputerInputPermissionAsync();
            StatusText = LocalizationService.GetString(
                Runtime.InputPermission == "allowed" ? "LocInputControlPermissionAvailable" : "LocInputControlPermissionUnavailable",
                Runtime.InputPermission == "allowed" ? "Input control permission is available." : "Input control permission is still unavailable. Review the operating-system prompt or privacy settings.");
            return Runtime.InputPermission == "allowed";
        }
        catch (Exception exception)
        {
            StatusText = exception.Message;
            _host.ReportError(exception);
            return false;
        }
    }

    public async Task<bool> EmergencyStopComputerUseAsync()
    {
        if (!await _host.EnsureSdkReadyAsync()) return false;
        try
        {
            Runtime = await _host.Sdk!.EmergencyStopComputerUseAsync();
            StatusText = LocalizationService.GetString("LocComputerUseStopped", "Computer Use stopped. Held input was released.");
            return true;
        }
        catch (Exception exception)
        {
            StatusText = exception.Message;
            _host.ReportError(exception);
            return false;
        }
    }

    public async Task<bool> TakeComputerControlAsync()
    {
        if (!await _host.EnsureSdkReadyAsync()) return false;
        try
        {
            Runtime = await _host.Sdk!.TakeComputerControlAsync();
            StatusText = LocalizationService.GetString("LocYouControlDesktop", "You control the desktop. Computer Use tools are paused.");
            return true;
        }
        catch (Exception exception)
        {
            StatusText = exception.Message;
            _host.ReportError(exception);
            return false;
        }
    }

    public async Task<bool> ReturnComputerControlAsync()
    {
        if (!await _host.EnsureSdkReadyAsync()) return false;
        try
        {
            Runtime = await _host.Sdk!.ReturnComputerControlAsync();
            StatusText = LocalizationService.GetString("LocControlReturnedFreshScreenshot", "Control returned to the agent. A fresh screenshot is required before coordinate input.");
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
