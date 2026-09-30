using System.Security.Cryptography;
using System.Text;
using Xilium.CefGlue;
using Xilium.CefGlue.Common;

namespace SunCode.Desktop.Infrastructure;

internal static class CefPreviewRuntime
{
    private static bool _initialized;
    private static readonly string CacheRoot = Path.Combine(AppDataPaths.DataDirectory, "cef-preview");

    internal static bool Available { get; private set; }
    internal static string? Error { get; private set; }

    internal static void Initialize()
    {
        if (_initialized) return;
        _initialized = true;
        try
        {
            Directory.CreateDirectory(CacheRoot);
            CefRuntimeLoader.Initialize(new CefSettings
            {
                RootCachePath = CacheRoot,
                WindowlessRenderingEnabled = false,
                LogSeverity = CefLogSeverity.Error
            });
            Available = true;
        }
        catch (Exception exception)
        {
            Error = "The embedded Chromium runtime is unavailable.";
            DiagnosticLog.Error("browser.preview", exception, "cef_initialize_failed=true");
        }
    }

    internal static CefRequestContext CreateProjectContext(string projectId)
    {
        if (!Available) throw new InvalidOperationException(Error);
        var hash = Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(projectId))).ToLowerInvariant();
        var cachePath = Path.Combine(CacheRoot, hash);
        Directory.CreateDirectory(cachePath);
        return CefRequestContext.CreateContext(new CefRequestContextSettings
        {
            CachePath = cachePath,
            PersistSessionCookies = true
        }, null);
    }

    internal static void Shutdown()
    {
        if (!Available) return;
        CefRuntime.Shutdown();
        Available = false;
    }
}
