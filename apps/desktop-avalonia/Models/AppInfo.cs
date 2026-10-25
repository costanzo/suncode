using System.Reflection;

namespace SunCode.Desktop.Models;

public static class AppInfo
{
    public const string ProductName = "SunCode";

    public static string DisplayVersion => FormatVersion(
        typeof(AppInfo).Assembly
            .GetCustomAttribute<AssemblyInformationalVersionAttribute>()?
            .InformationalVersion,
        // Fallback shown in About when the assembly has no InformationalVersion.
        "v0.1.0");

    public static string FormatVersion(string? version, string fallback = "Unavailable")
    {
        if (string.IsNullOrWhiteSpace(version)) return fallback;
        var normalized = version.Trim().TrimStart('v', 'V').Split('+', 2)[0];
        return $"v{normalized}";
    }
}
