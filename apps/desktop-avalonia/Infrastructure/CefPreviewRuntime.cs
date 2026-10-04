using System.Security.Cryptography;
using System.Text;
using Xilium.CefGlue;
using Xilium.CefGlue.BrowserProcess;
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
