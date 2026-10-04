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
    private ulong _reloadGeneration;
    private readonly DispatcherTimer _stateTimer = new() { Interval = TimeSpan.FromMilliseconds(700) };

    public event Action? CloseRequested;

    public BrowserPreviewPane() { InitializeComponent(); _stateTimer.Tick += PreviewStateTick; }

    public void CloseBrowser()
    {
        _stateTimer.Stop();
        if (DataContext is DesktopViewModel viewModel) _ = viewModel.StopPreviewAsync();
        BrowserHost.Child = null;
        BrowserHost.IsVisible = false;
        EmptyState.IsVisible = true;
        _browser?.Dispose();
        _browser = null;
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
        if (!CefPreviewRuntime.Available)
        {
            StatusText.Text = CefPreviewRuntime.Error ?? "Chromium unavailable";
            return;
        }

        try
        {
            if (_browser is null)
            {
                _browser = new AvaloniaCefBrowser(() => CefPreviewRuntime.CreateProjectContext(project.ProjectId))
                {
                    RequestHandler = new PreviewRequestHandler(),
                    LifeSpanHandler = new PreviewLifeSpanHandler()
                };
                // Windowless CEF defaults to 30 fps, which reads as dropped frames while
                // scrolling. Must be set before the browser is created on first layout.
                _browser.Settings.WindowlessFrameRate = 60;
                _browser.LoadEnd += (_, args) =>
                {
                    if (args.Frame.IsMain) Dispatcher.UIThread.Post(() => StatusText.Text = "Live");
                };
                BrowserHost.Child = _browser;
                BrowserHost.IsVisible = true;
                EmptyState.IsVisible = false;
            }
            if (DataContext is DesktopViewModel viewModel && !await viewModel.StartConfiguredPreviewAsync(AddressInput.Text!))
            {
                StatusText.Text = "Server failed";
                return;
            }
            StatusText.Text = "Loading";
            _browser.Address = AddressInput.Text;
            _stateTimer.Start();
        }
        catch (Exception exception)
        {
            DiagnosticLog.Error("browser.preview", exception, "open_failed=true");
            CloseBrowser();
            StatusText.Text = "Preview unavailable";
        }
    }

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
