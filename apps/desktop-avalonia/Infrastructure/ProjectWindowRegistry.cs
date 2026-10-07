namespace SunCode.Desktop.Infrastructure;

internal enum ProjectWindowDisposition
{
    OpenNew,
    ActivateExisting,
    AwaitOpening
}

/// <summary>
/// UI-free bookkeeping for project windows: which projects have a window, which are
/// still opening, and who is waiting for a window to become available. All members
/// must be called from one thread (the UI thread in production). Returned tasks may be
/// awaited or waited on from any thread; their continuations run asynchronously.
/// </summary>
internal sealed class ProjectWindowRegistry<TWindow> where TWindow : class
{
    private readonly Dictionary<string, TWindow> _windows = [];
    private readonly Dictionary<string, TaskCompletionSource<bool>> _opening = [];
    private readonly Dictionary<string, TaskCompletionSource<TWindow>> _windowWaiters = [];

    public int Count => _windows.Count;

    public IReadOnlyCollection<TWindow> Windows => _windows.Values;

    public KeyValuePair<string, TWindow>[] Snapshot() => _windows.ToArray();

    public bool IsOpen(string projectId) => _windows.ContainsKey(projectId);

    public bool IsOpening(string projectId) => _opening.ContainsKey(projectId);

    public bool TryGet(string projectId, out TWindow window)
    {
        if (_windows.TryGetValue(projectId, out var found))
        {
            window = found;
            return true;
        }
        window = null!;
        return false;
    }

    public ProjectWindowDisposition Resolve(string projectId) =>
        ResolveDisposition(IsOpen(projectId), IsOpening(projectId));

    internal static ProjectWindowDisposition ResolveDisposition(bool isOpen, bool isOpening) =>
        isOpen
            ? ProjectWindowDisposition.ActivateExisting
            : isOpening ? ProjectWindowDisposition.AwaitOpening : ProjectWindowDisposition.OpenNew;

    /// <summary>Marks the project as opening. Returns false if it already is.</summary>
    public bool TryBeginOpening(string projectId)
    {
        if (_opening.ContainsKey(projectId)) return false;
        _opening[projectId] = new TaskCompletionSource<bool>(TaskCreationOptions.RunContinuationsAsynchronously);
        return true;
    }

    /// <summary>
    /// Completes when an in-progress open finishes, with whether a window is registered.
    /// Completes immediately when nothing is opening.
    /// </summary>
    public Task<bool> WhenOpened(string projectId) =>
        _opening.TryGetValue(projectId, out var signal) ? signal.Task : Task.FromResult(IsOpen(projectId));

    public void EndOpening(string projectId)
    {
        if (_opening.Remove(projectId, out var signal)) signal.TrySetResult(IsOpen(projectId));
    }

    /// <summary>Registers the project's window and releases anyone waiting for it.</summary>
    public void Register(string projectId, TWindow window)
    {
        _windows[projectId] = window;
        if (_windowWaiters.Remove(projectId, out var waiter)) waiter.TrySetResult(window);
    }

    public bool Remove(string projectId) => _windows.Remove(projectId);

    /// <summary>
    /// Completes with the project's window as soon as one is registered. Waiters share
    /// one signal per project and are not cancelled; callers apply their own timeout.
    /// </summary>
    public Task<TWindow> WaitForWindowAsync(string projectId)
    {
        if (_windows.TryGetValue(projectId, out var window)) return Task.FromResult(window);
        if (!_windowWaiters.TryGetValue(projectId, out var waiter))
        {
            waiter = new TaskCompletionSource<TWindow>(TaskCreationOptions.RunContinuationsAsynchronously);
            _windowWaiters[projectId] = waiter;
        }
        return waiter.Task;
    }
}
