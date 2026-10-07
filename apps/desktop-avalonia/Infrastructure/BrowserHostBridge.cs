using Avalonia.Threading;
using SunCode.Desktop.Models;

namespace SunCode.Desktop.Infrastructure;

/// <summary>UI-thread operations the browser host bridge needs from the window layer.</summary>
internal interface IBrowserHostWindows
{
    /// <summary>
    /// UI thread. Completes true once the project's window is registered, or false when
    /// an open attempt finished without producing one. May never complete if the project
    /// is unknown and nobody opens it; the bridge applies the timeout.
    /// </summary>
    Task<bool> EnsureProjectWindowAsync(string projectId);

    /// <summary>UI thread. Starts the Browser Use page for the project and completes <paramref name="initialized"/>.</summary>
    void EnsureBrowserUsePage(string projectId, TaskCompletionSource<bool> initialized);
}

/// <summary>
/// Answers the Rust SDK's synchronous browser-host callback
/// (<c>AgentSdk.BrowserHostRequested</c>, a native <c>Func&lt;string, bool&gt;</c>).
/// The callback runs on an SDK thread and must return a result, so this blocks that
/// thread on signals completed by the UI thread. It never polls and never blocks the
/// UI thread; calling it on the UI thread fails fast instead of deadlocking.
/// </summary>
internal sealed class BrowserHostBridge
{
    // Matches the previous 120 polls x 125 ms window wait.
    internal static readonly TimeSpan DefaultWindowTimeout = TimeSpan.FromSeconds(15);
    internal static readonly TimeSpan DefaultPageTimeout = TimeSpan.FromSeconds(15);

    private readonly IBrowserHostWindows _windows;
    private readonly Func<bool> _isUiThread;
    private readonly Action<Action> _postToUi;
    private readonly TimeSpan _windowTimeout;
    private readonly TimeSpan _pageTimeout;

    public BrowserHostBridge(IBrowserHostWindows windows)
        : this(windows, Dispatcher.UIThread.CheckAccess, action => Dispatcher.UIThread.Post(action), DefaultWindowTimeout, DefaultPageTimeout)
    {
    }

    internal BrowserHostBridge(
        IBrowserHostWindows windows,
        Func<bool> isUiThread,
        Action<Action> postToUi,
        TimeSpan windowTimeout,
        TimeSpan pageTimeout)
    {
        _windows = windows;
        _isUiThread = isUiThread;
        _postToUi = postToUi;
        _windowTimeout = windowTimeout;
        _pageTimeout = pageTimeout;
    }

    public bool EnsureBrowserHost(string projectId)
    {
        if (_isUiThread())
        {
            // Blocking here would wait on work queued behind this very call.
            DiagnosticLog.Error("browser.host", $"ui_thread_call=true rejected=true project={projectId}");
            return false;
        }
        try
        {
            var windowReady = new TaskCompletionSource<bool>(TaskCreationOptions.RunContinuationsAsynchronously);
            _postToUi(() => _ = SignalWindowReadyAsync(projectId, windowReady));
            if (!windowReady.Task.Wait(_windowTimeout) || !windowReady.Task.Result)
            {
                DiagnosticLog.Warn("browser.host", $"project_window_missing=true project={projectId}");
                return false;
            }

            var initialized = new TaskCompletionSource<bool>(TaskCreationOptions.RunContinuationsAsynchronously);
            _postToUi(() =>
            {
                try { _windows.EnsureBrowserUsePage(projectId, initialized); }
                catch (Exception exception) { initialized.TrySetException(exception); }
            });
            return initialized.Task.Wait(_pageTimeout) && initialized.Task.Result;
        }
        catch (Exception exception)
        {
            DiagnosticLog.Error("browser.host", exception, $"project={projectId}");
            return false;
        }
    }

    private async Task SignalWindowReadyAsync(string projectId, TaskCompletionSource<bool> windowReady)
    {
        try
        {
            windowReady.TrySetResult(await _windows.EnsureProjectWindowAsync(projectId));
        }
        catch (Exception exception)
        {
            DiagnosticLog.Error("browser.host", exception.GetBaseException(), $"project_open_failed project={projectId}");
            windowReady.TrySetResult(false);
        }
    }
}

/// <summary>Production <see cref="IBrowserHostWindows"/> over <see cref="ProjectWindowManager"/>.</summary>
internal sealed class ProjectWindowBrowserHost : IBrowserHostWindows
{
    private readonly ProjectWindowManager _windows;
    private readonly Func<string, ProjectItem?> _findProject;

    public ProjectWindowBrowserHost(ProjectWindowManager windows, Func<string, ProjectItem?> findProject)
    {
        _windows = windows;
        _findProject = findProject;
    }

    public async Task<bool> EnsureProjectWindowAsync(string projectId)
    {
        var registered = _windows.WaitForWindowAsync(projectId);
        if (registered.IsCompleted) return true;
        if (_findProject(projectId) is not { } project)
        {
            // Another path may still open it; the bridge timeout bounds the wait.
            DiagnosticLog.Warn("browser.host", $"project_unknown=true project={projectId}");
            await registered;
            return true;
        }
        var open = _windows.OpenProjectWindowAsync(project);
        if (await Task.WhenAny(registered, open) == registered) return true;
        await open; // Surfaces open failures to the bridge.
        return _windows.TryGetWindow(projectId, out _);
    }

    public void EnsureBrowserUsePage(string projectId, TaskCompletionSource<bool> initialized)
    {
        if (_windows.TryGetWindow(projectId, out var window)) window.Workspace.EnsureBrowserUsePage(initialized);
        else initialized.TrySetResult(false);
    }
}
