using System.Text.Json;
using SunCode.Desktop.Infrastructure;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

public sealed partial class DesktopViewModel
{
    private PreviewState? _previewState;
    private bool _isBrowserConnected;
    public PreviewState? PreviewState { get => _previewState; private set => SetProperty(ref _previewState, value); }
    public bool IsBrowserConnected
    {
        get => _isBrowserConnected;
        set
        {
            if (SetProperty(ref _isBrowserConnected, value))
            {
                OnPropertyChanged(nameof(BrowserGutterTip));
            }
        }
    }
    private string BrowserGutterTip => IsBrowserConnected ? "Show built-in browser (connected)" : "Show built-in browser";
    public bool IsBrowserAvailable => CefPreviewRuntime.Bundled;

    public async Task<bool> StartConfiguredPreviewAsync(string requestedUrl)
    {
        if (SelectedProject is null || !await EnsureSdkReadyAsync()) return false;
        return await RunPreviewOperationAsync("start", async () =>
        {
            var settings = (await _sdk!.GetSettingsAsync(new(SelectedProject.ProjectId, null))).Settings;
            var program = settings.FirstOrDefault(item => item.Key == "preview_program")?.Value.GetString() ?? "npm";
            var url = string.IsNullOrWhiteSpace(requestedUrl)
                ? settings.FirstOrDefault(item => item.Key == "preview_url")?.Value.GetString() ?? "http://localhost:5173/"
                : requestedUrl;
            var cwd = settings.FirstOrDefault(item => item.Key == "preview_cwd")?.Value.GetString();
            var args = settings.FirstOrDefault(item => item.Key == "preview_args")?.Value.ValueKind == JsonValueKind.Array
                ? settings.First(item => item.Key == "preview_args").Value.EnumerateArray().Where(item => item.ValueKind == JsonValueKind.String).Select(item => item.GetString() ?? string.Empty).ToArray()
                : ["run", "dev"];
            await _sdk.SetSettingAsync(new("project", SelectedProject.ProjectId, null, "preview_program", JsonSerializer.SerializeToElement(program)));
            await _sdk.SetSettingAsync(new("project", SelectedProject.ProjectId, null, "preview_url", JsonSerializer.SerializeToElement(url)));
            await _sdk.SetSettingAsync(new("project", SelectedProject.ProjectId, null, "preview_args", JsonSerializer.SerializeToElement(args)));
            if (cwd is not null) await _sdk.SetSettingAsync(new("project", SelectedProject.ProjectId, null, "preview_cwd", JsonSerializer.SerializeToElement(cwd)));
            PreviewState = await _sdk.StartPreviewAsync(SelectedProject.ProjectId, program, args, cwd, url);
        });
    }

    public async Task RefreshPreviewStateAsync()
    {
        if (SelectedProject is null || !await EnsureSdkReadyAsync()) return;
        await RunPreviewOperationAsync("refresh", async () =>
            PreviewState = await _sdk!.GetPreviewStateAsync(SelectedProject.ProjectId));
    }

    public async Task StopPreviewAsync()
    {
        if (SelectedProject is null || !await EnsureSdkReadyAsync()) return;
        await RunPreviewOperationAsync("stop", async () =>
            PreviewState = await _sdk!.StopPreviewAsync(SelectedProject.ProjectId));
    }

    internal async Task<bool> RunPreviewOperationAsync(string operation, Func<Task> action)
    {
        try
        {
            await action();
            return true;
        }
        catch (Exception exception)
        {
            DiagnosticLog.Error("viewmodel.preview", exception, $"operation={operation} project={SelectedProject?.ProjectId ?? "none"}");
            // Preview process failures do not indicate that the embedded agent disconnected.
            ReportPresentationError($"Could not {operation} project preview: {exception.Message}");
            return false;
        }
    }
}
