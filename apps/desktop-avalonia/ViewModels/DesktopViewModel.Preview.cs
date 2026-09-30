using System.Text.Json;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

public sealed partial class DesktopViewModel
{
    private PreviewState? _previewState;
    public PreviewState? PreviewState { get => _previewState; private set => SetProperty(ref _previewState, value); }

    public async Task<bool> StartConfiguredPreviewAsync(string requestedUrl)
    {
        if (SelectedProject is null || !await EnsureSdkReadyAsync()) return false;
        try
        {
            var settings = (await _sdk!.GetSettingsAsync(new(SelectedProject.ProjectId, null))).Settings;
            var program = settings.FirstOrDefault(item => item.Key == "preview_program")?.Value.GetString() ?? "npm";
            var url = string.IsNullOrWhiteSpace(requestedUrl)
                ? settings.FirstOrDefault(item => item.Key == "preview_url")?.Value.GetString() ?? "http://127.0.0.1:5173/"
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
            return true;
        }
        catch (Exception exception) { ReportError(exception); return false; }
    }

    public async Task RefreshPreviewStateAsync()
    {
        if (SelectedProject is null || !await EnsureSdkReadyAsync()) return;
        try { PreviewState = await _sdk!.GetPreviewStateAsync(SelectedProject.ProjectId); } catch (Exception exception) { ReportError(exception); }
    }

    public async Task StopPreviewAsync()
    {
        if (SelectedProject is null || !await EnsureSdkReadyAsync()) return;
        try { PreviewState = await _sdk!.StopPreviewAsync(SelectedProject.ProjectId); }
        catch (Exception exception) { ReportError(exception); }
    }
}
