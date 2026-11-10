using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Tests;

public sealed class BrowserComponentUninstallTests
{
    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task UninstallRemovesManagedPayloadsAndTemporaryPackages(bool corruptPointer)
    {
        using var fixture = new BrowserComponentTests.Fixture();
        var service = fixture.Service();
        service.SelectSource("mirror");
        await service.InstallAsync(BrowserComponentTests.Network);
        var active = service.RequireReady();
        var orphan = Path.Combine(fixture.Root, "installed", Guid.NewGuid().ToString("N"));
        var staging = Path.Combine(fixture.Root, ".staging-" + Guid.NewGuid().ToString("N"));
        var saved = Path.Combine(fixture.Root, "saved-browser-data");
        Directory.CreateDirectory(orphan);
        Directory.CreateDirectory(staging);
        Directory.CreateDirectory(saved);
        await File.WriteAllTextAsync(Path.Combine(orphan, "libcef.dll"), "old runtime");
        await File.WriteAllTextAsync(Path.Combine(staging, "runtime.nupkg"), "temporary package");
        await File.WriteAllTextAsync(Path.Combine(saved, "cookies"), "saved data");
        if (corruptPointer) await File.WriteAllTextAsync(Path.Combine(fixture.Root, "active.json"), "\"../outside\"");

        await service.UninstallAsync();

        Assert.Equal("missing", service.Status.State);
        Assert.Equal("missing", service.Inspect().State);
        Assert.False(service.HasInstallation);
        Assert.False(service.UninstallPending);
        Assert.False(Directory.Exists(active));
        Assert.False(Directory.Exists(orphan));
        Assert.False(Directory.Exists(staging));
        Assert.False(File.Exists(Path.Combine(fixture.Root, "active.json")));
        Assert.Equal("saved data", await File.ReadAllTextAsync(Path.Combine(saved, "cookies")));
        Assert.Equal("mirror", fixture.Service().Source);
        await service.InstallAsync(BrowserComponentTests.Network);
        Assert.Equal("ready", service.Inspect().State);
    }

    [Fact]
    public async Task LoadedRuntimeIsRemovedOnNextPrimaryStartup()
    {
        using var fixture = new BrowserComponentTests.Fixture();
        var service = fixture.Service();
        await service.InstallAsync(BrowserComponentTests.Network);
        var loaded = service.RequireReady();
        service.MarkLoaded(loaded);
        var downloads = fixture.Downloads;

        await service.UninstallAsync();

        Assert.Equal("uninstall_restart_required", service.Status.State);
        Assert.True(service.UninstallPending);
        Assert.True(Directory.Exists(loaded));
        Assert.Throws<BrowserComponentException>(() => service.RequireReady());
        await service.InstallAsync(BrowserComponentTests.Network);
        Assert.Equal(downloads, fixture.Downloads);
        service.CleanupUnusedInstallations();
        Assert.True(Directory.Exists(loaded));

        var restarted = fixture.Service();
        restarted.CleanupUnusedInstallations();
        Assert.Equal("missing", restarted.Inspect().State);
        Assert.False(restarted.HasInstallation);
        Assert.False(restarted.UninstallPending);
        Assert.False(Directory.Exists(loaded));
    }

    [Fact]
    public async Task InterruptedUninstallIsRetriedOnStartup()
    {
        if (!OperatingSystem.IsWindows()) return;
        using var fixture = new BrowserComponentTests.Fixture();
        var service = fixture.Service();
        await service.InstallAsync(BrowserComponentTests.Network);
        var library = Path.Combine(service.RequireReady(), "libcef.dll");
        using (var lockedLibrary = new FileStream(library, FileMode.Open, FileAccess.Read, FileShare.Read))
        {
            await service.UninstallAsync();
            Assert.Equal("uninstall_failed", service.Status.State);
            Assert.Equal("uninstall_failed", service.Inspect().State);
            Assert.True(service.UninstallPending);
            Assert.True(File.Exists(library));
        }

        var restarted = fixture.Service();
        restarted.CleanupUnusedInstallations();
        Assert.Equal("missing", restarted.Status.State);
        Assert.False(restarted.HasInstallation);
        Assert.False(restarted.UninstallPending);
    }

    [Fact]
    public async Task UninstallRequiresConfirmationAndWorksWithoutTheAgent()
    {
        using var fixture = new BrowserComponentTests.Fixture();
        var service = fixture.Service();
        await service.InstallAsync(BrowserComponentTests.Network);
        var model = new BrowserRuntimeViewModel(new FakeViewModelHost(), service);
        var dialogs = new UninstallDialogs();
        model.AttachDialogs(dialogs);

        model.UninstallComponentCommand.Execute(null);

        Assert.NotNull(dialogs.Request);
        Assert.True(dialogs.Request.Destructive);
        Assert.True(service.HasInstallation);
        Assert.NotNull(dialogs.Confirmed);
        dialogs.Confirmed();
        await service.UninstallAsync();
        model.RefreshPresentation();
        Assert.Equal("missing", service.Inspect().State);
        Assert.True(model.ShowComponentInstall);
        Assert.False(model.ShowComponentUninstall);
    }

    private sealed class UninstallDialogs : ISettingsDialogs
    {
        internal ConfirmationRequest? Request;
        internal Action? Confirmed;
        public void Confirm(ConfirmationRequest request, Action confirmed)
        {
            Request = request;
            Confirmed = confirmed;
        }
        public Task<bool> CopyTextAsync(string text) => Task.FromResult(false);
    }
}
