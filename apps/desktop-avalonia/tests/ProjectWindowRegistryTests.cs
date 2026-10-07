using SunCode.Desktop.Infrastructure;

namespace SunCode.Desktop.Tests;

public sealed class ProjectWindowRegistryTests
{
    private sealed class FakeWindow;

    [Fact]
    public void OpeningIsDeduplicatedAndResolvesDisposition()
    {
        var registry = new ProjectWindowRegistry<FakeWindow>();
        Assert.Equal(ProjectWindowDisposition.OpenNew, registry.Resolve("p"));

        Assert.True(registry.TryBeginOpening("p"));
        Assert.False(registry.TryBeginOpening("p"));
        Assert.Equal(ProjectWindowDisposition.AwaitOpening, registry.Resolve("p"));

        registry.Register("p", new FakeWindow());
        Assert.Equal(ProjectWindowDisposition.ActivateExisting, registry.Resolve("p"));
        registry.EndOpening("p");
        Assert.False(registry.IsOpening("p"));
        Assert.Equal(1, registry.Count);
    }

    [Fact]
    public async Task WhenOpenedReportsWhetherAWindowWasRegistered()
    {
        var registry = new ProjectWindowRegistry<FakeWindow>();
        registry.TryBeginOpening("ok");
        registry.TryBeginOpening("failed");
        var ok = registry.WhenOpened("ok");
        var failed = registry.WhenOpened("failed");
        Assert.False(ok.IsCompleted);

        registry.Register("ok", new FakeWindow());
        registry.EndOpening("ok");
        registry.EndOpening("failed");

        Assert.True(await ok);
        Assert.False(await failed);
        Assert.False(await registry.WhenOpened("never"));
    }

    [Fact]
    public async Task WindowWaitersAreReleasedOnRegistration()
    {
        var registry = new ProjectWindowRegistry<FakeWindow>();
        var first = registry.WaitForWindowAsync("p");
        var second = registry.WaitForWindowAsync("p");
        Assert.False(first.IsCompleted);

        var window = new FakeWindow();
        registry.Register("p", window);

        Assert.Same(window, await first);
        Assert.Same(window, await second);
        Assert.Same(window, await registry.WaitForWindowAsync("p"));
    }

    [Fact]
    public void RemoveForgetsTheWindow()
    {
        var registry = new ProjectWindowRegistry<FakeWindow>();
        registry.Register("p", new FakeWindow());
        Assert.True(registry.Remove("p"));
        Assert.False(registry.TryGet("p", out _));
        Assert.Equal(0, registry.Count);
    }
}
