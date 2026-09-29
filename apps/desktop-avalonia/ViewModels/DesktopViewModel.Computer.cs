using SunCode.Desktop.Infrastructure;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

public sealed partial class DesktopViewModel
{
    private ComputerRuntimeInfo? _computerRuntime;
    private bool _computerRuntimeLoading;
    private string _computerStatusText = string.Empty;

    public ComputerRuntimeInfo? ComputerRuntime
    {
        get => _computerRuntime;
        private set => SetProperty(ref _computerRuntime, value);
    }

    public string ComputerStatusText
    {
        get => _computerStatusText;
        private set => SetProperty(ref _computerStatusText, value);
    }

    public async Task LoadComputerRuntimeAsync()
    {
        if (_computerRuntimeLoading || !await EnsureSdkReadyAsync()) return;
        _computerRuntimeLoading = true;
        try
        {
            ComputerRuntime = await _sdk!.GetComputerRuntimeInfoAsync();
            ComputerStatusText = string.Empty;
        }
        catch (Exception exception)
        {
            ComputerStatusText = exception.Message;
            ReportError(exception);
        }
        finally
        {
            _computerRuntimeLoading = false;
        }
    }

    public async Task<bool> SetComputerUseEnabledAsync(bool enabled)
    {
        if (!await EnsureSdkReadyAsync()) return false;
        try
        {
            ComputerRuntime = await _sdk!.SetComputerUseEnabledAsync(enabled);
            ComputerStatusText = LocalizationService.GetString(
                enabled ? "Loc_ComputerUseEnabledStatus" : "Loc_ComputerUseDisabledStatus",
                enabled ? "Computer Use enabled. Desktop input still requires approval." : "Computer Use disabled and held input was released.");
            return true;
        }
        catch (Exception exception)
        {
            ComputerStatusText = exception.Message;
            ReportError(exception);
            return false;
        }
    }

    public async Task<bool> RequestComputerCapturePermissionAsync()
    {
        if (!await EnsureSdkReadyAsync()) return false;
        try
        {
            ComputerRuntime = await _sdk!.RequestComputerCapturePermissionAsync();
            ComputerStatusText = LocalizationService.GetString(
                ComputerRuntime.CapturePermission == "allowed" ? "Loc_ScreenCapturePermissionAvailable" : "Loc_ScreenCapturePermissionUnavailable",
                ComputerRuntime.CapturePermission == "allowed" ? "Screen capture permission is available." : "Screen capture permission is still unavailable. Review the operating-system prompt or privacy settings.");
            return ComputerRuntime.CapturePermission == "allowed";
        }
        catch (Exception exception)
        {
            ComputerStatusText = exception.Message;
            ReportError(exception);
            return false;
        }
    }

    public async Task<bool> RequestComputerInputPermissionAsync()
    {
        if (!await EnsureSdkReadyAsync()) return false;
        try
        {
            ComputerRuntime = await _sdk!.RequestComputerInputPermissionAsync();
            ComputerStatusText = LocalizationService.GetString(
                ComputerRuntime.InputPermission == "allowed" ? "Loc_InputControlPermissionAvailable" : "Loc_InputControlPermissionUnavailable",
                ComputerRuntime.InputPermission == "allowed" ? "Input control permission is available." : "Input control permission is still unavailable. Review the operating-system prompt or privacy settings.");
            return ComputerRuntime.InputPermission == "allowed";
        }
        catch (Exception exception)
        {
            ComputerStatusText = exception.Message;
            ReportError(exception);
            return false;
        }
    }

    public async Task<bool> EmergencyStopComputerUseAsync()
    {
        if (!await EnsureSdkReadyAsync()) return false;
        try
        {
            ComputerRuntime = await _sdk!.EmergencyStopComputerUseAsync();
            ComputerStatusText = LocalizationService.GetString("Loc_ComputerUseStopped", "Computer Use stopped. Held input was released.");
            return true;
        }
        catch (Exception exception)
        {
            ComputerStatusText = exception.Message;
            ReportError(exception);
            return false;
        }
    }

    public async Task<bool> TakeComputerControlAsync()
    {
        if (!await EnsureSdkReadyAsync()) return false;
        try
        {
            ComputerRuntime = await _sdk!.TakeComputerControlAsync();
            ComputerStatusText = LocalizationService.GetString("Loc_YouControlDesktop", "You control the desktop. Computer Use tools are paused.");
            return true;
        }
        catch (Exception exception)
        {
            ComputerStatusText = exception.Message;
            ReportError(exception);
            return false;
        }
    }

    public async Task<bool> ReturnComputerControlAsync()
    {
        if (!await EnsureSdkReadyAsync()) return false;
        try
        {
            ComputerRuntime = await _sdk!.ReturnComputerControlAsync();
            ComputerStatusText = LocalizationService.GetString("Loc_ControlReturnedFreshScreenshot", "Control returned to the agent. A fresh screenshot is required before coordinate input.");
            return true;
        }
        catch (Exception exception)
        {
            ComputerStatusText = exception.Message;
            ReportError(exception);
            return false;
        }
    }
}
