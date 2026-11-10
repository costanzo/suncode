using System.Security.Cryptography;
using System.Text;

namespace SunCode.Desktop.Infrastructure;

internal static class CefPreviewRuntime
{
    internal const int RemoteDebuggingPort = 9222;
    private static bool _configured;
    private static int _activeViews;
    private static readonly Dictionary<string, int> ProjectBrowsers = new();
    internal static int ActiveViews => Volatile.Read(ref _activeViews);
    internal static bool HasProjectBrowser(string projectId)
    {
        lock (ProjectBrowsers) return ProjectBrowsers.GetValueOrDefault(projectId) > 0;
    }
    internal static bool Available { get; private set; }
    internal static string? Error { get; private set; }

    internal static void Initialize()
    {
        try
        {
            var service = BrowserComponentService.Current;
            var directory = service.RequireReady();
            if (!_configured)
            {
                service.VerifyFilesAsync(directory, CancellationToken.None).GetAwaiter().GetResult();
                service.MarkLoaded(directory);
                CefBrowserHost.Initialize(directory, Path.Combine(AppDataPaths.DataDirectory, "cef"));
                _configured = true;
            }
            Error = null;
            Available = true;
        }
        catch (Exception exception)
        {
            Available = false;
            Error = LocalizationService.GetString("LocBrowserInstallGuidance", "Install or repair Chromium in Settings → Browser use.");
            DiagnosticLog.Error("browser.host", exception, "cef_initialize_failed=true");
        }
    }

    internal static void RunSubprocess(string[] args)
    {
        // Resolve before logging, application IPC, Avalonia, or opening the agent.
        var directory = BrowserComponentService.Current.RequireReady();
        CefBrowserHost.RunSubprocess(directory, args);
    }

    internal static IBrowserView CreateView(string projectId, bool browserUse)
    {
        if (!Available) throw new InvalidOperationException(Error);
        var profile = Path.Combine(AppDataPaths.DataDirectory, "cef", $"{(browserUse ? "browser" : "preview")}--{ProjectHash(projectId)}");
        var view = CefBrowserHost.CreateView(profile, !browserUse);
        Interlocked.Increment(ref _activeViews);
        if (browserUse) lock (ProjectBrowsers) ProjectBrowsers[projectId] = ProjectBrowsers.GetValueOrDefault(projectId) + 1;
        return new TrackedView(view, browserUse ? projectId : null);
    }

    private static string ProjectHash(string projectId) => Convert.ToHexStringLower(SHA256.HashData(Encoding.UTF8.GetBytes(projectId)).AsSpan(0, 16));
    internal static string BrowserTargetUrl(string projectId) => $"data:text/html,%3Ctitle%3Esuncode-browser-use-{ProjectHash(projectId)}%3C%2Ftitle%3E";
    internal static void Shutdown()
    {
        if (_configured) CefBrowserHost.Shutdown();
        Available = false;
    }

    private sealed class TrackedView(IBrowserView view, string? projectId) : IBrowserView
    {
        private int _disposed;
        public Avalonia.Controls.Control Control => view.Control;
        public string Address { get => view.Address; set => view.Address = value; }
        public bool CanGoBack => view.CanGoBack;
        public bool CanGoForward => view.CanGoForward;
        public bool IsLoading => view.IsLoading;
        public event Action? Initialized { add => view.Initialized += value; remove => view.Initialized -= value; }
        public event Action? NavigationChanged { add => view.NavigationChanged += value; remove => view.NavigationChanged -= value; }
        public event Action<Exception>? Failed { add => view.Failed += value; remove => view.Failed -= value; }
        public void GoBack() => view.GoBack();
        public void GoForward() => view.GoForward();
        public void Reload() => view.Reload();
        public void Dispose()
        {
            if (Interlocked.Exchange(ref _disposed, 1) != 0) return;
            try { view.Dispose(); }
            finally
            {
                Interlocked.Decrement(ref _activeViews);
                if (projectId is not null) lock (ProjectBrowsers)
                {
                    var count = ProjectBrowsers.GetValueOrDefault(projectId) - 1;
                    if (count <= 0) ProjectBrowsers.Remove(projectId);
                    else ProjectBrowsers[projectId] = count;
                }
            }
        }
    }
}
