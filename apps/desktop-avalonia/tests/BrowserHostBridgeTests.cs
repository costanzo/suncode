using System.Diagnostics;
using SunCode.Desktop.Infrastructure;

namespace SunCode.Desktop.Tests;

public sealed class BrowserHostBridgeTests
{
    private sealed class FakeHost : IBrowserHostWindows
    {
        public TaskCompletionSource<bool> Window { get; } = new(TaskCreationOptions.RunContinuationsAsynchronously);
        public bool? PageResult { get; init; } = true;
        public int WindowRequests;
        public int PageRequests;

        public Task<bool> EnsureProjectWindowAsync(string projectId)
        {
            Interlocked.Increment(ref WindowRequests);
            return Window.Task;
        }

        public void EnsureBrowserUsePage(string projectId, TaskCompletionSource<bool> initialized)
        {
            Interlocked.Increment(ref PageRequests);
            if (PageResult is { } result) initialized.TrySetResult(result);
        }
    }

    // Runs "UI" work on the thread pool, standing in for the dispatcher.
    private static BrowserHostBridge Bridge(FakeHost host, bool onUiThread = false, double windowSeconds = 5, double pageSeconds = 5) =>
        new(host, () => onUiThread, action => ThreadPool.QueueUserWorkItem(_ => action()),
            TimeSpan.FromSeconds(windowSeconds), TimeSpan.FromSeconds(pageSeconds));

    [Fact]
    public void ReturnsAsSoonAsTheWindowIsSignalled()
    {
        var host = new FakeHost();
        var bridge = Bridge(host);
        _ = Task.Delay(100).ContinueWith(_ => host.Window.TrySetResult(true));

        var stopwatch = Stopwatch.StartNew();
        Assert.True(bridge.EnsureBrowserHost("p"));
        Assert.True(stopwatch.Elapsed < TimeSpan.FromSeconds(4));
        Assert.Equal(1, host.WindowRequests);
        Assert.Equal(1, host.PageRequests);
    }

    [Fact]
    public void RejectsUiThreadCallsWithoutTouchingTheHost()
    {
        var host = new FakeHost();
        Assert.False(Bridge(host, onUiThread: true).EnsureBrowserHost("p"));
        Assert.Equal(0, host.WindowRequests);
    }

    [Fact]
    public void FailsWhenTheWindowNeverAppears()
    {
        var host = new FakeHost();
        Assert.False(Bridge(host, windowSeconds: 0.2).EnsureBrowserHost("p"));
        Assert.Equal(0, host.PageRequests);
    }

    [Fact]
    public void FailsWhenOpeningProducesNoWindow()
    {
        var host = new FakeHost();
        host.Window.TrySetResult(false);
        Assert.False(Bridge(host).EnsureBrowserHost("p"));
        Assert.Equal(0, host.PageRequests);
    }

    [Fact]
    public void FailsWhenOpeningThrows()
    {
        var host = new FakeHost();
        host.Window.TrySetException(new InvalidOperationException("boom"));
        Assert.False(Bridge(host).EnsureBrowserHost("p"));
    }

    [Theory]
    [InlineData(false)]
    [InlineData(null)]
    public void ReportsPageFailureOrTimeout(bool? pageResult)
    {
        var host = new FakeHost { PageResult = pageResult };
        host.Window.TrySetResult(true);
        Assert.False(Bridge(host, pageSeconds: 0.2).EnsureBrowserHost("p"));
        Assert.Equal(1, host.PageRequests);
    }
}
