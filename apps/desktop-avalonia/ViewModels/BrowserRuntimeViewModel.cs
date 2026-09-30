using SunCode.Desktop.Infrastructure;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

public sealed class BrowserRuntimeViewModel : ObservableObject
{
    private readonly IViewModelHost _host;
    private BrowserRuntimeInfo? _runtime;
    private bool _loading;
    private string _statusText = string.Empty;

    internal BrowserRuntimeViewModel(IViewModelHost host) => _host = host;

    public BrowserRuntimeInfo? Runtime
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

    public Task<bool> SetBrowserUseEnabledAsync(bool enabled) => RunMutationAsync(
        async () => Runtime = await _host.Sdk!.SetBrowserUseEnabledAsync(enabled),
        enabled
            ? LocalizationService.GetString("Loc_BrowserUseEnabledStatus", "Browser Use enabled. Chromium starts only when a project needs it.")
            : LocalizationService.GetString("Loc_BrowserUseDisabledStatus", "Browser Use disabled. Active browser runtimes were stopped."));

    public Task<bool> VerifyBrowserRuntimeAsync() => RunMutationAsync(
        async () => Runtime = await _host.Sdk!.VerifyBrowserRuntimeAsync(_host.SelectedProject?.ProjectId),
        LocalizationService.GetString("Loc_BrowserRuntimeVerified", "Bundled browser runtime verified."));

    public Task<bool> StartBrowserProjectAsync() => RunProjectMutationAsync(
        projectId => _host.Sdk!.StartBrowserProjectAsync(projectId),
        LocalizationService.GetString("Loc_ProjectBrowserStarted", "Project browser started in the background."));

    public Task<bool> TakeBrowserControlAsync() => RunProjectMutationAsync(
        projectId => _host.Sdk!.TakeBrowserControlAsync(projectId),
        LocalizationService.GetString("Loc_BrowserControlPaused", "Browser tools paused while you control Chromium."));

    public Task<bool> ReturnBrowserControlAsync() => RunProjectMutationAsync(
        projectId => _host.Sdk!.ReturnBrowserControlAsync(projectId),
        LocalizationService.GetString("Loc_BrowserControlReturned", "Control returned. The agent must take a fresh page snapshot."));

    public Task<bool> RestartBrowserRuntimeAsync() => RunProjectMutationAsync(
        projectId => _host.Sdk!.RestartBrowserRuntimeAsync(projectId),
        LocalizationService.GetString("Loc_ProjectBrowserRestarted", "Project browser restarted with its persistent profile."));

    public Task<bool> StopBrowserRuntimeAsync() => RunProjectMutationAsync(
        projectId => _host.Sdk!.StopBrowserRuntimeAsync(projectId),
        LocalizationService.GetString("Loc_ProjectBrowserStopped", "Project browser stopped. Its profile was preserved."));

    public async Task<bool> ClearBrowserProfileAsync()
    {
        if (_host.SelectedProject is null || !await _host.EnsureSdkReadyAsync()) return false;
        try
        {
            await _host.Sdk!.ClearBrowserProfileAsync(_host.SelectedProject.ProjectId);
            StatusText = string.Format(
                System.Globalization.CultureInfo.CurrentCulture,
                LocalizationService.GetString("Loc_BrowserDataCleared", "Browser data cleared for {0}."),
                _host.SelectedProject.DisplayName);
            await LoadAsync();
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
            StatusText = LocalizationService.GetString("Loc_OpenProjectToManageBrowser", "Open a project to manage its browser runtime.");
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
