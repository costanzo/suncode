using SunCode.Desktop.Infrastructure;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

public sealed partial class DesktopViewModel
{
    private BrowserRuntimeInfo? _browserRuntime;
    private bool _browserRuntimeLoading;
    private string _browserStatusText = string.Empty;

    public BrowserRuntimeInfo? BrowserRuntime
    {
        get => _browserRuntime;
        private set => SetProperty(ref _browserRuntime, value);
    }

    public string BrowserStatusText
    {
        get => _browserStatusText;
        private set => SetProperty(ref _browserStatusText, value);
    }

    public async Task LoadBrowserRuntimeAsync()
    {
        if (_browserRuntimeLoading || !await EnsureSdkReadyAsync()) return;
        _browserRuntimeLoading = true;
        try
        {
            BrowserRuntime = await _sdk!.GetBrowserRuntimeInfoAsync(SelectedProject?.ProjectId);
            BrowserStatusText = string.Empty;
        }
        catch (Exception exception)
        {
            BrowserStatusText = exception.Message;
            ReportError(exception);
        }
        finally
        {
            _browserRuntimeLoading = false;
        }
    }

    public Task<bool> SetBrowserUseEnabledAsync(bool enabled) => RunBrowserMutationAsync(
        async () => BrowserRuntime = await _sdk!.SetBrowserUseEnabledAsync(enabled),
        enabled
            ? LocalizationService.GetString("Loc_BrowserUseEnabledStatus", "Browser Use enabled. Chromium starts only when a project needs it.")
            : LocalizationService.GetString("Loc_BrowserUseDisabledStatus", "Browser Use disabled. Active browser runtimes were stopped."));

    public Task<bool> VerifyBrowserRuntimeAsync() => RunBrowserMutationAsync(
        async () => BrowserRuntime = await _sdk!.VerifyBrowserRuntimeAsync(SelectedProject?.ProjectId),
        LocalizationService.GetString("Loc_BrowserRuntimeVerified", "Bundled browser runtime verified."));

    public Task<bool> StartBrowserProjectAsync() => RunBrowserProjectMutationAsync(
        projectId => _sdk!.StartBrowserProjectAsync(projectId),
        LocalizationService.GetString("Loc_ProjectBrowserStarted", "Project browser started in the background."));

    public Task<bool> TakeBrowserControlAsync() => RunBrowserProjectMutationAsync(
        projectId => _sdk!.TakeBrowserControlAsync(projectId),
        LocalizationService.GetString("Loc_BrowserControlPaused", "Browser tools paused while you control Chromium."));

    public Task<bool> ReturnBrowserControlAsync() => RunBrowserProjectMutationAsync(
        projectId => _sdk!.ReturnBrowserControlAsync(projectId),
        LocalizationService.GetString("Loc_BrowserControlReturned", "Control returned. The agent must take a fresh page snapshot."));

    public Task<bool> RestartBrowserRuntimeAsync() => RunBrowserProjectMutationAsync(
        projectId => _sdk!.RestartBrowserRuntimeAsync(projectId),
        LocalizationService.GetString("Loc_ProjectBrowserRestarted", "Project browser restarted with its persistent profile."));

    public Task<bool> StopBrowserRuntimeAsync() => RunBrowserProjectMutationAsync(
        projectId => _sdk!.StopBrowserRuntimeAsync(projectId),
        LocalizationService.GetString("Loc_ProjectBrowserStopped", "Project browser stopped. Its profile was preserved."));

    public async Task<bool> ClearBrowserProfileAsync()
    {
        if (SelectedProject is null || !await EnsureSdkReadyAsync()) return false;
        try
        {
            await _sdk!.ClearBrowserProfileAsync(SelectedProject.ProjectId);
            BrowserStatusText = string.Format(
                System.Globalization.CultureInfo.CurrentCulture,
                LocalizationService.GetString("Loc_BrowserDataCleared", "Browser data cleared for {0}."),
                SelectedProject.DisplayName);
            await LoadBrowserRuntimeAsync();
            return true;
        }
        catch (Exception exception)
        {
            BrowserStatusText = exception.Message;
            ReportError(exception);
            return false;
        }
    }

    private async Task<bool> RunBrowserProjectMutationAsync(
        Func<string, Task<BrowserRuntimeInfo>> operation,
        string success)
    {
        if (SelectedProject is null)
        {
            BrowserStatusText = LocalizationService.GetString("Loc_OpenProjectToManageBrowser", "Open a project to manage its browser runtime.");
            return false;
        }
        return await RunBrowserMutationAsync(
            async () => BrowserRuntime = await operation(SelectedProject.ProjectId),
            success);
    }

    private async Task<bool> RunBrowserMutationAsync(Func<Task> operation, string success)
    {
        if (!await EnsureSdkReadyAsync()) return false;
        try
        {
            await operation();
            BrowserStatusText = success;
            return true;
        }
        catch (Exception exception)
        {
            BrowserStatusText = exception.Message;
            ReportError(exception);
            return false;
        }
    }
}
