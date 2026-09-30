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

    public async Task SaveThemeAsync(string mode)
    {
        if (!EnsureSdk() || mode is not ("dark" or "light")) return;
        await RunAsync(async () =>
        {
            await _sdk!.SetSettingAsync(new SetSettingRequest(
                "global", null, null, "theme_mode", JsonSerializer.SerializeToElement(mode)));
            SetTheme(mode);
        }, "Theme saved");
    }

    public async Task SaveLanguageAsync(string locale)
    {
        if (!EnsureSdk()) return;
        locale = locale is LocalizationService.SimplifiedChineseLocale
            ? LocalizationService.SimplifiedChineseLocale
            : LocalizationService.DefaultLocale;
        await RunAsync(async () =>
        {
            await _sdk!.SetSettingAsync(new SetSettingRequest(
                "global", null, null, "ui_locale", JsonSerializer.SerializeToElement(locale)));
            SetLanguage(locale);
        }, "Language saved");
    }

    public async Task<bool> SaveLoggingSettingsAsync(
        string level,
        string? directory,
        string maxBytesText,
        string retentionText)
    {
        if (!EnsureSdk()) return false;

        level = level.Trim().ToUpperInvariant();
        directory = directory?.Trim() ?? string.Empty;
        if (string.Equals(directory, AppDataPaths.DefaultLogDirectory, StringComparison.Ordinal))
        {
            directory = string.Empty;
        }
        if (level is not ("TRACE" or "DEBUG" or "INFO" or "WARN" or "ERROR" or "OFF"))
        {
            StatusText = "Choose a valid logging level";
            return false;
        }
        if (!long.TryParse(maxBytesText.Trim(), NumberStyles.Integer, CultureInfo.InvariantCulture, out var maxBytes)
            || maxBytes < 1024)
        {
            StatusText = "Maximum log size must be at least 1024 bytes";
            return false;
        }
        if (!int.TryParse(retentionText.Trim(), NumberStyles.Integer, CultureInfo.InvariantCulture, out var retention)
            || retention is < 0 or > 100)
        {
            StatusText = "Log retention must be between 0 and 100 files";
            return false;
        }

        IsBusy = true;
        var sdk = _sdk!;
        try
        {
            await sdk.SetSettingAsync(new SetSettingRequest("global", null, null, "log_level", JsonSerializer.SerializeToElement(level)));
            await sdk.SetSettingAsync(new SetSettingRequest("global", null, null, "log_directory", JsonSerializer.SerializeToElement(directory)));
            await sdk.SetSettingAsync(new SetSettingRequest("global", null, null, "log_max_bytes", JsonSerializer.SerializeToElement(maxBytes)));
            await sdk.SetSettingAsync(new SetSettingRequest("global", null, null, "log_retention", JsonSerializer.SerializeToElement(retention)));
            LogLevel = level;
            LogDirectory = directory;
            LogMaxBytes = maxBytes;
            LogRetention = retention;
            DiagnosticLog.Configure(level, directory, maxBytes, retention);
            StatusText = "Logging settings saved";
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

    public async Task<bool> SaveImageDirectoryAsync(string? directory)
    {
        if (!EnsureSdk()) return false;
        directory = directory?.Trim() ?? string.Empty;
        if (string.Equals(directory, AppDataPaths.DefaultImageDirectory, StringComparison.Ordinal))
        {
            directory = string.Empty;
        }
        IsBusy = true;
        try
        {
            await _sdk!.SetSettingAsync(new SetSettingRequest("global", null, null, "image_directory", JsonSerializer.SerializeToElement(directory)));
            ImageDirectory = directory;
            StatusText = "Image storage location saved";
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
