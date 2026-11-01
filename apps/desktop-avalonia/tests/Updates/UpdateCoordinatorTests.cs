using System.Net;
using SunCode.Desktop.Infrastructure.Updates;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Tests.Updates;

public sealed class UpdateCoordinatorTests
{
    [Fact]
    public async Task MatchingRunningVersionsAreCurrentWithoutEnrollingAnUnmanagedBuild()
    {
        using var f = new UpdateFixture();
        f.Store.Change(state => state.Installation = null);
        f.Catalog = f.Catalog with { Components = f.Catalog.Components with { Sdk = f.Catalog.Components.Sdk with { Version = "0.1.0" } } };
        using var coordinator = new UpdateCoordinator(f.Distribution, f.Store, f.Environment, new StubPlatformVerifier(), f.Handler, f.Time);
        coordinator.ReadRunningVersionsAsync = () => Task.FromResult(new RunningUpdateVersions("0.1.0", 27, "0.1.0", 27));
        using var model = new UpdatesSettingsViewModel(coordinator, action => action());
        await model.LoadIdentityAsync();
        await coordinator.CheckAsync();
        Assert.Equal(UpdatePhase.Current, coordinator.Snapshot.Phase);
        Assert.Equal(StatusTone.Success, model.Tone);
        Assert.Equal("v0.1.0", model.SdkVersion);
        Assert.False(model.ShowSdkTarget); Assert.False(model.ShowDesktopTarget);
        Assert.Null(coordinator.Snapshot.Plan);
        await coordinator.DownloadAsync();
        await coordinator.RestartAsync();
        Assert.Null(f.Store.Read().Installation);
        Assert.Null(f.Store.Read().Staged);
        Assert.Null(f.Store.Read().Activation);
        Assert.Equal(1, f.Handler.Requests);
    }
    [Theory]
    [InlineData("0.0.9", 27, "0.1.0", 27)]
    [InlineData("0.1.0", 27, "0.0.9", 27)]
    [InlineData("0.1.0", 27, null, null)]
    [InlineData("0.1.0", 27, "0.1.0", 28)]
    [InlineData("0.1.0", 28, "0.1.0", 28)]
    public async Task UnmanagedVersionOrAbiMismatchCannotBeCalledCurrent(string desktop, int desktopAbi, string? sdk, int? sdkAbi)
    {
        using var f = new UpdateFixture();
        f.Store.Change(state => state.Installation = null);
        f.Catalog = f.Catalog with { Components = f.Catalog.Components with { Sdk = f.Catalog.Components.Sdk with { Version = "0.1.0" } } };
        using var coordinator = new UpdateCoordinator(f.Distribution, f.Store, f.Environment, new StubPlatformVerifier(), f.Handler, f.Time);
        coordinator.ReadRunningVersionsAsync = () => Task.FromResult(new RunningUpdateVersions(desktop, (uint)desktopAbi, sdk, sdkAbi is { } value ? (uint)value : null));
        await coordinator.CheckAsync();
        Assert.Equal("installation_unmanaged", coordinator.Snapshot.ErrorCode);
        Assert.NotEqual(UpdatePhase.Current, coordinator.Snapshot.Phase);
        Assert.Null(coordinator.Snapshot.Plan);
    }
    [Theory]
    [InlineData(false, "signature_invalid")]
    [InlineData(true, "catalog_expired")]
    public async Task UnmanagedCurrentCheckStillRequiresTrustedFreshMetadata(bool expired, string expected)
    {
        using var f = new UpdateFixture();
        f.Store.Change(state => state.Installation = null);
        f.Catalog = f.Catalog with { Components = f.Catalog.Components with { Sdk = f.Catalog.Components.Sdk with { Version = "0.1.0" } } };
        if (expired) f.Time.Now = f.Time.Now.AddDays(8);
        else f.Handler.Respond = (_, _) =>
        {
            var changed = System.Text.Json.Nodes.JsonNode.Parse(f.Sign())!;
            changed["signed"]!["components"]!["sdk"]!["releaseNotes"]!["en-US"] = "tampered";
            return Task.FromResult(UpdateFixture.JsonResponse(System.Text.Encoding.UTF8.GetBytes(changed.ToJsonString())));
        };
        using var coordinator = new UpdateCoordinator(f.Distribution, f.Store, f.Environment, new StubPlatformVerifier(), f.Handler, f.Time);
        var identityQueries = 0;
        coordinator.ReadRunningVersionsAsync = () => { identityQueries++; return Task.FromResult(new RunningUpdateVersions("0.1.0", 27, "0.1.0", 27)); };
        await coordinator.CheckAsync();
        Assert.Equal(expected, coordinator.Snapshot.ErrorCode);
        Assert.Equal(0, identityQueries);
        Assert.Null(coordinator.Snapshot.Plan);
    }
    [Fact]
    public async Task MissingInstallationReceiptIsAWarningAndNeverCreatesAnInstallablePlan()
    {
        using var f = new UpdateFixture();
        f.Store.Change(state => state.Installation = null);
        using var coordinator = new UpdateCoordinator(f.Distribution, f.Store, f.Environment, new StubPlatformVerifier(), f.Handler, f.Time);
        coordinator.ReadRunningVersionsAsync = () => Task.FromResult(new RunningUpdateVersions("0.1.0", 27, "0.1.0", 27));
        using var vm = new UpdatesSettingsViewModel(coordinator, action => action());
        await coordinator.CheckAsync();
        Assert.Equal("installation_unmanaged", coordinator.Snapshot.ErrorCode);
        Assert.Equal(StatusTone.Warning, vm.Tone);
        Assert.Null(coordinator.Snapshot.Plan);
        Assert.False(vm.IsReady);
        await coordinator.DownloadAsync();
        Assert.Null(f.Store.Read().Installation);
        Assert.Null(f.Store.Read().Staged);
    }
    [Fact]
    public async Task DamagedManagedDesktopStillRequiresRepair()
    {
        using var f = new UpdateFixture();
        File.WriteAllText(Path.Combine(f.Installed.Desktop.Directory, "SunCode.dll"), "damaged");
        using var coordinator = new UpdateCoordinator(f.Distribution, f.Store, f.Environment, new StubPlatformVerifier(), f.Handler, f.Time);
        using var vm = new UpdatesSettingsViewModel(coordinator, action => action());
        await coordinator.CheckAsync();
        Assert.Equal("desktop_damaged", coordinator.Snapshot.ErrorCode);
        Assert.Equal(StatusTone.Danger, vm.Tone);
        Assert.Null(coordinator.Snapshot.Plan);
    }
    [Theory]
    [InlineData("network_failed", "The download was interrupted")]
    [InlineData("download_failed", "The download was interrupted")]
    [InlineData("archive_invalid", "The update couldn’t be verified")]
    [InlineData("package_layout", "The update couldn’t be verified")]
    public async Task DownloadErrorsDescribeTheFailedStageAndPreserveRetry(string code, string heading)
    {
        using var f = new UpdateFixture();
        if (code is "archive_invalid" or "package_layout")
            f.Catalog = f.Catalog with { Components = f.Catalog.Components with { Sdk = f.Catalog.Components.Sdk with
            { Package = f.Package("sdk", UpdateFixture.Zip((code == "archive_invalid" ? "../escape" : "wrong-library", "invalid"))) } } };
        using var coordinator = new UpdateCoordinator(f.Distribution, f.Store, f.Environment, new StubPlatformVerifier(), f.Handler, f.Time);
        using var vm = new UpdatesSettingsViewModel(coordinator, action => action());
        await coordinator.CheckAsync();
        if (code is "network_failed" or "download_failed")
            f.Handler.Respond = (_, _) => throw (code == "network_failed" ? new HttpRequestException() : new UpdateException(code, "Test failure"));
        await coordinator.DownloadAsync();
        Assert.Equal(heading, vm.Heading);
        Assert.Equal("Download again", vm.PrimaryText);
        Assert.True(vm.PrimaryCommand.CanExecute(null));
        Assert.Null(f.Store.Read().Staged);
        Assert.Equal("0.1.0", f.Store.Read().Installation!.Sdk!.Version);
    }
    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task SdkDamageIsVisibleBeforeCheckingEvenWhenDeliveryIsDisabled(bool missing)
    {
        using var f = new UpdateFixture();
        var library = Path.Combine(f.Installed.Sdk!.Directory, UpdateStager.SdkLibraryName);
        if (missing) File.Delete(library); else File.WriteAllText(library, "damaged");
        using var coordinator = new UpdateCoordinator(new(), f.Store, f.Environment, new StubPlatformVerifier(), f.Handler, f.Time);
        using var vm = new UpdatesSettingsViewModel(coordinator, action => action());
        await vm.LoadIdentityAsync();
        Assert.Equal("Unavailable", vm.SdkVersion);
        Assert.Equal("Unavailable", vm.SdkStatus);
        Assert.Equal(0, f.Handler.Requests);
    }
    [Fact]
    public async Task RuntimeFailureForcesSdkRepairEvenWhenFileManifestIsHealthy()
    {
        using var f = new UpdateFixture();
        using var coordinator = new UpdateCoordinator(f.Distribution, f.Store, f.Environment, new StubPlatformVerifier(), f.Handler, f.Time);
        using var vm = new UpdatesSettingsViewModel(coordinator, action => action());
        coordinator.RecordSdkRuntimeAvailability(false);
        await vm.LoadIdentityAsync();
        Assert.True(UpdatePlanner.Healthy(f.Installed.Sdk));
        Assert.Equal("Unavailable", vm.SdkStatus);
        await coordinator.CheckAsync();
        Assert.True(coordinator.Snapshot.Plan!.Repair);
        Assert.True(coordinator.Snapshot.Plan.Sdk);
        Assert.Equal("Repair Agent SDK", vm.PrimaryText);
    }
    [Fact]
    public async Task SdkOnlyFlowPreservesInstalledVersionsUntilConfirmedRestartAndBlocksActiveWork()
    {
        using var f=new UpdateFixture();
        using var coordinator=new UpdateCoordinator(f.Distribution,f.Store,f.Environment,new StubPlatformVerifier(),f.Handler,f.Time);
        using var vm=new UpdatesSettingsViewModel(coordinator,action=>action());
        await coordinator.CheckAsync();
        Assert.Equal(UpdatePhase.Available,coordinator.Snapshot.Phase);
        Assert.False(vm.ShowDesktopTarget);Assert.True(vm.ShowSdkTarget);
        Assert.Contains("Agent SDK only",vm.PlanSummary);
        await coordinator.DownloadAsync();
        Assert.Equal(UpdatePhase.Ready,coordinator.Snapshot.Phase);
        Assert.Equal("0.1.0",f.Store.Read().Installation!.Sdk!.Version);
        coordinator.IsBusyAsync=()=>Task.FromResult(true);
        await vm.PollAsync();
        Assert.False(vm.PrimaryCommand.CanExecute(null));
        await coordinator.RestartAsync();
        Assert.Null(f.Store.Read().Activation);
        Assert.Equal(UpdatePhase.Ready,coordinator.Snapshot.Phase);
        coordinator.IsBusyAsync=()=>Task.FromResult(false);
        await vm.PollAsync();
        var dialogs=new CaptureUpdateDialogs();vm.AttachDialogs(dialogs);
        await ((AsyncRelayCommand)vm.PrimaryCommand).ExecuteAsync();
        Assert.NotNull(dialogs.Request);Assert.False(dialogs.Request.Destructive);
        Assert.Equal("Later",dialogs.Request.CancelLabel);
        Assert.Null(f.Store.Read().Activation); // Showing a dialog isn't permission to install.
    }
    [Fact]
    public async Task CheckFailureAndIntegrityFailureNeverEnableRestartOrChangeInstallation()
    {
        using var f=new UpdateFixture();
        using var coordinator=new UpdateCoordinator(f.Distribution,f.Store,f.Environment,new StubPlatformVerifier(),f.Handler,f.Time);
        await coordinator.CheckAsync();
        var normal=f.Handler.Respond;
        f.Handler.Respond=(request,token)=>request.RequestUri!.Host.StartsWith("downloads",StringComparison.Ordinal)
            ?Task.FromResult(new HttpResponseMessage(HttpStatusCode.OK) {Content=new ByteArrayContent([1,2,3])}) : normal(request,token);
        await coordinator.DownloadAsync();
        Assert.Equal(UpdatePhase.Failed,coordinator.Snapshot.Phase);Assert.Null(f.Store.Read().Staged);
        Assert.Equal("0.1.0",f.Store.Read().Installation!.Sdk!.Version);
        f.Handler.Respond=(_,_)=>Task.FromResult(new HttpResponseMessage(HttpStatusCode.NotFound));
        await coordinator.CheckAsync();
        Assert.Equal(UpdatePhase.Failed,coordinator.Snapshot.Phase);Assert.Null(coordinator.Snapshot.Plan);
    }
    [Fact]
    public async Task CancelAndWindowDisposalDoNotCancelSharedChecksOrPersistFalseSuccess()
    {
        using var f=new UpdateFixture();
        using var coordinator=new UpdateCoordinator(f.Distribution,f.Store,f.Environment,new StubPlatformVerifier(),f.Handler,f.Time);
        var vm=new UpdatesSettingsViewModel(coordinator,action=>action());
        await coordinator.CheckAsync();
        vm.Dispose();
        await coordinator.DownloadAsync();
        Assert.Equal(UpdatePhase.Ready,coordinator.Snapshot.Phase);
        coordinator.SetAutomaticChecks(false);
        using var reopened=new UpdatesSettingsViewModel(coordinator,action=>action());
        Assert.False(reopened.AutomaticChecks);
        Assert.False(new UpdateStateStore(f.Store.DirectoryPath).Read().AutomaticChecks);
        Assert.Equal("0.1.0",f.Store.Read().Installation!.Sdk!.Version);
    }
    [Fact]
    public async Task UnconfiguredBuildHasAnHonestStateAndDoesNotContactAnyService()
    {
        using var f=new UpdateFixture();
        using var coordinator=new UpdateCoordinator(new(),f.Store,f.Environment,new StubPlatformVerifier(),f.Handler,f.Time);
        using var vm=new UpdatesSettingsViewModel(coordinator,action=>action());
        Assert.False(vm.PrimaryCommand.CanExecute(null));
        Assert.Contains("not configured",vm.Heading);
        coordinator.StartAutomaticChecks();
        await coordinator.CheckAsync();
        Assert.Equal(0,f.Handler.Requests);
        Assert.Equal("configuration_missing",coordinator.Snapshot.ErrorCode);
    }
    [Fact]
    public async Task CancelDuringDownloadRetainsInstalledPairAndDoesNotPublishAnIncompletePlan()
    {
        using var f=new UpdateFixture();
        using var coordinator=new UpdateCoordinator(f.Distribution,f.Store,f.Environment,new StubPlatformVerifier(),f.Handler,f.Time);
        await coordinator.CheckAsync();
        var started=new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
        f.Handler.Respond=async (_,token)=> {started.SetResult();await Task.Delay(Timeout.Infinite,token);return new(HttpStatusCode.OK);};
        var download=coordinator.DownloadAsync();await started.Task.WaitAsync(TimeSpan.FromSeconds(5));
        Assert.Equal(UpdatePhase.Downloading,coordinator.Snapshot.Phase);
        await coordinator.CheckAsync();Assert.Equal(UpdatePhase.Downloading,coordinator.Snapshot.Phase);
        coordinator.CancelDownload();await download;
        Assert.Equal(UpdatePhase.Available,coordinator.Snapshot.Phase);
        Assert.Null(f.Store.Read().Staged);Assert.True(UpdatePlanner.Healthy(f.Store.Read().Installation!.Sdk));
    }
    private sealed class CaptureUpdateDialogs : ISettingsDialogs
    {
        public ConfirmationRequest? Request {get;private set;}
        public void Confirm(ConfirmationRequest request,Action confirmed)=>Request=request;
        public Task<bool> CopyTextAsync(string text)=>Task.FromResult(false);
    }
}
