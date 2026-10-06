using SunCode.Desktop.Infrastructure;
using SunCode.Sdk;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

public sealed class ComputerRuntimeViewModel : ObservableObject
{
    private readonly IViewModelHost _host;
    private ComputerRuntimeInfo? _runtime;
    private bool _loading;
    private string _statusText = string.Empty;

    internal ComputerRuntimeViewModel(IViewModelHost host) => _host = host;

    public ComputerRuntimeInfo? Runtime
    {
        get => _runtime;
        private set => SetProperty(ref _runtime, value);
    }

    public string StatusText
    {
        get => _statusText;
        private set => SetProperty(ref _statusText, value);
    }

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
