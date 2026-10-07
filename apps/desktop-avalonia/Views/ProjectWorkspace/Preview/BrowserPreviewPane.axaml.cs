using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Threading;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.ViewModels;
using Xilium.CefGlue;
using Xilium.CefGlue.Avalonia;
using Xilium.CefGlue.Common.Handlers;

namespace SunCode.Desktop.Views.ProjectWorkspace.Preview;

public sealed partial class BrowserPreviewPane : UserControl
{
    private AvaloniaCefBrowser? _browser;
    private bool _browserUseMode;
    private volatile bool _browserInitialized;
    private TaskCompletionSource<bool>? _browserUseInitialization;
    private string? _browserUseProjectId;
    private bool _browserUseLayoutHooked;
    private ulong _reloadGeneration;
    private readonly DispatcherTimer _stateTimer = new() { Interval = TimeSpan.FromMilliseconds(700) };

    public event Action? CloseRequested;

    public BrowserPreviewPane() { InitializeComponent(); _stateTimer.Tick += PreviewStateTick; }

    public void CloseBrowser()
    {
        _stateTimer.Stop();
        if (!_browserUseMode && DataContext is DesktopViewModel viewModel) _ = viewModel.StopPreviewAsync();
        BrowserHost.Child = null;
        BrowserHost.IsVisible = false;
        EmptyState.IsVisible = true;
        _browser?.Dispose();
        _browser = null;
        _browserUseMode = false;
        _browserInitialized = false;
        _browserUseProjectId = null;
        _browserUseInitialization?.TrySetResult(false);
        _browserUseInitialization = null;
        UnhookBrowserUseLayout();
        StatusText.Text = "Stopped";
    }

    private void AddressKeyDown(object? sender, KeyEventArgs e)
    {
        if (e.Key != Key.Enter) return;
        _ = OpenPreviewAsync();
        e.Handled = true;
    }

    private async void OpenClicked(object? sender, RoutedEventArgs e) => await OpenPreviewAsync();

    private void ReloadClicked(object? sender, RoutedEventArgs e) => _browser?.Reload();

    private void CloseClicked(object? sender, RoutedEventArgs e) => CloseRequested?.Invoke();

    private async Task OpenPreviewAsync()
    {
        if (DataContext is not DesktopViewModel { SelectedProject: { } project }) return;
        if (!IsAllowedPreviewUrl(AddressInput.Text))
        {
            StatusText.Text = "Enter a localhost HTTP URL";
            return;
        }
        CefPreviewRuntime.Initialize();
        if (!CefPreviewRuntime.Available)
        {
            StatusText.Text = CefPreviewRuntime.Error ?? "Chromium unavailable";
            return;
        }

        try
        {
            EnsureBrowser(project.ProjectId, false);
            if (DataContext is DesktopViewModel viewModel && !await viewModel.StartConfiguredPreviewAsync(AddressInput.Text!))
            {
                StatusText.Text = "Server failed";
                return;
            }
            StatusText.Text = "Loading";
            (_browser ?? throw new InvalidOperationException("Preview browser was not created.")).Address = AddressInput.Text;
            _stateTimer.Start();
        }
        catch (Exception exception)
        {
            DiagnosticLog.Error("browser.preview", exception, "open_failed=true");
            CloseBrowser();
            StatusText.Text = "Preview unavailable";
        }
    }

    internal void EnsureBrowserUsePage(TaskCompletionSource<bool> initialized)
    {
        if (DataContext is not DesktopViewModel { SelectedProject: { } project })
        {
            initialized.TrySetResult(false);
            return;
        }
        try
        {
            CefPreviewRuntime.Initialize();
            if (!CefPreviewRuntime.Available)
            {
                initialized.TrySetResult(false);
                return;
            }
            DiagnosticLog.Info("browser.host", $"ensure_page project={project.ProjectId} bounds={Bounds.Width:0.##}x{Bounds.Height:0.##} attached={VisualRoot is not null}");
            _browserUseInitialization?.TrySetResult(false);
            _browserUseInitialization = initialized;
            _browserUseProjectId = project.ProjectId;
            TryCreateBrowserUsePage();
        }
        catch (Exception exception)
        {
            DiagnosticLog.Error("browser.host", exception, "page_create_failed=true");
            CloseBrowser();
            initialized.TrySetResult(false);
        }
    }

    private void EnsureBrowser(string projectId, bool browserUse)
    {
        if (_browser is not null && _browserUseMode == browserUse) return;
        if (_browser is not null)
        {
            BrowserHost.Child = null;
            _browser.Dispose();
            _browser = null;
        }
        _browserUseMode = browserUse;
        _browserInitialized = false;
        DiagnosticLog.Info("browser.host", $"create_begin project={projectId} mode={(browserUse ? "browser_use" : "preview")} bounds={Bounds.Width:0.##}x{Bounds.Height:0.##}");
        _browser = new AvaloniaCefBrowser(() => CefPreviewRuntime.CreateProjectContext(projectId, browserUse));
        _browser.BrowserInitialized += () =>
        {
            _browserInitialized = true;
            DiagnosticLog.Info("browser.host", $"browser_initialized project={projectId} bounds={Bounds.Width:0.##}x{Bounds.Height:0.##}");
            if (browserUse) _ = CompleteBrowserUseInitializationAsync();
        };
        _browser.LoadStart += (_, args) =>
        {
            if (args.Frame.IsMain) DiagnosticLog.Debug("browser.host", $"load_start project={projectId}");
        };
        _browser.LoadError += (_, args) =>
        {
            if (args.Frame.IsMain) DiagnosticLog.Warn("browser.host", $"load_error project={projectId} code={args.ErrorCode} url_length={args.FailedUrl.Length}");
        };
        _browser.UnhandledException += (_, args) => DiagnosticLog.Error("browser.host", args.Exception, $"unhandled=true project={projectId}");
        if (!browserUse)
        {
            _browser.RequestHandler = new PreviewRequestHandler();
            _browser.LifeSpanHandler = new PreviewLifeSpanHandler();
        }
        _browser.Settings.WindowlessFrameRate = 60;
        _browser.LoadEnd += (_, args) =>
        {
            if (args.Frame.IsMain) Dispatcher.UIThread.Post(() => StatusText.Text = browserUse ? "Browser ready" : "Live");
        };
        BrowserHost.Child = _browser;
        BrowserHost.IsVisible = true;
        EmptyState.IsVisible = false;
        if (browserUse) _browser.Address = CefPreviewRuntime.BrowserTargetUrl(projectId);
        DiagnosticLog.Debug("browser.host", $"create_requested project={projectId} host_bounds={BrowserHost.Bounds.Width:0.##}x{BrowserHost.Bounds.Height:0.##}");
    }

    private void TryCreateBrowserUsePage()
    {
        if (_browserUseInitialization is null || _browserUseProjectId is null) return;
        // BrowserHost has no measurable size while it is empty. Measure the
        // pane first, then attach the browser; the child will receive the
        // available host bounds during the following layout pass.
        if (Bounds.Width <= 1 || Bounds.Height <= 1)
        {
            DiagnosticLog.Debug("browser.host", $"waiting_for_layout project={_browserUseProjectId} bounds={Bounds.Width:0.##}x{Bounds.Height:0.##} host_bounds={BrowserHost.Bounds.Width:0.##}x{BrowserHost.Bounds.Height:0.##}");
            HookBrowserUseLayout();
            return;
        }
        UnhookBrowserUseLayout();
        try
        {
            EnsureBrowser(_browserUseProjectId, true);
            StatusText.Text = "Browser ready";
            if (_browserInitialized) _ = CompleteBrowserUseInitializationAsync();
        }
        catch (Exception exception)
        {
            DiagnosticLog.Error("browser.host", exception, "page_create_failed=true");
            _browserUseInitialization?.TrySetResult(false);
            _browserUseInitialization = null;
            CloseBrowser();
        }
    }

    private async Task CompleteBrowserUseInitializationAsync()
    {
        var initialized = _browserUseInitialization;
        if (initialized is null) return;
        await Task.Delay(500);
        initialized.TrySetResult(_browserInitialized && _browser is not null);
        if (ReferenceEquals(_browserUseInitialization, initialized)) _browserUseInitialization = null;
    }

    private void HookBrowserUseLayout()
    {
        if (_browserUseLayoutHooked) return;
        _browserUseLayoutHooked = true;
        LayoutUpdated += BrowserUseLayoutUpdated;
        AttachedToVisualTree += BrowserUseAttached;
    }

    private void UnhookBrowserUseLayout()
    {
        if (!_browserUseLayoutHooked) return;
        _browserUseLayoutHooked = false;
        LayoutUpdated -= BrowserUseLayoutUpdated;
        AttachedToVisualTree -= BrowserUseAttached;
    }

    private void BrowserUseLayoutUpdated(object? sender, EventArgs e) => TryCreateBrowserUsePage();
    private void BrowserUseAttached(object? sender, VisualTreeAttachmentEventArgs e) => TryCreateBrowserUsePage();

    private async void PreviewStateTick(object? sender, EventArgs e)
    {
        if (DataContext is not DesktopViewModel viewModel || viewModel.SelectedProject is null) return;
        await viewModel.RefreshPreviewStateAsync();
        if (viewModel.PreviewState is { ReloadGeneration: > 0 } state && state.ReloadGeneration != _reloadGeneration)
        {
            _reloadGeneration = state.ReloadGeneration;
            _browser?.Reload();
        }
        if (viewModel.PreviewState?.Status == "stopped") { StatusText.Text = "Stopped"; _stateTimer.Stop(); }
    }

    internal static bool IsAllowedPreviewUrl(string? value) =>
        Uri.TryCreate(value, UriKind.Absolute, out var uri)
        && uri.Scheme == Uri.UriSchemeHttp
        && (uri.Host.Equals("localhost", StringComparison.OrdinalIgnoreCase) || uri.Host == "127.0.0.1")
        && uri.UserInfo.Length == 0;

    private sealed class PreviewRequestHandler : RequestHandler
    {
        protected override bool OnBeforeBrowse(CefBrowser browser, CefFrame frame, CefRequest request, bool userGesture, bool isRedirect) =>
            frame.IsMain && request.Url != "about:blank" && !IsAllowedPreviewUrl(request.Url);
    }

    private sealed class PreviewLifeSpanHandler : LifeSpanHandler
    {
        protected override bool OnBeforePopup(
            CefBrowser browser, CefFrame frame, int popupId, string targetUrl, string targetFrameName,
            CefWindowOpenDisposition targetDisposition, bool userGesture, CefPopupFeatures popupFeatures,
            CefWindowInfo windowInfo, ref CefClient client, CefBrowserSettings settings,
            ref CefDictionaryValue extraInfo, ref bool noJavascriptAccess) => true;
    }
}
