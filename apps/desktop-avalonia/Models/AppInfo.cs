using System.Reflection;

namespace SunCode.Desktop.Models;

public static class AppInfo
{
    public const string ProductName = "SunCode";

    public static string DisplayVersion => FormatVersion(
        typeof(AppInfo).Assembly
            .GetCustomAttribute<AssemblyInformationalVersionAttribute>()?
            .InformationalVersion,
        "v0.0.1");

    public static string FormatVersion(string? version, string fallback = "Unavailable")
    {
        if (string.IsNullOrWhiteSpace(version)) return fallback;
        var normalized = version.Trim().TrimStart('v', 'V').Split('+', 2)[0];
        return $"v{normalized}";
    }
}
