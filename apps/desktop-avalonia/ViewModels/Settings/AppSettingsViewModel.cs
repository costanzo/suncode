using System.Globalization;
using System.Text.Json;
using SunCode.Desktop.Infrastructure;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

// Application-wide appearance, logging, and image storage settings. One
// instance is shared by every window view model so a change made from any
// Settings window is visible everywhere.
public sealed class AppSettingsViewModel : ObservableObject
{
    private string _themeMode = "light";
    private string _language = LocalizationService.DefaultLocale;
    private string _logLevel = "INFO";
    private string _logDirectory = string.Empty;
    private string _imageDirectory = string.Empty;
    private long _logMaxBytes = 10 * 1024 * 1024;
    private int _logRetention = 5;

    public event Action<string>? ThemeChanged;
    public event Action<string>? LanguageChanged;

    public string ThemeMode { get => _themeMode; private set => SetProperty(ref _themeMode, value); }
    public string Language { get => _language; private set => SetProperty(ref _language, value); }
    public string LogLevel { get => _logLevel; private set => SetProperty(ref _logLevel, value); }
    public string LogDirectory { get => _logDirectory; private set => SetProperty(ref _logDirectory, value); }
    public string ImageDirectory { get => _imageDirectory; private set => SetProperty(ref _imageDirectory, value); }
    public string EffectiveLogDirectory => string.IsNullOrWhiteSpace(LogDirectory)
        ? AppDataPaths.DefaultLogDirectory
        : LogDirectory;
    public string EffectiveImageDirectory => string.IsNullOrWhiteSpace(ImageDirectory)
        ? AppDataPaths.DefaultImageDirectory
        : ImageDirectory;
    public long LogMaxBytes { get => _logMaxBytes; private set => SetProperty(ref _logMaxBytes, value); }
    public int LogRetention { get => _logRetention; private set => SetProperty(ref _logRetention, value); }

    internal void Apply(SettingsSnapshot settings)
    {
        var retention = settings.Long("log_retention", 5);
        var configuredLevel = settings.String("log_level", "INFO").Trim().ToUpperInvariant();
        SetLanguage(settings.String("ui_locale", LocalizationService.DefaultLocale));
        LogLevel = configuredLevel is "TRACE" or "DEBUG" or "INFO" or "WARN" or "ERROR" or "OFF"
            ? configuredLevel
            : "INFO";
        LogDirectory = settings.String("log_directory", string.Empty);
        ImageDirectory = settings.String("image_directory", string.Empty);
        var maxBytes = settings.Long("log_max_bytes", 10 * 1024 * 1024);
        LogMaxBytes = maxBytes >= 1024 ? maxBytes : 10 * 1024 * 1024;
        LogRetention = retention is >= 0 and <= 100 ? (int)retention : 5;
        var theme = settings.String("theme_mode", string.Empty);
        if (theme is "dark" or "light") SetTheme(theme);
    }

    internal async Task SaveThemeAsync(IViewModelHost host, string mode)
    {
        if (!host.EnsureSdk() || mode is not ("dark" or "light")) return;
        await RunAsync(host, async sdk =>
        {
            await sdk.SetSettingAsync(new SetSettingRequest(
                "global", null, null, "theme_mode", JsonSerializer.SerializeToElement(mode)));
            SetTheme(mode);
        }, "Theme saved");
    }

    internal async Task SaveLanguageAsync(IViewModelHost host, string locale)
    {
        if (!host.EnsureSdk()) return;
        locale = NormalizeLocale(locale);
        await RunAsync(host, async sdk =>
        {
            await sdk.SetSettingAsync(new SetSettingRequest(
                "global", null, null, "ui_locale", JsonSerializer.SerializeToElement(locale)));
            SetLanguage(locale);
        }, "Language saved");
    }

    // Validation failures are reported through the host status text and leave settings unchanged.
    internal async Task<bool> SaveLoggingSettingsAsync(
        IViewModelHost host,
        string level,
        string? directory,
        string maxBytesText,
        string retentionText)
    {
        if (!host.EnsureSdk()) return false;

        level = level.Trim().ToUpperInvariant();
        directory = directory?.Trim() ?? string.Empty;
        if (string.Equals(directory, AppDataPaths.DefaultLogDirectory, StringComparison.Ordinal))
        {
            directory = string.Empty;
        }
        if (level is not ("TRACE" or "DEBUG" or "INFO" or "WARN" or "ERROR" or "OFF"))
        {
            host.ReportPresentationError("Choose a valid logging level");
            return false;
        }
        if (!long.TryParse(maxBytesText.Trim(), NumberStyles.Integer, CultureInfo.InvariantCulture, out var maxBytes)
            || maxBytes < 1024)
        {
            host.ReportPresentationError("Maximum log size must be at least 1024 bytes");
            return false;
        }
        if (!int.TryParse(retentionText.Trim(), NumberStyles.Integer, CultureInfo.InvariantCulture, out var retention)
            || retention is < 0 or > 100)
        {
            host.ReportPresentationError("Log retention must be between 0 and 100 files");
            return false;
        }

        return await RunAsync(host, async sdk =>
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
        }, "Logging settings saved");
    }

    internal async Task<bool> SaveImageDirectoryAsync(IViewModelHost host, string? directory)
    {
        if (!host.EnsureSdk()) return false;
        directory = directory?.Trim() ?? string.Empty;
        if (string.Equals(directory, AppDataPaths.DefaultImageDirectory, StringComparison.Ordinal))
        {
            directory = string.Empty;
        }
        return await RunAsync(host, async sdk =>
        {
            await sdk.SetSettingAsync(new SetSettingRequest("global", null, null, "image_directory", JsonSerializer.SerializeToElement(directory)));
            ImageDirectory = directory;
        }, "Image storage location saved");
    }

    internal static string NormalizeLocale(string locale) =>
        locale is LocalizationService.SimplifiedChineseLocale
            ? LocalizationService.SimplifiedChineseLocale
            : LocalizationService.DefaultLocale;

    private void SetTheme(string mode)
    {
        ThemeMode = mode;
        ThemeChanged?.Invoke(mode);
    }

    private void SetLanguage(string locale)
    {
        var normalized = NormalizeLocale(locale);
        if (Language == normalized) return;
        Language = normalized;
        LanguageChanged?.Invoke(normalized);
    }

    private static async Task<bool> RunAsync(IViewModelHost host, Func<SunCode.Sdk.AgentSdk, Task> operation, string success)
    {
        host.SetBusy(true);
        try
        {
            await operation(host.Sdk!);
            host.ReportSuccess(success);
            return true;
        }
        catch (Exception exception)
        {
            host.ReportError(exception);
            return false;
        }
        finally
        {
            host.SetBusy(false);
        }
    }
}
