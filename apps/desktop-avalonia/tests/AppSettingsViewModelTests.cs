using System.Text.Json;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.ViewModels;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Tests;

public sealed class AppSettingsViewModelTests
{
    [Fact]
    public void ApplyReadsPersistedAppearanceAndLoggingSettings()
    {
        var settings = new AppSettingsViewModel();
        var themes = new List<string>();
        var locales = new List<string>();
        settings.ThemeChanged += themes.Add;
        settings.LanguageChanged += locales.Add;

        settings.Apply(Snapshot(
            ("theme_mode", "dark"),
            ("ui_locale", LocalizationService.SimplifiedChineseLocale),
            ("log_level", "debug"),
            ("log_directory", "/tmp/logs"),
            ("image_directory", "/tmp/images"),
            ("log_max_bytes", 2048L),
            ("log_retention", 7L)));

        Assert.Equal("dark", settings.ThemeMode);
        Assert.Equal(LocalizationService.SimplifiedChineseLocale, settings.Language);
        Assert.Equal("DEBUG", settings.LogLevel);
        Assert.Equal("/tmp/logs", settings.EffectiveLogDirectory);
        Assert.Equal("/tmp/images", settings.EffectiveImageDirectory);
        Assert.Equal(2048, settings.LogMaxBytes);
        Assert.Equal(7, settings.LogRetention);
        Assert.Equal(["dark"], themes);
        Assert.Equal([LocalizationService.SimplifiedChineseLocale], locales);
    }

    [Fact]
    public void ApplyFallsBackForInvalidValuesAndSkipsUnchangedLocale()
    {
        var settings = new AppSettingsViewModel();
        var locales = new List<string>();
        settings.LanguageChanged += locales.Add;

        settings.Apply(Snapshot(
            ("theme_mode", "sepia"),
            ("ui_locale", "fr-FR"),
            ("log_level", "verbose"),
            ("log_max_bytes", 12L),
            ("log_retention", 500L)));

        Assert.Equal("light", settings.ThemeMode);
        Assert.Equal(LocalizationService.DefaultLocale, settings.Language);
        Assert.Equal("INFO", settings.LogLevel);
        Assert.Equal(10 * 1024 * 1024, settings.LogMaxBytes);
        Assert.Equal(5, settings.LogRetention);
        Assert.Empty(locales);
    }

    [Fact]
    public async Task SaveWithoutSdkFailsWithoutSideEffects()
    {
        var settings = new AppSettingsViewModel();
        var host = new FakeViewModelHost();

        Assert.False(await settings.SaveLoggingSettingsAsync(host, "INFO", null, "100", "5"));
        Assert.Empty(host.PresentationErrors);
        Assert.Empty(host.BusyChanges);
    }

    [Fact]
    public void WindowViewModelsShareOneAppSettingsInstance()
    {
        var shared = new AppSettingsViewModel();
        using var hub = new DesktopViewModel(appSettings: shared);
        using var workspace = new DesktopViewModel(appSettings: shared);

        shared.Apply(Snapshot(("theme_mode", "dark")));

        Assert.Same(hub.AppSettings, workspace.AppSettings);
        Assert.Equal("dark", workspace.AppSettings.ThemeMode);
    }

    private static SettingsSnapshot Snapshot(params (string Key, object Value)[] values) =>
        new(values
            .Select(item => new SettingRecord(item.Key, JsonSerializer.SerializeToElement(item.Value), "global", string.Empty))
            .ToArray());
}
