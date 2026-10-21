using System.Collections.ObjectModel;
using System.Runtime.CompilerServices;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Threading;
using Avalonia.VisualTree;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.ViewModels;
using Xilium.CefGlue;
using Xilium.CefGlue.Avalonia;
using Xilium.CefGlue.Common.Handlers;

namespace SunCode.Desktop.Views.ProjectWorkspace.Preview;

public sealed class BrowserTab : ObservableObject
{
    private bool _isActive;

    public BrowserTab(string title, string address)
    {
        Title = title;
        Address = address;
    }

    public string Title { get; }
    public string Address { get; set; }
    public AvaloniaCefBrowser? Browser { get; set; }
    public bool IsBrowserUse { get; set; }
    public bool BrowserInitialized { get; set; }
    public bool IsActive
    {
        get => _isActive;
        set => SetProperty(ref _isActive, value);
    }
}

public sealed partial class BrowserPreviewPane : UserControl
{
    private const string DefaultAddress = "http://localhost:5173/";

    private readonly ObservableCollection<BrowserTab> _tabs = new();
    private BrowserTab? _activeTab;
    private BrowserTab? _browserUseTab;
    private bool _previewStarted;
    private TaskCompletionSource<bool>? _browserUseInitialization;
    private string? _browserUseProjectId;
    private bool _browserUseLayoutHooked;
    private ulong _reloadGeneration;
    private readonly DispatcherTimer _stateTimer = new() { Interval = TimeSpan.FromMilliseconds(700) };

    public BrowserPreviewPane()
    {
        InitializeComponent();
        BrowserTabs.ItemsSource = _tabs;
        _tabs.Add(new BrowserTab("Project preview", DefaultAddress));
        SetActiveTab(_tabs[0]);
        DisconnectButton.IsVisible = false;
        _stateTimer.Tick += PreviewStateTick;
    }

    public void CloseBrowser([CallerMemberName] string caller = "")
    {
        DiagnosticLog.Info("browser.host", $"close_all caller={caller}");
        _stateTimer.Stop();
        if (_previewStarted && DataContext is DesktopViewModel viewModel) _ = viewModel.StopPreviewAsync();
        _previewStarted = false;
        BrowserHost.Child = null;
        BrowserHost.IsVisible = false;
        EmptyState.IsVisible = true;
        DisconnectButton.IsVisible = false;

        foreach (var tab in _tabs) DisposeTab(tab);
        _tabs.Clear();
        _tabs.Add(new BrowserTab("Project preview", DefaultAddress));
        _browserUseTab = null;
        _activeTab = null;
        _browserUseProjectId = null;
        _browserUseInitialization?.TrySetResult(false);
        _browserUseInitialization = null;
        UnhookBrowserUseLayout();
        SetActiveTab(_tabs[0]);
        StatusText.Text = "Browser is offline";
        UpdateNavigationButtons();
    }

    private void NewTabClicked(object? sender, RoutedEventArgs e)
    {
        var tab = new BrowserTab("New tab", string.Empty);
        _tabs.Add(tab);
        SetActiveTab(tab);
        AddressInput.Focus();
    }

    private void TabClicked(object? sender, RoutedEventArgs e)
    {
        if ((sender as Button)?.Tag is BrowserTab tab) SetActiveTab(tab);
    }

    private void CloseTabClicked(object? sender, RoutedEventArgs e)
    {
        if ((sender as Button)?.Tag is not BrowserTab tab) return;
        if (_tabs.Count == 1)
        {
            DisconnectTab(tab);
            tab.Address = DefaultAddress;
            StatusText.Text = "Browser is offline";
            return;
        }

        var index = _tabs.IndexOf(tab);
        var next = _tabs.ElementAtOrDefault(Math.Min(index + 1, _tabs.Count - 1))
                   ?? _tabs.ElementAtOrDefault(Math.Max(0, index - 1));
        DisposeTab(tab);
        _tabs.Remove(tab);
        if (ReferenceEquals(_activeTab, tab)) SetActiveTab(next);
    }

    private void SetActiveTab(BrowserTab? tab)
    {
        if (tab is null) return;
        BrowserHost.Child = null;
        BrowserHost.IsVisible = false;
        EmptyState.IsVisible = true;

        if (_activeTab is not null) _activeTab.IsActive = false;
        _activeTab = tab;
        _activeTab.IsActive = true;
        AddressInput.Text = tab.Address;

        if (tab.Browser is not null)
        {
            BrowserHost.Child = tab.Browser;
            BrowserHost.IsVisible = true;
            EmptyState.IsVisible = false;
            DisconnectButton.IsVisible = true;
        }
        else
        {
            DisconnectButton.IsVisible = false;
        }

        UpdateNavigationButtons();
    }

    private void AddressKeyDown(object? sender, KeyEventArgs e)
    {
        if (e.Key != Key.Enter) return;
        _ = OpenPreviewAsync();
        e.Handled = true;
    }

    private async void OpenClicked(object? sender, RoutedEventArgs e) => await OpenPreviewAsync();

    private void BackClicked(object? sender, RoutedEventArgs e)
    {
        if (_activeTab?.Browser?.CanGoBack == true) _activeTab.Browser.GoBack();
        UpdateNavigationButtons();
    }

    private void ForwardClicked(object? sender, RoutedEventArgs e)
    {
        if (_activeTab?.Browser?.CanGoForward == true) _activeTab.Browser.GoForward();
        UpdateNavigationButtons();
    }

    private void ReloadClicked(object? sender, RoutedEventArgs e)
    {
        if (_activeTab?.Browser is not { } browser) return;
        browser.Reload();
        UpdateNavigationButtons();
    }

    private void DisconnectClicked(object? sender, RoutedEventArgs e)
    {
        if (_activeTab is not null) DisconnectTab(_activeTab);
    }

    private void DisconnectTab(BrowserTab tab)
    {
        if (ReferenceEquals(_activeTab, tab))
        {
            BrowserHost.Child = null;
            BrowserHost.IsVisible = false;
            EmptyState.IsVisible = true;
        }

        var wasPreview = tab.Browser is not null && !tab.IsBrowserUse;
        DisposeTab(tab);
        if (wasPreview && !_tabs.Any(item => item.Browser is not null && !item.IsBrowserUse))
        {
            _previewStarted = false;
            if (DataContext is DesktopViewModel viewModel) _ = viewModel.StopPreviewAsync();
            _stateTimer.Stop();
        }

        DisconnectButton.IsVisible = false;
        StatusText.Text = "Browser is offline";
        UpdateNavigationButtons();
    }

    private async Task OpenPreviewAsync()
    {
        if (DataContext is not DesktopViewModel { SelectedProject: { } project } viewModel || _activeTab is null) return;

        var address = AddressInput.Text?.Trim();
        if (address is null || !IsAllowedPreviewUrl(address))
        {
            StatusText.Text = "Enter a localhost HTTP URL";
            EmptyState.IsVisible = true;
            return;
        }

        _activeTab.Address = address;
        if (_activeTab.Browser is not null && _activeTab.BrowserInitialized)
        {
            _activeTab.Browser.Address = address;
            EmptyState.IsVisible = false;
            UpdateNavigationButtons();
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
            EnsureBrowser(_activeTab, project.ProjectId, false);
            if (!_previewStarted)
            {
                if (!await viewModel.StartConfiguredPreviewAsync(address))
                {
                    StatusText.Text = "Unable to connect to the project server";
                    return;
                }

                _previewStarted = true;
            }

            _activeTab.Browser!.Address = address;
            _stateTimer.Start();
            UpdateNavigationButtons();
        }
        catch (Exception exception)
        {
            DiagnosticLog.Error("browser.preview", exception, "open_failed=true");
            CloseBrowser();
            StatusText.Text = "Browser unavailable";
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

            var browserTab = _tabs.FirstOrDefault(tab => tab.IsBrowserUse);
            if (browserTab is null)
            {
                browserTab = _activeTab is { Browser: null } ? _activeTab : new BrowserTab("Browser Use", string.Empty);
                if (!_tabs.Contains(browserTab)) _tabs.Add(browserTab);
                browserTab.IsBrowserUse = true;
            }

            _browserUseTab = browserTab;
            SetActiveTab(browserTab);
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

    private void EnsureBrowser(BrowserTab tab, string projectId, bool browserUse)
    {
        if (tab.Browser is not null && tab.IsBrowserUse == browserUse)
        {
            if (ReferenceEquals(_activeTab, tab)) SetActiveTab(tab);
            return;
        }

        DisposeTab(tab);
        tab.IsBrowserUse = browserUse;
        tab.BrowserInitialized = false;
        DiagnosticLog.Info("browser.host", $"create_begin project={projectId} mode={(browserUse ? "browser_use" : "preview")} bounds={Bounds.Width:0.##}x{Bounds.Height:0.##}");

        var browser = new AvaloniaCefBrowser(() => CefPreviewRuntime.CreateProjectContext(projectId, browserUse));
        tab.Browser = browser;
        browser.BrowserInitialized += () =>
        {
            tab.BrowserInitialized = true;
            DiagnosticLog.Info("browser.host", $"browser_initialized project={projectId} bounds={Bounds.Width:0.##}x{Bounds.Height:0.##}");
            if (browserUse) _ = CompleteBrowserUseInitializationAsync();
            Dispatcher.UIThread.Post(UpdateNavigationButtons);
            Dispatcher.UIThread.Post(SyncConnectionState);
        };
        browser.LoadStart += (_, args) =>
        {
            if (args.Frame.IsMain) DiagnosticLog.Debug("browser.host", $"load_start project={projectId}");
        };
        browser.LoadError += (_, args) =>
        {
            if (args.Frame.IsMain) DiagnosticLog.Warn("browser.host", $"load_error project={projectId} code={args.ErrorCode} url_length={args.FailedUrl.Length}");
        };
        browser.UnhandledException += (_, args) => DiagnosticLog.Error("browser.host", args.Exception, $"unhandled=true project={projectId}");
        if (!browserUse)
        {
            browser.RequestHandler = new PreviewRequestHandler();
            browser.LifeSpanHandler = new PreviewLifeSpanHandler();
        }

        browser.Settings.WindowlessFrameRate = 60;
        browser.LoadEnd += (_, args) =>
        {
            if (args.Frame.IsMain) Dispatcher.UIThread.Post(UpdateNavigationButtons);
        };
        browser.AddressChanged += (_, _) => Dispatcher.UIThread.Post(UpdateNavigationButtons);
        browser.LoadingStateChange += (_, _) => Dispatcher.UIThread.Post(UpdateNavigationButtons);

        if (ReferenceEquals(_activeTab, tab))
        {
            BrowserHost.Child = browser;
            BrowserHost.IsVisible = true;
            EmptyState.IsVisible = false;
            DisconnectButton.IsVisible = true;
        }

        if (browserUse) browser.Address = CefPreviewRuntime.BrowserTargetUrl(projectId);
        DiagnosticLog.Debug("browser.host", $"create_requested project={projectId} host_bounds={BrowserHost.Bounds.Width:0.##}x{BrowserHost.Bounds.Height:0.##}");
    }

    private void DisposeTab(BrowserTab tab, [CallerMemberName] string caller = "")
    {
        if (tab.Browser is not null)
        {
            DiagnosticLog.Info("browser.host", $"browser_disposed mode={(tab.IsBrowserUse ? "browser_use" : "preview")} caller={caller}");
        }
        if (ReferenceEquals(BrowserHost.Child, tab.Browser)) BrowserHost.Child = null;
        tab.Browser?.Dispose();
        tab.Browser = null;
        tab.BrowserInitialized = false;
        SyncConnectionState();
    }
    
    private void SyncConnectionState()
    {
        if (DataContext is DesktopViewModel viewModel)
        {
            viewModel.IsBrowserConnected = _tabs.Any(item => item.Browser is not null && item.BrowserInitialized);
        }
    }

    private void TryCreateBrowserUsePage()
    {
        if (_browserUseInitialization is null || _browserUseProjectId is null || _browserUseTab is null) return;
        if (Bounds.Width <= 1 || Bounds.Height <= 1)
        {
            HookBrowserUseLayout();
            return;
        }

        UnhookBrowserUseLayout();
        try
        {
            EnsureBrowser(_browserUseTab, _browserUseProjectId, true);
            if (_browserUseTab.BrowserInitialized) _ = CompleteBrowserUseInitializationAsync();
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
        initialized.TrySetResult(_browserUseTab?.BrowserInitialized == true && _browserUseTab.Browser is not null);
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
        if (_activeTab?.Browser is { } browser && !_activeTab.IsBrowserUse &&
            viewModel.PreviewState is { ReloadGeneration: > 0 } state && state.ReloadGeneration != _reloadGeneration)
        {
            _reloadGeneration = state.ReloadGeneration;
            browser.Reload();
        }
        if (viewModel.PreviewState?.Status == "stopped")
        {
            StatusText.Text = "Browser is offline";
            _stateTimer.Stop();
        }

        UpdateNavigationButtons();
    }

    private void UpdateNavigationButtons()
    {
        var browser = _activeTab?.Browser;
        var tab = _activeTab;
        if (browser is null || tab is null || !tab.BrowserInitialized)
        {
            BackButton.IsEnabled = false;
            ForwardButton.IsEnabled = false;
            ReloadButton.IsEnabled = false;
            return;
        }

        if (!string.IsNullOrWhiteSpace(browser.Address) && AddressInput.Text != browser.Address)
        {
            AddressInput.Text = browser.Address;
            tab.Address = browser.Address;
        }

        BackButton.IsEnabled = browser.CanGoBack;
        ForwardButton.IsEnabled = browser.CanGoForward;
        ReloadButton.IsEnabled = !browser.IsLoading;
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
