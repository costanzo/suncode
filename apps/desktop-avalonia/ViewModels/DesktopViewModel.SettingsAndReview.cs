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
    public Task RefreshProviderTracesAsync() => RefreshProviderTracesAsync(null, null);

    private async Task RefreshProviderTracesAsync(string? requestedSessionId, long? loadVersion)
    {
        if (!EnsureSdk() || SelectedSession is null)
        {
            ClearProviderTraces();
            return;
        }
        var sessionId = requestedSessionId ?? SelectedSession.SessionId;
        if (!IsSessionContextCurrent(sessionId, loadVersion)) return;
        ProviderTraceState = "loading";
        ProviderTraceError = string.Empty;
        try
        {
            var result = await _sdk!.GetProviderExchangesAsync(sessionId);
            if (!IsSessionContextCurrent(sessionId, loadVersion)) return;
            ProviderTraces.Clear();
            ProviderTraceTurns.Clear();
            _providerTraceDetails.Clear();
            _providerTraceDetailLoads.Clear();
            var exchanges = result.Exchanges.Select(ProviderTraceFromSdk).ToList();
            foreach (var exchange in exchanges) ProviderTraces.Add(exchange);
            var latestUsage = result.Exchanges
                .Where(item => item.Usage is not null && (SelectedModel is null || item.ModelId == SelectedModel.Id))
                .OrderBy(item => item.StartedAt, StringComparer.Ordinal)
                .ThenBy(item => item.Iteration)
                .LastOrDefault()?.Usage;
            if (latestUsage is not null) UpdateContextUsage(latestUsage);
            for (var index = 0; index < result.Turns.Count; index++)
            {
                var item = result.Turns[index];
                var turnId = item.TurnId;
                var calls = exchanges.Where(call => call.TurnId == turnId).OrderBy(call => call.Iteration).ThenBy(call => call.StartedAt).ToList();
                ProviderTraceTurns.Add(ProviderTraceTurnFromSdk(item, result.Turns.Count - index, calls));
            }
            ApplyProviderTraceFilter();
            ProviderTraceState = "ready";
            if (SelectedProviderTrace is { } selected)
            {
                await LoadProviderTraceAsync(selected);
            }
        }
        catch (Exception exception)
        {
            if (!IsSessionContextCurrent(sessionId, loadVersion)) return;
            ProviderTraceState = "error";
            ProviderTraceError = exception.Message;
        }
    }

    public async Task LoadProviderTraceAsync(ProviderTraceItem trace)
    {
        if (!EnsureSdk() || SelectedSession is null) return;
        var sessionId = SelectedSession.SessionId;
        SelectedProviderTrace = trace;
        SelectedProviderTraceDetails = trace;
        if (_providerTraceDetails.TryGetValue(trace.ExchangeId, out var cached))
        {
            SelectedProviderTraceDetails = cached;
            return;
        }
        ProviderTraceState = "loading";
        ProviderTraceError = string.Empty;
        try
        {
            var details = await GetProviderTraceDetailsAsync(sessionId, trace.ExchangeId);
            if (SelectedSession?.SessionId != sessionId || SelectedProviderTrace?.ExchangeId != trace.ExchangeId) return;
            SelectedProviderTraceDetails = details;
            ProviderTraceState = "ready";
        }
        catch (Exception exception)
        {
            if (SelectedSession?.SessionId != sessionId || SelectedProviderTrace?.ExchangeId != trace.ExchangeId) return;
            ProviderTraceState = "error";
            ProviderTraceError = exception.Message;
        }
    }

    public void SetProviderTraceFilter(string filter)
    {
        ProviderTraceFilter = filter ?? string.Empty;
        ApplyProviderTraceFilter();
    }

    public void SelectProviderTraceTurn()
    {
        SelectedProviderTrace = null;
        if (ProviderTraceState == "loading") ProviderTraceState = "ready";
    }

    public void SetGitScope(string scope)
    {
        GitScope = scope is "staged" or "unstaged" ? scope : "all";
        OnPropertyChanged(nameof(ScopeAll));
        OnPropertyChanged(nameof(ScopeStaged));
        OnPropertyChanged(nameof(ScopeUnstaged));
        ApplyGitFilter();
        if (SelectedGitFile is not null && FilteredGitFiles.Contains(SelectedGitFile))
            _ = LoadGitDiffAsync(SelectedGitFile, GitScope);
    }

    public void SetGitFilter(string filter)
    {
        GitFilter = filter ?? string.Empty;
        ApplyGitFilter();
        if (SelectedGitFile is not null) _ = LoadGitDiffAsync(SelectedGitFile, GitScope);
    }

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

    public async Task<bool> SaveHttpsCertificateVerificationAsync(bool enabled)
    {
        if (!EnsureSdk()) return false;

        IsBusy = true;
        try
        {
            await _sdk!.SetSettingAsync(new SetSettingRequest("global", null, null, "verify_https_certificates", JsonSerializer.SerializeToElement(enabled)));
            VerifyHttpsCertificates = enabled;
            StatusText = enabled
                ? "HTTPS certificate verification enabled"
                : "HTTPS certificate verification disabled";
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

    public async Task<bool> SaveCertificateTrustAsync(bool useSystem, string? certificatePath)
    {
        if (!EnsureSdk()) return false;
        IsBusy = true;
        try
        {
            certificatePath = certificatePath?.Trim() ?? string.Empty;
            await _sdk!.SetSettingAsync(new SetSettingRequest("global", null, null, "use_system_certificates", JsonSerializer.SerializeToElement(useSystem)));
            await _sdk.SetSettingAsync(new SetSettingRequest("global", null, null, "certificate_path", JsonSerializer.SerializeToElement(certificatePath)));
            UseSystemCertificates = useSystem;
            CertificatePath = certificatePath;
            StatusText = "Certificate trust settings saved";
            ConnectionState = "connected";
            return true;
        }
        catch (Exception exception)
        {
            ReportError(exception);
            return false;
        }
        finally { IsBusy = false; }
    }

    public async Task<bool> SaveProxyConfigurationAsync(
        string mode,
        string? url,
        string? username,
        string? password,
        bool clearPassword,
        string? bypassRules)
    {
        if (!EnsureSdk()) return false;
        IsBusy = true;
        try
        {
            var bypass = (bypassRules ?? string.Empty)
                .Split(['\r', '\n'], StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries)
                .Where(value => value.Length > 0)
                .ToArray();
            var result = await _sdk!.SetProxyConfigurationAsync(new ProxyConfigurationRequest(
                mode,
                url?.Trim() ?? string.Empty,
                username?.Trim() ?? string.Empty,
                string.IsNullOrEmpty(password) ? null : password,
                clearPassword,
                bypass));
            ProxyMode = result.Mode;
            ProxyUrl = result.Url;
            ProxyUsername = result.Username;
            ProxyPasswordConfigured = result.PasswordConfigured;
            ProxyBypassRules = string.Join(Environment.NewLine, result.Bypass);
            StatusText = "Proxy settings applied";
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
        ToolCallLimit = 64;
        if (_sdk is null || SelectedProject is null) return;

        try
        {
            var result = await _sdk.GetSettingsAsync(new(SelectedProject.ProjectId, null));
            var setting = result.Settings.FirstOrDefault(item => item.Key == "tool_call_limit");
            if (setting is not null
                && setting.Value.TryGetInt32(out var limit)
                && limit is >= 1 and <= 256)
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
        if (limit is < 1 or > 256)
        {
            StatusText = "Tool-call limit must be between 1 and 256";
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
