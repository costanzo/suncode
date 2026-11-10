using System.IO.Compression;
using System.Net;
using System.Security.Cryptography;
using System.Text.Json;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.ViewModels;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Tests;

public sealed class BrowserComponentTests
{
    internal static DownloadNetworkConfiguration Network => new(true, true, null, new("no_proxy", "", "", "", []));

    internal sealed class Fixture : IDisposable
    {
        internal string Root { get; } = Path.Combine(Path.GetTempPath(), "suncode-browser-test-" + Guid.NewGuid().ToString("N"));
        internal byte[] Archive { get; }
        internal BrowserComponentPackage Package { get; }
        internal bool BadDigest;
        internal bool MissingVersion;
        internal bool BlockDownload;
        internal int Downloads;
        internal CancellationToken LastToken;
        internal Fixture(string? unsafeEntry = null)
        {
            using var memory = new MemoryStream();
            using (var zip = new ZipArchive(memory, ZipArchiveMode.Create, true))
            {
                foreach (var path in new[] { "LICENSE.txt", "runtimes/win-x64/native/libcef.dll", "runtimes/win-x64/native/icudtl.dat", "runtimes/win-x64/native/resources.pak", "CEF/win-x64/locales/en-US.pak" })
                {
                    using var writer = new StreamWriter(zip.CreateEntry(path).Open());
                    writer.Write("test fixture");
                }
                if (unsafeEntry is not null) { using var writer = new StreamWriter(zip.CreateEntry(unsafeEntry).Open()); writer.Write("unsafe"); }
            }
            Archive = memory.ToArray();
            Package = new("win-x64", "chromiumembeddedframework.runtime.win-x64", BrowserComponentCatalog.Version, Convert.ToHexStringLower(SHA256.HashData(Archive)), "libcef.dll");
            Directory.CreateDirectory(Root);
        }
        internal BrowserComponentService Service() => new(Root, BadDigest ? Package with { Sha256 = new string('0', 64) } : Package, _ => new Handler(this));
        private sealed class Handler(Fixture fixture) : HttpMessageHandler
        {
            protected override async Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken token)
            {
                if (request.RequestUri!.AbsolutePath.EndsWith("index.json"))
                    return new(HttpStatusCode.OK) { Content = new StringContent("{\"resources\":[{\"@type\":\"PackageBaseAddress/3.0.0\",\"@id\":\"https://packages.example.test/flat/\"}]}") };
                Interlocked.Increment(ref fixture.Downloads);
                fixture.LastToken = token;
                if (fixture.BlockDownload) await Task.Delay(Timeout.Infinite, token);
                return new(fixture.MissingVersion ? HttpStatusCode.NotFound : HttpStatusCode.OK)
                {
                    RequestMessage = request,
                    Content = new ByteArrayContent(fixture.Archive),
                };
            }
        }
        public void Dispose() { if (Directory.Exists(Root)) Directory.Delete(Root, true); }
    }

    [Fact]
    public async Task BothSourcesUsePinnedDigestAndInstallationSurvivesReopen()
    {
        using var fixture = new Fixture();
        foreach (var source in new[] { "official", "mirror" })
        {
            var service = fixture.Service();
            service.SelectSource(source);
            await service.InstallAsync(Network);
            Assert.Equal("ready", service.Inspect().State);
            Assert.Equal("ready", service.Status.State);
            var license = Path.Combine(service.RequireReady(), "LICENSE.txt");
            Assert.Equal("test fixture", await File.ReadAllTextAsync(license));
            await service.VerifyAsync();
            Assert.Equal("ready", service.Status.State);
            var reopened = fixture.Service();
            Assert.Equal(source, reopened.Source);
            Assert.Equal("ready", reopened.Inspect().State);
        }
    }

    [Fact]
    public async Task DigestMismatchNeverExtractsOrActivates()
    {
        using var fixture = new Fixture { BadDigest = true };
        var service = fixture.Service();
        await service.InstallAsync(Network);
        Assert.Equal("hash_mismatch", service.Status.State);
        Assert.Equal("missing", service.Inspect().State);
        Assert.False(File.Exists(Path.Combine(fixture.Root, "active.json")));
        Assert.Empty(Directory.EnumerateDirectories(fixture.Root, ".staging-*"));
    }

    [Theory]
    [InlineData("../escape")]
    [InlineData("runtimes/win-x64/native/../../escape")]
    [InlineData("runtimes/win-x64/native/libcef.dll")]
    public async Task VerifiedArchivesStillRejectUnsafeAndDuplicateEntries(string name)
    {
        using var fixture = new Fixture(name);
        var service = fixture.Service();
        await service.InstallAsync(Network);
        Assert.NotEqual("ready", service.Status.State);
        Assert.Equal("missing", service.Inspect().State);
        Assert.False(File.Exists(Path.Combine(fixture.Root, "active.json")));
    }

    [Fact]
    public async Task FailedRepairAndCanceledDuplicateDownloadsPreserveActivePointer()
    {
        using var fixture = new Fixture();
        var service = fixture.Service();
        await service.InstallAsync(Network);
        var original = service.Inspect().Directory;
        fixture.MissingVersion = true;
        await service.InstallAsync(Network);
        Assert.Equal("source_missing", service.Status.State);
        Assert.Equal(original, service.Inspect().Directory);
        fixture.MissingVersion = false;
        fixture.BlockDownload = true;
        var first = service.InstallAsync(Network);
        var second = service.InstallAsync(Network);
        Assert.Same(first, second);
        service.Cancel();
        await first;
        Assert.Equal("ready", service.Status.State);
        Assert.Equal(original, service.Inspect().Directory);
        Assert.Empty(Directory.EnumerateDirectories(fixture.Root, ".staging-*"));
    }

    [Fact]
    public async Task MetadataAndExplicitVerificationDetectDifferentCorruption()
    {
        using var fixture = new Fixture();
        var service = fixture.Service();
        await service.InstallAsync(Network);
        var directory = service.RequireReady();
        var library = Path.Combine(directory, "libcef.dll");
        await File.WriteAllTextAsync(library, "bad! fixture"); // Same length as the original.
        Assert.Equal("ready", service.Inspect().State);
        await service.VerifyAsync();
        Assert.Equal("corrupt", service.Status.State);
        Assert.Equal("corrupt", service.Inspect().State);
        await service.InstallAsync(Network);
        Assert.Equal("ready", service.Inspect().State);
        File.Delete(Path.Combine(service.RequireReady(), "icudtl.dat"));
        Assert.Equal("corrupt", service.Inspect().State);
        Assert.Equal("corrupt", service.Status.State);
    }

    [Fact]
    public async Task SuccessfulRepairRemovesInactiveInstallationsAndPreservesOtherDirectories()
    {
        using var fixture = new Fixture();
        var service = fixture.Service();
        await service.InstallAsync(Network);
        var previous = service.RequireReady();
        var unrelated = Path.Combine(fixture.Root, "installed", "notes");
        Directory.CreateDirectory(unrelated);
        await File.WriteAllTextAsync(Path.Combine(unrelated, "keep.txt"), "keep");

        await service.InstallAsync(Network);

        var active = service.RequireReady();
        Assert.NotEqual(previous, active);
        Assert.False(Directory.Exists(previous));
        Assert.True(File.Exists(Path.Combine(active, "libcef.dll")));
        Assert.True(File.Exists(Path.Combine(unrelated, "keep.txt")));
        Assert.Single(Directory.EnumerateDirectories(Path.Combine(fixture.Root, "installed")),
            directory => Guid.TryParseExact(Path.GetFileName(directory), "N", out _));
    }

    [Fact]
    public async Task LoadedRuntimeReplacementRequiresRestartAndRetainsLoadedFiles()
    {
        using var fixture = new Fixture();
        var service = fixture.Service();
        await service.InstallAsync(Network);
        var loaded = service.RequireReady();
        service.MarkLoaded(loaded);
        await service.InstallAsync(Network);
        Assert.Equal("restart_required", service.Inspect().State);
        Assert.True(File.Exists(Path.Combine(loaded, "libcef.dll")));
        service.CleanupUnusedInstallations();
        Assert.True(File.Exists(Path.Combine(loaded, "libcef.dll")));
        var restarted = fixture.Service();
        restarted.CleanupUnusedInstallations();
        Assert.False(Directory.Exists(loaded));
        Assert.Equal("ready", restarted.Inspect().State);
    }

    [Fact]
    public async Task MalformedInstalledPathsFailClosedWithoutCrashingSettings()
    {
        using var fixture = new Fixture();
        var service = fixture.Service();
        await service.InstallAsync(Network);
        var directory = service.RequireReady();
        var manifestPath = Path.Combine(directory, "installation.json");
        var manifest = JsonSerializer.Deserialize<BrowserInstallation>(await File.ReadAllTextAsync(manifestPath))!;
        var files = manifest.Files.Append(new BrowserInstalledFile("../outside", 0, new string('0', 64))).ToArray();
        await File.WriteAllTextAsync(manifestPath, JsonSerializer.Serialize(manifest with { Files = files }));
        Assert.Equal("corrupt", service.Inspect().State);
        Assert.Equal("corrupt", service.Status.State);
        service.CleanupUnusedInstallations();
        Assert.True(Directory.Exists(directory));
    }

    [Fact]
    public void MissingLocalComponentCannotStartEvenWhenSdkStatusClaimsReady()
    {
        using var fixture = new Fixture();
        var host = new FakeViewModelHost { SelectedProject = new("project", "Project", fixture.Root) };
        var model = new BrowserRuntimeViewModel(host, fixture.Service())
        {
            Runtime = new(true, "ready", "not_started", "", "", "CEF", 1, "unverified", null, null, null, 0, "full", null),
        };
        Assert.False(model.CanStart);
        Assert.True(model.CanInstallComponent);
        Assert.False(model.CanVerify);
    }

    [Fact]
    public async Task ExplicitDesktopInstallationWorksWithoutAnAgentConnection()
    {
        using var fixture = new Fixture();
        var service = fixture.Service();
        var host = new FakeViewModelHost();
        var model = new BrowserRuntimeViewModel(host, service);
        await ((AsyncRelayCommand)model.InstallComponentCommand).ExecuteAsync();
        Assert.Equal("ready", service.Inspect().State);
        Assert.False(model.CanStart);
        Assert.Empty(host.Errors);
    }
}
