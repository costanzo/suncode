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
            ComputerStatusText = enabled
                ? "Computer Use enabled. Desktop input still requires approval."
                : "Computer Use disabled and held input was released.";
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
            ComputerStatusText = ComputerRuntime.CapturePermission == "allowed"
                ? "Screen capture permission is available."
                : "Screen capture permission is still unavailable. Review the operating-system prompt or privacy settings.";
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
            ComputerStatusText = ComputerRuntime.InputPermission == "allowed"
                ? "Input control permission is available."
                : "Input control permission is still unavailable. Review the operating-system prompt or privacy settings.";
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
            ComputerStatusText = "Computer Use stopped. Held input was released.";
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
            ComputerStatusText = "You control the desktop. Computer Use tools are paused.";
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
            ComputerStatusText = "Control returned to the agent. A fresh screenshot is required before coordinate input.";
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
