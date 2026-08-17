using System.Reflection;

namespace SunCode.Desktop.Models;

public static class AppInfo
{
    public const string ProductName = "SunCode";

    public static string DisplayVersion
    {
        get
        {
            var version = typeof(AppInfo).Assembly
                .GetCustomAttribute<AssemblyInformationalVersionAttribute>()?
                .InformationalVersion;
            if (string.IsNullOrWhiteSpace(version)) return "v0.0.1";
            var normalized = version.TrimStart('v', 'V').Split('+', 2)[0];
            return $"v{normalized}";
        }
    }
}
