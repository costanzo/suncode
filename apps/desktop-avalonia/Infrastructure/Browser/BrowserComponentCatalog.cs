using System.Runtime.InteropServices;

namespace SunCode.Desktop.Infrastructure;

internal sealed record BrowserComponentPackage(string Target, string Id, string Version, string Sha256, string Library);

internal static class BrowserComponentCatalog
{
    internal const string Version = "152.0.6";
    internal static string SourceUrl(string source) => source switch
    {
        "official" => "https://api.nuget.org/v3/index.json",
        "mirror" => "https://repo.huaweicloud.com/repository/nuget/v3/index.json",
        _ => throw new ArgumentException("Unknown Chromium download source."),
    };

    // Digests of NuGet.org repository-signed packages, shared by all download sources.
    internal static BrowserComponentPackage? ForTarget(string target) => target switch
    {
        "win-x64" => new(target, "chromiumembeddedframework.runtime.win-x64", Version, "c9564f3b223fff6df9d16a3b834f245aa1c2697cd3fbb9a92872a01bfe84990e", "libcef.dll"),
        "win-arm64" => new(target, "chromiumembeddedframework.runtime.win-arm64", Version, "1a7ab5b78fbfbd4fecca6e6fda64bd2808c73e71a9afd1c6df8ae88dfc0311ee", "libcef.dll"),
        "linux-x64" => new(target, "cef.runtime.linux-x64", Version, "f7af22b720afd19f62e6ab670c4783c513ea4a9bca1374b82400b10003a8094e", "libcef.so"),
        "linux-arm64" => new(target, "cef.runtime.linux-arm64", Version, "780ebe991cdc294951c78d741b72770d2884d98e9d18e92d4b8f1509538f0cd4", "libcef.so"),
        // The current binding searches inside the signed .app for framework resources.
        // External macOS frameworks need a binding change before they can be activated.
        _ => null,
    };

    internal static string Target => $"{(OperatingSystem.IsWindows() ? "win" : OperatingSystem.IsLinux() ? "linux" : "osx")}-{RuntimeInformation.ProcessArchitecture.ToString().ToLowerInvariant()}";
    internal static BrowserComponentPackage? Current => ForTarget(Target);
}

internal sealed class BrowserComponentException(string code) : Exception(code)
{
    internal string Code { get; } = code;
}
