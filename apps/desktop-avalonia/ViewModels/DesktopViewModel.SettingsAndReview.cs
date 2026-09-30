using System.Collections.ObjectModel;
using System.Diagnostics;
using System.Globalization;
using System.Text.Json;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Threading;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Sdk;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

public sealed partial class DesktopViewModel : ObservableObject, IDisposable
{
    public bool IsAgentHealthy => DiagnosticsText.Contains("Ready", StringComparison.Ordinal);

    public async Task SaveCredentialAsync(string provider, string apiKey)
    {
        if (!EnsureSdk() || string.IsNullOrWhiteSpace(provider) || string.IsNullOrWhiteSpace(apiKey)) return;
        await RunAsync(async () =>
        {
            await _sdk!.SetCredentialAsync(new SetCredentialRequest(provider, apiKey.Trim()));
            await LoadCredentialsAsync();
            await LoadModelsAsync();
        }, "Credential stored");
    }

    public async Task RemoveCredentialAsync(string provider)
    {
        if (!EnsureSdk() || string.IsNullOrWhiteSpace(provider)) return;
        await RunAsync(async () =>
        {
            await _sdk!.RemoveCredentialAsync(provider);
            await LoadCredentialsAsync();
            await LoadModelsAsync();
        }, "Credential removed");
    }

    public async Task<bool> SaveProviderEndpointAsync(string provider, string? endpoint)
    {
        if (!EnsureSdk()) return false;
        provider = provider.Trim();
        endpoint = endpoint?.Trim() ?? string.Empty;
        if (provider.Length == 0 || endpoint.Length == 0)
        {
            StatusText = "Provider URL is required";
            return false;
        }
        IsBusy = true;
        try
        {
            await _sdk!.SetProviderEndpointAsync(new ProviderEndpointRequest(provider, endpoint));
            await LoadModelsAsync();
            StatusText = "Provider URL saved";
            ConnectionState = "connected";
            return true;
        }
        catch (Exception exception)
        {
            ReportError(exception);
            return false;
        }
        finally
        {
            IsBusy = false;
        }
    }

    public async Task SaveDefaultModelAsync(ModelItem model)
    {
        if (!EnsureSdk()) return;
        await RunAsync(async () =>
        {
            await _sdk!.SetSettingAsync(new SetSettingRequest(
                "global", null, null, "default_model", JsonSerializer.SerializeToElement(model.Id)));
            SelectedModel = model;
        }, "Default model saved");
    }

    public Task SaveThemeAsync(string mode) => AppSettings.SaveThemeAsync(this, mode);

    public Task SaveLanguageAsync(string locale) => AppSettings.SaveLanguageAsync(this, locale);

    public Task<bool> SaveLoggingSettingsAsync(string level, string? directory, string maxBytesText, string retentionText) =>
        AppSettings.SaveLoggingSettingsAsync(this, level, directory, maxBytesText, retentionText);

    public Task<bool> SaveImageDirectoryAsync(string? directory) => AppSettings.SaveImageDirectoryAsync(this, directory);

    public async Task LoadProjectToolCallLimitAsync()
    {
        ToolCallLimit = 640;
        if (_sdk is null || SelectedProject is null) return;

        try
        {
            var result = await _sdk.GetSettingsAsync(new(SelectedProject.ProjectId, null));
            var setting = result.Settings.FirstOrDefault(item => item.Key == "tool_call_limit");
            if (setting is not null
                && setting.Value.TryGetInt32(out var limit)
                && limit is >= 1 and <= 640)
            {
                ToolCallLimit = limit;
            }
        }
        catch (Exception exception)
        {
            ReportError(exception);
        }
    }

    public async Task<bool> SaveProjectToolCallLimitAsync(int limit)
    {
        if (!EnsureSdk() || SelectedProject is null)
        {
            StatusText = "Open a project to configure its tool-call limit";
            return false;
        }
        if (limit is < 1 or > 640)
        {
            StatusText = "Tool-call limit must be between 1 and 640";
            return false;
        }

        IsBusy = true;
        try
        {
            await _sdk!.SetSettingAsync(new SetSettingRequest(
                "project",
                SelectedProject.ProjectId,
                null,
                "tool_call_limit",
                JsonSerializer.SerializeToElement(limit)));
            ToolCallLimit = limit;
            StatusText = "Project tool-call limit saved";
            ConnectionState = "connected";
            return true;
        }
        catch (Exception exception)
        {
            ReportError(exception);
            return false;
        }
        finally
        {
            IsBusy = false;
        }
    }

    public bool IsProviderConfigured(string provider) =>
        Credentials.Any(item => item.Provider == provider && item.Configured);

    public string ProviderModels(string provider)
    {
        var models = Models.Where(item => item.Provider == provider).Select(item => item.Display).ToArray();
        return models.Length == 0 ? "No models available" : string.Join(Environment.NewLine, models);
    }

    public string ProviderEndpoint(string provider) =>
        Providers.FirstOrDefault(item => item.Id == provider)?.ApiBase ?? string.Empty;
}
