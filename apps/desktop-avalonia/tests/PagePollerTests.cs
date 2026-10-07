using SunCode.Desktop.Infrastructure;

namespace SunCode.Desktop.Tests;

public sealed class PagePollerTests
{
    [Fact]
    public void ActivateLoadsOnceAndTicksPollTheActivePage()
    {
        var timer = new FakePollTimer();
        using var poller = new PagePoller(timer);
        var page = new RecordingPage();

        poller.Activate(page);
        timer.Fire();
        timer.Fire();

        Assert.Equal(1, page.Activations);
        Assert.Equal(2, page.Polls);
    }

    [Fact]
    public void SwitchingPagesDeactivatesThePreviousPage()
    {
        var timer = new FakePollTimer();
        using var poller = new PagePoller(timer);
        var first = new RecordingPage();
        var second = new RecordingPage();

        poller.Activate(first);
        poller.Activate(second);
        timer.Fire();

        Assert.Equal(1, first.Deactivations);
        Assert.Equal(0, first.Polls);
        Assert.Equal(1, second.Polls);
    }

    [Fact]
    public void ReselectingTheActivePageReloadsWithoutDeactivating()
    {
        using var poller = new PagePoller(new FakePollTimer());
        var page = new RecordingPage();

        poller.Activate(page);
        poller.Activate(page);

        Assert.Equal(2, page.Activations);
        Assert.Equal(0, page.Deactivations);
    }

    [Fact]
    public void TickIsSkippedWhileThePreviousPollRuns()
    {
        var timer = new FakePollTimer();
        using var poller = new PagePoller(timer);
        var gate = new TaskCompletionSource();
        var page = new RecordingPage { PollTask = gate.Task };

        poller.Activate(page);
        timer.Fire();
        timer.Fire();
        gate.SetResult();
        timer.Fire();

        Assert.Equal(2, page.Polls);
    }

    [Fact]
    public void DisposeStopsTimerAndIgnoresLaterActivation()
    {
        var timer = new FakePollTimer();
        var poller = new PagePoller(timer);
        var page = new RecordingPage();
        poller.Activate(page);

        poller.Dispose();
        poller.Activate(new RecordingPage());
        timer.Fire();

        Assert.False(timer.Running);
        Assert.Equal(1, page.Deactivations);
        Assert.Equal(0, page.Polls);
        Assert.Null(poller.Active);
    }
}

internal sealed class FakePollTimer : IPollTimer
{
    public event EventHandler? Tick;
    public bool Running { get; private set; }
    public void Start() => Running = true;
    public void Stop() => Running = false;
    public void Fire() => Tick?.Invoke(this, EventArgs.Empty);
}

internal sealed class RecordingPage : IPolledPage
{
    public int Activations { get; private set; }
    public int Polls { get; private set; }
    public int Deactivations { get; private set; }
    public Task PollTask { get; set; } = Task.CompletedTask;

    public Task ActivateAsync()
    {
        Activations++;
        return Task.CompletedTask;
    }

    public Task PollAsync()
    {
        Polls++;
        return PollTask;
    }

    public void Deactivate() => Deactivations++;
}
