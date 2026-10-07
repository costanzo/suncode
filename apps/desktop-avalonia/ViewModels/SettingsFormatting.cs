using System.Globalization;
using System.IO;
using SunCode.Desktop.Infrastructure;

namespace SunCode.Desktop.ViewModels;

// Pure display formatting for settings pages. Text resolves through the active
// localization dictionary and falls back to English when no application runs.
internal static class SettingsFormatting
{
    public const string Dash = "—";

    public static string L(string key, string fallback) => LocalizationService.GetString(key, fallback);

    public static string LF(string key, string fallback, params object[] args) =>
        string.Format(CultureInfo.CurrentCulture, L(key, fallback), args);

    public static string EmptyAsDash(string? value) => string.IsNullOrWhiteSpace(value) ? Dash : value;

    public static string ComputerValue(string? value) => value switch
    {
        "allowed" => L("LocAllowed", "Allowed"),
        "denied" => L("LocDenied", "Denied"),
        "unsupported" => L("LocUnsupported", "Unsupported"),
        "primary_display" => L("LocPrimaryDisplay", "Primary display"),
        "user" => L("LocUser", "User"),
        "agent" => L("LocAgent", "Agent"),
        _ => Humanize(value)
    };

    public static string BrowserState(string? value) => value switch
    {
        "ready" => L("LocReady", "Ready"),
        "not_started" => L("LocNotStarted", "Not started"),
        "starting" => L("LocStarting", "Starting"),
        "stopping" => L("LocStopping", "Stopping"),
        "verifying" => L("LocVerifying", "Verifying"),
        "background" => L("LocBackground", "Background"),
        "user_controlled" => L("LocUserControlled", "User controlled"),
        "missing" => L("LocMissing", "Missing"),
        "invalid" => L("LocInvalid", "Invalid"),
        "unsupported" => L("LocUnsupported", "Unsupported"),
        "failed" => L("LocFailed", "Failed"),
        _ => Humanize(value)
    };

    public static StatusTone PermissionTone(string? permission) => permission switch
    {
        "allowed" => StatusTone.Success,
        "denied" => StatusTone.Danger,
        "unsupported" => StatusTone.Warning,
        _ => StatusTone.Muted
    };

    public static StatusTone BrowserStateTone(string? state) => state switch
    {
        "ready" or "background" => StatusTone.Success,
        "verifying" or "starting" or "stopping" or "user_controlled" => StatusTone.Warning,
        "missing" or "invalid" or "unsupported" or "failed" => StatusTone.Danger,
        _ => StatusTone.Muted
    };

    public static StatusTone SaveResultTone(bool saved) => saved ? StatusTone.Success : StatusTone.Danger;

    public static string ByteSize(ulong value)
    {
        if (value >= 1024UL * 1024UL * 1024UL) return $"{value / (1024d * 1024d * 1024d):0.0} GB";
        if (value >= 1024UL * 1024UL) return $"{value / (1024d * 1024d):0.0} MB";
        if (value >= 1024UL) return $"{value / 1024d:0.0} KB";
        return $"{value} B";
    }

    public static string NormalizeDirectory(string? directory) =>
        directory?.Trim().TrimEnd(Path.DirectorySeparatorChar, Path.AltDirectorySeparatorChar) ?? string.Empty;

    public static string NormalizeProxyBypass(string? value) => string.Join('\n',
        (value ?? string.Empty)
            .Split(['\r', '\n'], StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries)
            .Where(item => item.Length > 0));

    // "some_state_value" -> "Some state value"; blank -> dash.
    private static string Humanize(string? value)
    {
        if (string.IsNullOrWhiteSpace(value)) return Dash;
        return string.Join(
            " ",
            value.Split('_', StringSplitOptions.RemoveEmptyEntries)
                .Select((part, index) => index == 0
                    ? char.ToUpperInvariant(part[0]) + part[1..]
                    : part));
    }
}
