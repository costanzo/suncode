using System.Runtime.InteropServices;
using Xilium.CefGlue;
using Xilium.CefGlue.Avalonia;
using Xilium.CefGlue.BrowserProcess;
using Xilium.CefGlue.Common;
using Xilium.CefGlue.Common.Handlers;

namespace SunCode.Desktop.Infrastructure;

internal static class CefBrowserHost
{
    private static bool _resolverInstalled;
    private static string? _directory;

    private static void Load(string directory)
    {
        if (_directory is not null && _directory != directory)
            throw new InvalidOperationException("Restart SunCode before switching Chromium versions.");
        _directory = directory;
        if (!_resolverInstalled)
        {
            NativeLibrary.SetDllImportResolver(typeof(CefRuntime).Assembly, (name, _, _) =>
                name is "libcef" or "cef" ? NativeLibrary.Load(Path.Combine(directory, OperatingSystem.IsWindows() ? "libcef.dll" : "libcef.so")) : IntPtr.Zero);
            _resolverInstalled = true;
        }
        CefRuntime.Load(OperatingSystem.IsWindows() ? directory : null);
    }

    internal static void Initialize(string runtimeDirectory, string cacheRoot)
    {
        Load(runtimeDirectory);
        Directory.CreateDirectory(cacheRoot);
        CefRuntimeLoader.Initialize(new CefSettings
        {
            RootCachePath = cacheRoot,
            CachePath = cacheRoot,
            RemoteDebuggingPort = CefPreviewRuntime.RemoteDebuggingPort,
            BrowserSubprocessPath = CefSubProcess.GetSubProcessPath(),
            ResourcesDirPath = runtimeDirectory,
            LocalesDirPath = Path.Combine(runtimeDirectory, "locales"),
            LogSeverity = CefLogSeverity.Error,
        });
    }

    internal static void RunSubprocess(string runtimeDirectory, string[] arguments)
    {
        Load(runtimeDirectory);
        CefSubProcess.Run(arguments, true);
    }

    internal static IBrowserView CreateView(string profilePath, bool localPreview) => new CefView(profilePath, localPreview);
    internal static void Shutdown() { if (CefRuntime.IsInitialized) CefRuntime.Shutdown(); }

    private sealed class CefView : IBrowserView
    {
        private readonly AvaloniaCefBrowser _browser;
        private CefRequestContext? _context;
        public CefView(string profilePath, bool localPreview)
        {
            Directory.CreateDirectory(profilePath);
            _browser = new AvaloniaCefBrowser(() =>
            {
                _context = CefRequestContext.CreateContext(new CefRequestContextSettings { CachePath = profilePath, PersistSessionCookies = true }, null);
                return _context;
            });
            _browser.BrowserInitialized += () => Initialized?.Invoke();
            _browser.AddressChanged += (_, _) => NavigationChanged?.Invoke();
            _browser.LoadingStateChange += (_, _) => NavigationChanged?.Invoke();
            _browser.LoadEnd += (_, args) => { if (args.Frame.IsMain) NavigationChanged?.Invoke(); };
            _browser.UnhandledException += (_, args) => Failed?.Invoke(args.Exception);
            _browser.LifeSpanHandler = new BlockPopups();
            if (localPreview) _browser.RequestHandler = new LocalPreviewRequests();
            _browser.Settings.WindowlessFrameRate = 60;
        }
        public Avalonia.Controls.Control Control => _browser;
        public string Address { get => _browser.Address; set => _browser.Address = value; }
        public bool CanGoBack => _browser.CanGoBack;
        public bool CanGoForward => _browser.CanGoForward;
        public bool IsLoading => _browser.IsLoading;
        public event Action? Initialized;
        public event Action? NavigationChanged;
        public event Action<Exception>? Failed;
        public void GoBack() => _browser.GoBack();
        public void GoForward() => _browser.GoForward();
        public void Reload() => _browser.Reload();
        public void Dispose() { _browser.Dispose(); _context?.Dispose(); }
    }

    private sealed class LocalPreviewRequests : RequestHandler
    {
        protected override bool OnBeforeBrowse(CefBrowser browser, CefFrame frame, CefRequest request, bool userGesture, bool isRedirect) =>
            frame.IsMain && request.Url != "about:blank" && !(Uri.TryCreate(request.Url, UriKind.Absolute, out var uri)
                && uri.Scheme == "http" && (uri.Host is "localhost" or "127.0.0.1") && uri.UserInfo.Length == 0);
    }
    private sealed class BlockPopups : LifeSpanHandler
    {
        protected override bool OnBeforePopup(CefBrowser browser, CefFrame frame, int popupId, string targetUrl, string targetFrameName,
            CefWindowOpenDisposition targetDisposition, bool userGesture, CefPopupFeatures popupFeatures,
            CefWindowInfo windowInfo, ref CefClient client, CefBrowserSettings settings, ref CefDictionaryValue extraInfo, ref bool noJavascriptAccess) => true;
    }
}
