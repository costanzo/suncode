using System.Security.Cryptography;
using System.Text;
using Xilium.CefGlue;
using Xilium.CefGlue.BrowserProcess;
using Xilium.CefGlue.Common;

namespace SunCode.Desktop.Infrastructure;

internal static class CefPreviewRuntime
{
    // Rust connects to this CEF-owned CDP endpoint; CEF remains the sole browser process owner.
    internal const int RemoteDebuggingPort = 9222;
    private static bool _initialized;
    // CEF requires every request-context cache to be beneath this root. Keep the
    // application data directory as the common parent for Preview and Browser Use
    // profiles, which intentionally live in separate child directories.
    private static readonly string CacheRoot = AppDataPaths.DataDirectory;
    private static readonly string PreviewCacheRoot = Path.Combine(AppDataPaths.DataDirectory, "cef-preview");

    internal static bool Available { get; private set; }
    internal static string? Error { get; private set; }

    internal static void Initialize()
    {
        if (_initialized) return;
        _initialized = true;
        try
        {
            Directory.CreateDirectory(CacheRoot);
            Directory.CreateDirectory(PreviewCacheRoot);
            CefRuntimeLoader.Initialize(new CefSettings
            {
                RootCachePath = CacheRoot,
                RemoteDebuggingPort = RemoteDebuggingPort,
                // CEF CHECK-fails inside cef_initialize when no subprocess path is set,
                // and CefGlue only resolves the macOS bundle, framework, and resource
                // paths when it is. The app re-enters itself through CefSubProcess.Run.
                BrowserSubprocessPath = CefSubProcess.GetSubProcessPath(),
                // Avalonia owns the macOS Cocoa view hierarchy. Windowed CEF tries to
                // attach another native browser view and collides with AvaloniaNative's
                // Objective-C classes (and can deadlock while the view is attached).
                // The forked Avalonia adapter provides the OSR bitmap/input path. Use
                // it on macOS, where windowed CEF's Cocoa view collides with
                // AvaloniaNative's Objective-C classes. Keep the existing native
                // hosting path on other platforms.
                WindowlessRenderingEnabled = OperatingSystem.IsMacOS(),
                LogSeverity = CefLogSeverity.Error
            });
            Available = true;
            DiagnosticLog.Info("browser.host", $"cef_initialized=true cdp_port={RemoteDebuggingPort}");
        }
        catch (Exception exception)
        {
            Error = "The embedded Chromium runtime is unavailable.";
            DiagnosticLog.Error("browser.preview", exception, "cef_initialize_failed=true");
        }
    }

    internal static CefRequestContext CreateProjectContext(string projectId, bool browserUse = false)
    {
        if (!Available) throw new InvalidOperationException(Error);
        var digest = SHA256.HashData(Encoding.UTF8.GetBytes(projectId));
        var hash = Convert.ToHexString(digest.AsSpan(0, 16)).ToLowerInvariant();
        var profileRoot = browserUse
            ? Path.Combine(AppDataPaths.DataDirectory, "browser", "profiles")
            : PreviewCacheRoot;
        var cachePath = Path.Combine(profileRoot, hash);
        Directory.CreateDirectory(cachePath);
        return CefRequestContext.CreateContext(new CefRequestContextSettings
        {
            CachePath = cachePath,
            PersistSessionCookies = true
        }, null);
    }

    internal static string BrowserTargetUrl(string projectId)
    {
        var digest = SHA256.HashData(Encoding.UTF8.GetBytes(projectId));
        var hash = Convert.ToHexString(digest.AsSpan(0, 16)).ToLowerInvariant();
        return $"data:text/html,%3Ctitle%3Esuncode-browser-use-{hash}%3C%2Ftitle%3E";
    }

    internal static void Shutdown()
    {
        if (!Available) return;
        CefRuntime.Shutdown();
        Available = false;
    }
}
