using SunCode.Desktop.Infrastructure;

namespace SunCode.Desktop.Tests;

public sealed class DesktopActivationTests
{
    [Fact]
    public void Launch_argument_round_trips_only_bounded_navigation_intent()
    {
        var request = DesktopActivationRequest.Session(
            "project-1", "child-1", "approval-1", "notification",
            "parent-1", "child-1");

        Assert.True(DesktopActivationRequest.TryParseLaunchArgument(request.ToLaunchArgument(), out var parsed));
        Assert.Equal(request, parsed);
        Assert.False(DesktopActivationRequest.TryParseLaunchArgument("not-base64", out _));
    }

    [Fact]
    public async Task Framing_round_trips_and_rejects_oversized_payloads()
    {
        var request = DesktopActivationRequest.Session("project-1", "session-1", "turn-1", "test");
        await using var stream = new MemoryStream();
        await DesktopInstanceCoordinator.WriteFrameAsync(stream, request, CancellationToken.None);
        stream.Position = 0;

        var parsed = await DesktopInstanceCoordinator.ReadFrameAsync<DesktopActivationRequest>(stream, CancellationToken.None);
        Assert.Equal(request, parsed);

        await using var oversized = new MemoryStream();
        var header = new byte[] { 0, 1, 0, 1 };
        await oversized.WriteAsync(header);
        oversized.Position = 0;
        await Assert.ThrowsAsync<InvalidDataException>(() =>
            DesktopInstanceCoordinator.ReadFrameAsync<DesktopActivationRequest>(oversized, CancellationToken.None));
    }

    [Fact]
    public void Child_activation_requires_a_matching_parent_child_pair()
    {
        var invalid = DesktopActivationRequest.Session(
            "project-1", "child-2", "approval-1", "test", "parent-1", "child-1");
        Assert.Throws<InvalidDataException>(invalid.Validate);
    }

    [Fact]
    public async Task Secondary_forwards_to_the_primary_instance_for_the_same_data_directory()
    {
        if (OperatingSystem.IsWindows()) return;
        using var directory = new TemporaryDirectory();
        using var primary = DesktopInstanceCoordinator.Acquire(directory.Path);
        Assert.True(primary.IsPrimary);
        var received = new TaskCompletionSource<DesktopActivationRequest>(TaskCreationOptions.RunContinuationsAsynchronously);
        primary.ActivationReceived += request => received.TrySetResult(request);
        primary.StartServer();
        using var secondary = DesktopInstanceCoordinator.Acquire(directory.Path);
        Assert.False(secondary.IsPrimary);
        var request = DesktopActivationRequest.Session("project-1", "session-1", "turn-1", "test");

        Assert.True(await secondary.ForwardAsync(request));
        Assert.Equal(request, await received.Task.WaitAsync(TimeSpan.FromSeconds(2)));
    }

    private sealed class TemporaryDirectory : IDisposable
    {
        public string Path { get; } = System.IO.Path.Combine(System.IO.Path.GetTempPath(), $"suncode-ipc-{Guid.NewGuid():N}");
        public TemporaryDirectory() => Directory.CreateDirectory(Path);
        public void Dispose()
        {
            try { Directory.Delete(Path, true); } catch (IOException) { }
        }
    }
}
