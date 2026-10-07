using Avalonia.Threading;

namespace SunCode.Desktop.Infrastructure;

// A settings page whose data refreshes while it is the visible page.
internal interface IPolledPage
{
    // Runs once when the page becomes visible.
    Task ActivateAsync();

    // Runs on every poll interval while the page stays visible.
    Task PollAsync();

    // Runs when another page replaces this one or the window closes.
    void Deactivate();
}

internal interface IPollTimer
{
    event EventHandler? Tick;
    void Start();
    void Stop();
}

internal sealed class DispatcherPollTimer(TimeSpan interval) : IPollTimer
{
    private readonly DispatcherTimer _timer = new() { Interval = interval };

    public event EventHandler? Tick
    {
        add => _timer.Tick += value;
        remove => _timer.Tick -= value;
    }

    public void Start() => _timer.Start();
    public void Stop() => _timer.Stop();
}

// One shared timer for every polled settings page. Only the active page polls,
// and a tick is skipped while the previous poll is still running.
internal sealed class PagePoller : IDisposable
{
    public static readonly TimeSpan DefaultInterval = TimeSpan.FromSeconds(2);

    private readonly IPollTimer _timer;
    private IPolledPage? _active;
    private bool _polling;
    private bool _disposed;

    public PagePoller(IPollTimer? timer = null)
    {
        _timer = timer ?? new DispatcherPollTimer(DefaultInterval);
        _timer.Tick += OnTick;
    }

    public IPolledPage? Active => _active;

    // Makes the page the active poll target and runs its activation load,
    // also when it is already active. Passing null stops polling.
    public void Activate(IPolledPage? page)
    {
        if (_disposed) return;
        if (!ReferenceEquals(page, _active)) Deactivate();
        if (page is null) return;
        _active = page;
        _ = RunAsync(page.ActivateAsync);
        _timer.Start();
    }

    public void Deactivate()
    {
        _timer.Stop();
        var previous = _active;
        _active = null;
        previous?.Deactivate();
    }

    private async void OnTick(object? sender, EventArgs e)
    {
        if (_polling || _active is not { } page) return;
        _polling = true;
        try
        {
            await RunAsync(page.PollAsync);
        }
        finally
        {
            _polling = false;
        }
    }

    private static async Task RunAsync(Func<Task> operation)
    {
        try
        {
            await operation();
        }
        catch (Exception exception)
        {
            DiagnosticLog.Error("settings.poll", exception, "operation=poll");
        }
    }

    public void Dispose()
    {
        if (_disposed) return;
        Deactivate();
        _timer.Tick -= OnTick;
        _disposed = true;
    }
}
