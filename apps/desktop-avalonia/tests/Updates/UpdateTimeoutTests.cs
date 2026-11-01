using System.Net;
using Microsoft.Extensions.Time.Testing;
using SunCode.Desktop.Infrastructure.Updates;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Tests.Updates;

public sealed class UpdateTimeoutTests
{
    [Theory]
    [InlineData(408)]
    [InlineData(504)]
    public async Task ServerTimeoutStatusesUseTheSameTypedFailures(int status)
    {
        using var f = new UpdateFixture();
        using var client = new UpdateTransport(f.Distribution, f.Store, f.Environment, f.Handler, f.Time);
        f.Handler.Respond = (_, _) => Task.FromResult(new HttpResponseMessage((HttpStatusCode)status));
        Assert.Equal("check_timed_out", (await Assert.ThrowsAsync<UpdateException>(() => client.CheckAsync(default))).Code);
        Assert.Equal("download_timed_out", (await Assert.ThrowsAsync<UpdateException>(() => client.DownloadAsync(
            f.Catalog.Components.Sdk.Package, Path.Combine(f.Root, "archive.zip"), null, default))).Code);
    }
    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task CatalogHeaderAndBodyTimeoutsFailWithoutOverwritingAuthenticatedCache(bool body)
    {
        using var f = new UpdateFixture();
        var clock = new FakeTimeProvider(f.Time.Now);
        using var client = new UpdateTransport(f.Distribution, f.Store, f.Environment, f.Handler, clock);
        var previous = await client.CheckAsync(default);
        var waiting = Signal();
        f.Handler.Respond = body
            ? (_, _) => Task.FromResult(JsonStream(new ControlledStream(async (_, token) => { waiting.TrySetResult(); await Task.Delay(Timeout.Infinite, token); return 0; })))
            : async (_, token) => { waiting.TrySetResult(); await Task.Delay(Timeout.Infinite, token); return new(HttpStatusCode.OK); };
        var check = client.CheckAsync(default);
        await waiting.Task.WaitAsync(TimeSpan.FromSeconds(5));
        clock.Advance(UpdateTransport.RequestTimeout);
        Assert.Equal("check_timed_out", (await Assert.ThrowsAsync<UpdateException>(() => check).WaitAsync(TimeSpan.FromSeconds(5))).Code);
        Assert.Equal(previous.SignedSha256, f.Store.Cached(client.CatalogUrl)!.SignedSha256);
        Assert.Equal(2, f.Handler.Requests); // Timeout does not add an automatic retry.
    }

    [Fact]
    public async Task CatalogCallerCancellationIsNotReportedAsTimeout()
    {
        using var f = new UpdateFixture();
        var clock = new FakeTimeProvider(f.Time.Now);
        using var client = new UpdateTransport(f.Distribution, f.Store, f.Environment, f.Handler, clock);
        using var cancel = new CancellationTokenSource();
        var waiting = Signal();
        f.Handler.Respond = async (_, token) => { waiting.TrySetResult(); await Task.Delay(Timeout.Infinite, token); return new(HttpStatusCode.OK); };
        var check = client.CheckAsync(cancel.Token);
        await waiting.Task.WaitAsync(TimeSpan.FromSeconds(5));
        cancel.Cancel(); clock.Advance(UpdateTransport.RequestTimeout);
        await Assert.ThrowsAnyAsync<OperationCanceledException>(() => check).WaitAsync(TimeSpan.FromSeconds(5));
    }

    [Theory]
    [InlineData("headers", "download_timed_out")]
    [InlineData("first-byte", "download_stalled")]
    [InlineData("mid-stream", "download_stalled")]
    public async Task StalledArchiveTransfersFailAndCleanPartialFiles(string stage, string expected)
    {
        using var f = new UpdateFixture();
        var clock = new FakeTimeProvider(f.Time.Now);
        using var client = new UpdateTransport(f.Distribution, f.Store, f.Environment, f.Handler, clock);
        var waiting = Signal(); var reads = 0;
        f.Handler.Respond = stage == "headers"
            ? async (_, token) => { waiting.TrySetResult(); await Task.Delay(Timeout.Infinite, token); return new(HttpStatusCode.OK); }
            : (_, _) => Task.FromResult(new HttpResponseMessage(HttpStatusCode.OK) { Content = new StreamContent(new ControlledStream(async (buffer, token) =>
            {
                if (++reads == 1 && stage == "mid-stream") { buffer.Span[0] = 1; return 1; }
                waiting.TrySetResult(); await Task.Delay(Timeout.Infinite, token); return 0;
            })) });
        var destination = Path.Combine(f.Root, "archive.zip");
        await File.WriteAllTextAsync(destination, "previous download");
        var transfer = client.DownloadAsync(f.Catalog.Components.Sdk.Package, destination, null, default);
        await waiting.Task.WaitAsync(TimeSpan.FromSeconds(5));
        clock.Advance(UpdateTransport.DownloadIdleTimeout);
        Assert.Equal(expected, (await Assert.ThrowsAsync<UpdateException>(() => transfer).WaitAsync(TimeSpan.FromSeconds(5))).Code);
        Assert.False(File.Exists(destination + ".partial"));
        Assert.Equal("previous download", await File.ReadAllTextAsync(destination));
        Assert.Equal(1, f.Handler.Requests);
    }

    [Fact]
    public async Task ReceivingDataResetsTheIdleBudget()
    {
        using var f = new UpdateFixture();
        var clock = new FakeTimeProvider(f.Time.Now);
        using var client = new UpdateTransport(f.Distribution, f.Store, f.Environment, f.Handler, clock);
        var bytes = f.Packages["/sdk.zip"]; var position = 0;
        f.Handler.Respond = (_, _) => Task.FromResult(new HttpResponseMessage(HttpStatusCode.OK) { Content = new StreamContent(new ControlledStream((buffer, token) =>
        {
            token.ThrowIfCancellationRequested();
            if (position == bytes.Length) return ValueTask.FromResult(0);
            clock.Advance(TimeSpan.FromSeconds(25));
            var count = Math.Min(50, bytes.Length - position);
            bytes.AsSpan(position, count).CopyTo(buffer.Span); position += count;
            return ValueTask.FromResult(count);
        })) });
        var destination = Path.Combine(f.Root, "archive.zip");
        await client.DownloadAsync(f.Catalog.Components.Sdk.Package, destination, null, default);
        Assert.True(clock.GetUtcNow() - f.Time.Now > UpdateTransport.DownloadIdleTimeout);
        Assert.Equal(f.Catalog.Components.Sdk.Package.Sha256, UpdateStateStore.HashFile(destination));
    }

    [Fact]
    public async Task DataProgressCannotExtendTheOverallArchiveDeadline()
    {
        using var f = new UpdateFixture();
        var clock = new FakeTimeProvider(f.Time.Now);
        using var client = new UpdateTransport(f.Distribution, f.Store, f.Environment, f.Handler, clock);
        var destination = Path.Combine(f.Root, "archive.zip");
        var transfer = client.DownloadAsync(f.Catalog.Components.Sdk.Package, destination, _ => clock.Advance(UpdateTransport.DownloadTotalTimeout), default);
        Assert.Equal("download_timed_out", (await Assert.ThrowsAsync<UpdateException>(() => transfer)).Code);
        Assert.False(File.Exists(destination)); Assert.False(File.Exists(destination + ".partial"));
    }

    [Fact]
    public async Task CoordinatorShowsCheckTimeoutAsFailureRatherThanIdle()
    {
        using var f = new UpdateFixture();
        var clock = new FakeTimeProvider(f.Time.Now);
        var waiting = Signal();
        f.Handler.Respond = async (_, token) => { waiting.TrySetResult(); await Task.Delay(Timeout.Infinite, token); return new(HttpStatusCode.OK); };
        using var coordinator = new UpdateCoordinator(f.Distribution, f.Store, f.Environment, new StubPlatformVerifier(), f.Handler, clock);
        var check = coordinator.CheckAsync();
        await waiting.Task.WaitAsync(TimeSpan.FromSeconds(5)); clock.Advance(UpdateTransport.RequestTimeout);
        await check.WaitAsync(TimeSpan.FromSeconds(5));
        Assert.Equal(UpdatePhase.Failed, coordinator.Snapshot.Phase);
        Assert.Equal("check_timed_out", coordinator.Snapshot.ErrorCode);
        Assert.Null(coordinator.Snapshot.Plan);
    }

    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task CoordinatorDistinguishesUserCancelFromDownloadStallAndPreservesInstallation(bool cancel)
    {
        using var f = new UpdateFixture();
        var clock = new FakeTimeProvider(f.Time.Now);
        using var coordinator = new UpdateCoordinator(f.Distribution, f.Store, f.Environment, new StubPlatformVerifier(), f.Handler, clock);
        using var model = new UpdatesSettingsViewModel(coordinator, action => action());
        await coordinator.CheckAsync();
        var waiting = Signal();
        f.Handler.Respond = (_, _) => Task.FromResult(new HttpResponseMessage(HttpStatusCode.OK) { Content = new StreamContent(new ControlledStream(async (_, token) =>
        { waiting.TrySetResult(); await Task.Delay(Timeout.Infinite, token); return 0; })) });
        var transfer = coordinator.DownloadAsync();
        await waiting.Task.WaitAsync(TimeSpan.FromSeconds(5));
        if (cancel) coordinator.CancelDownload();
        clock.Advance(UpdateTransport.DownloadIdleTimeout);
        await transfer.WaitAsync(TimeSpan.FromSeconds(5));
        Assert.Equal(cancel ? UpdatePhase.Available : UpdatePhase.Failed, coordinator.Snapshot.Phase);
        Assert.Equal(cancel ? null : "download_stalled", coordinator.Snapshot.ErrorCode);
        Assert.NotNull(coordinator.Snapshot.Plan);
        Assert.True(model.PrimaryCommand.CanExecute(null));
        Assert.False(model.IsReady);
        Assert.Null(f.Store.Read().Staged);
        Assert.Null(f.Store.Read().Activation);
        Assert.Equal("0.1.0", f.Store.Read().Installation!.Sdk!.Version);
        Assert.Empty(Directory.GetFileSystemEntries(Path.Combine(f.Store.DirectoryPath, "staging")));
    }

    [Fact]
    public async Task UnrequestedTransportCancellationIsAnErrorRatherThanUserCancel()
    {
        using var f = new UpdateFixture();
        using var coordinator = new UpdateCoordinator(f.Distribution, f.Store, f.Environment, new StubPlatformVerifier(), f.Handler, f.Time);
        f.Handler.Respond = (_, _) => throw new TaskCanceledException("Transport cancelled itself");
        await coordinator.CheckAsync();
        Assert.Equal(UpdatePhase.Failed, coordinator.Snapshot.Phase);
        Assert.Equal("network_failed", coordinator.Snapshot.ErrorCode);
    }

    internal static TaskCompletionSource Signal() => new(TaskCreationOptions.RunContinuationsAsynchronously);
    private static HttpResponseMessage JsonStream(Stream stream)
    {
        var response = new HttpResponseMessage(HttpStatusCode.OK) { Content = new StreamContent(stream) };
        response.Content.Headers.ContentType = new("application/json"); return response;
    }
    private sealed class ControlledStream(Func<Memory<byte>, CancellationToken, ValueTask<int>> read) : Stream
    {
        public override bool CanRead => true;
        public override bool CanSeek => false;
        public override bool CanWrite => false;
        public override long Length => throw new NotSupportedException();
        public override long Position { get => throw new NotSupportedException(); set => throw new NotSupportedException(); }
        public override ValueTask<int> ReadAsync(Memory<byte> buffer, CancellationToken cancellationToken = default) => read(buffer, cancellationToken);
        public override int Read(byte[] buffer, int offset, int count) => throw new NotSupportedException();
        public override void Flush() => throw new NotSupportedException();
        public override long Seek(long offset, SeekOrigin origin) => throw new NotSupportedException();
        public override void SetLength(long value) => throw new NotSupportedException();
        public override void Write(byte[] buffer, int offset, int count) => throw new NotSupportedException();
    }
}
