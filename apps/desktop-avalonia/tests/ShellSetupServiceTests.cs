using System.IO.Compression;
using System.Security.Cryptography;
using SunCode.Desktop.Infrastructure;
using SunCode.Sdk;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Tests;

public sealed class ShellSetupServiceTests
{
    private static DownloadNetworkConfiguration Network => new(true, true, null, new("system", "", "", "", []));
    private static string ShellPath => OperatingSystem.IsWindows() ? ShellResolver.SystemPath(false) : "/bin/sh";

    [Fact]
    public async Task ProjectWindowsShareSetupAndClosingOneKeepsItsSdkReferenceAlive()
    {
        using var first = await AgentSdk.OpenAsync("shell-lifetime-test");
        using var second = await AgentSdk.OpenAsync("shell-lifetime-test");
        var acquired = 0;
        try
        {
            await ShellSetupService.AcquireAsync("shell-lifetime-test"); ++acquired;
            var service = ShellSetupService.Current;
            await ShellSetupService.AcquireAsync("shell-lifetime-test"); ++acquired;
            Assert.Same(service, ShellSetupService.Current);
            first.Dispose();
            await ShellSetupService.ReleaseAsync(); --acquired;
            Assert.Same(service, ShellSetupService.Current);
            second.Dispose(); // Only the setup service's retained managed reference remains.
            Assert.Equal("available", (await service.CheckAsync(new("agent", new()))).State);
        }
        finally { while (acquired-- > 0) await ShellSetupService.ReleaseAsync(); }
        Assert.Throws<InvalidOperationException>(() => ShellSetupService.Current);
    }

    [Fact]
    public void DesktopCanReadTheUnixLoginShellWithoutAnEnvironmentOverride()
    {
        if (OperatingSystem.IsWindows()) return;
        var shell = ShellResolver.LoginShell();
        Assert.NotNull(shell);
        Assert.True(Path.IsPathFullyQualified(shell));
        Assert.True(File.Exists(shell));
    }

    [Fact]
    public async Task DesktopDetectsTheShellAndPersistsLaunchFactsInOneRequest()
    {
        ShellPreferencesRequest? saved = null;
        var service = new ShellSetupService(() => Task.FromResult<(ShellConfiguration, ShellConfiguration)>((new(), new())),
            request => { saved = request; return Task.CompletedTask; }, () => Task.FromResult(Network));
        var result = await service.SaveAsync(new("both", new("custom", ShellPath)));
        Assert.Equal("available", result.State);
        Assert.NotNull(saved?.Agent?.Resolved);
        Assert.Equal(saved!.Agent, saved.Terminal);
        Assert.True(Path.IsPathFullyQualified(saved.Agent!.Resolved!.Path));
        Assert.Null(result.Configuration.Resolved); // Launch facts do not become editor draft state.
    }

    [Fact]
    public async Task UnavailableDraftCannotReplaceEitherSavedPreference()
    {
        var writes = 0;
        var service = new ShellSetupService(() => Task.FromResult<(ShellConfiguration, ShellConfiguration)>((new(), new())),
            _ => { ++writes; return Task.CompletedTask; }, () => Task.FromResult(Network));
        var result = await service.SaveAsync(new("both", new("custom", Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString(), "sh"))));
        Assert.Equal("unavailable", result.State);
        Assert.Equal(0, writes);
    }

    [Fact]
    public async Task InitializationClearsStaleLaunchFactsWhenTheExecutableDisappears()
    {
        var path = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString(), "sh");
        var configuration = new ShellConfiguration("custom", path, new(path, "sh", "stale", []));
        ShellPreferencesRequest? saved = null;
        var service = new ShellSetupService(() => Task.FromResult<(ShellConfiguration, ShellConfiguration)>((configuration, new())),
            request => { saved = request; return Task.CompletedTask; }, () => Task.FromResult(Network));
        await service.InitializeAsync();
        Assert.Equal(configuration.Path, saved?.Agent?.Path);
        Assert.Null(saved?.Agent?.Resolved);
        Assert.Null(saved?.Terminal);
    }

    [Fact]
    public async Task CancellingInstallationKeepsPreferencesAndShutdownWaitsForCleanup()
    {
        var started = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
        var cleaned = false; var writes = 0;
        var service = new ShellSetupService(() => Task.FromResult<(ShellConfiguration, ShellConfiguration)>((new(), new())),
            _ => { ++writes; return Task.CompletedTask; }, () => Task.FromResult(Network),
            install: async (_, _, token) =>
            {
                started.SetResult();
                try { await Task.Delay(Timeout.Infinite, token); } finally { cleaned = true; }
            }, ensureInstallSupported: () => { });
        service.StartInstallation(true);
        await started.Task.WaitAsync(TimeSpan.FromSeconds(3));
        await Assert.ThrowsAsync<SdkException>(() => service.SaveAsync(new("agent", new("custom", ShellPath))));
        await service.CancelInstallationAsync();
        await service.ShutdownAsync();
        Assert.True(cleaned);
        Assert.Equal(0, writes);
        Assert.Equal("cancelled", service.Status.Stage);
        Assert.False(service.Status.Running);
    }

    [Theory]
    [InlineData(true)]
    [InlineData(false)]
    public async Task OnlySuccessfulInstallationCanCommitBothPreferences(bool succeeds)
    {
        ShellPreferencesRequest? saved = null;
        var service = new ShellSetupService(() => Task.FromResult<(ShellConfiguration, ShellConfiguration)>((new(), new())),
            request => { saved = request; return Task.CompletedTask; }, () => Task.FromResult(Network),
            check: (request, _) => Task.FromResult(new ShellCheckResult(request.Configuration!, ShellPath, "available", "PowerShell 7.6.0", new(ShellPath, "powershell", "PowerShell 7.6.0", ["-NoLogo"]), null)),
            install: (_, _, _) => succeeds ? Task.CompletedTask : Task.FromException(new SdkException("shell_download_integrity_failed", "Checksum mismatch")),
            ensureInstallSupported: () => { });
        service.StartInstallation(true);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(3));
        while (service.Status.Running) await Task.Delay(10, deadline.Token);
        if (succeeds)
        {
            Assert.Equal("installed", service.Status.Stage);
            Assert.NotNull(saved?.Agent?.Resolved);
            Assert.Equal(saved!.Agent, saved.Terminal);
        }
        else { Assert.Equal("failed", service.Status.Stage); Assert.Null(saved); }
        await service.ShutdownAsync();
    }

    [Theory]
    [InlineData("../escape")]
    [InlineData("/absolute")]
    [InlineData("C:/drive")]
    [InlineData("foo\\bar")]
    [InlineData("NUL.txt")]
    [InlineData("CONIN$")]
    [InlineData("dir/file.")]
    [InlineData("dir/file ")]
    [InlineData("dir/./file")]
    public async Task InstallerRejectsUnsafeWindowsArchivePaths(string name)
    {
        using var directory = new TemporaryDirectory();
        var zip = Path.Combine(directory.Path, "input.zip");
        using (var archive = ZipFile.Open(zip, ZipArchiveMode.Create)) archive.CreateEntry(name);
        var error = await Assert.ThrowsAsync<SdkException>(() => PowerShellInstaller.ExtractAsync(zip, Path.Combine(directory.Path, "runtime"), default));
        Assert.Equal("shell_install_invalid", error.Code);
    }

    [Fact]
    public async Task InstallerRejectsSymlinksAndCaseInsensitiveDuplicates()
    {
        using var directory = new TemporaryDirectory();
        foreach (var symlink in new[] { true, false })
        {
            var zip = Path.Combine(directory.Path, $"{symlink}.zip");
            using (var archive = ZipFile.Open(zip, ZipArchiveMode.Create))
            {
                var entry = archive.CreateEntry("pwsh.exe");
                if (symlink) entry.ExternalAttributes = 0xA000 << 16;
                else archive.CreateEntry("PWSH.EXE");
            }
            await Assert.ThrowsAsync<SdkException>(() => PowerShellInstaller.ExtractAsync(zip, Path.Combine(directory.Path, $"runtime-{symlink}"), default));
        }
    }

    [Fact]
    public async Task InstallerExtractsAValidArchiveAndHonorsCancellation()
    {
        using var directory = new TemporaryDirectory();
        var zip = Path.Combine(directory.Path, "input.zip");
        using (var archive = ZipFile.Open(zip, ZipArchiveMode.Create))
        {
            using var writer = new StreamWriter(archive.CreateEntry("runtime/pwsh.exe").Open());
            writer.Write("verified bytes");
        }
        var target = Path.Combine(directory.Path, "output");
        await PowerShellInstaller.ExtractAsync(zip, target, default);
        Assert.Equal("verified bytes", await File.ReadAllTextAsync(Path.Combine(target, "runtime", "pwsh.exe")));
        using var cancellation = new CancellationTokenSource(); cancellation.Cancel();
        await Assert.ThrowsAnyAsync<OperationCanceledException>(() => PowerShellInstaller.ExtractAsync(zip, Path.Combine(directory.Path, "cancelled"), cancellation.Token));
    }

    [Fact]
    public void InstallerVerifiesReleaseIntegrityAndUsesTheConfiguredProxy()
    {
        var digest = SHA256.HashData("archive"u8);
        PowerShellInstaller.VerifyDigest(digest, Convert.ToHexString(digest));
        Assert.Throws<SdkException>(() => PowerShellInstaller.VerifyDigest(digest, new string('0', 64)));
        using var handler = PowerShellInstaller.CreateHandler(Network with { Proxy = new("custom", "http://localhost:8080", "user", "secret", [".example.com", "192.168.0.0/16"]) });
        Assert.Equal(new Uri("http://localhost:8080"), handler.Proxy!.GetProxy(new Uri("https://github.com")));
        Assert.True(handler.Proxy.IsBypassed(new Uri("https://example.com")));
        Assert.True(handler.Proxy.IsBypassed(new Uri("https://sub.example.com")));
        Assert.True(handler.Proxy.IsBypassed(new Uri("https://192.168.1.1")));
        Assert.False(handler.Proxy.IsBypassed(new Uri("https://notexample.com")));
        Assert.DoesNotContain("secret", (Network with { Proxy = new("custom", "http://localhost:8080", "user", "secret", []) }).ToString());
    }

    [Fact]
    public async Task DetectionUsesTheDesktopTimeoutAndBoundsOutput()
    {
        if (OperatingSystem.IsWindows()) return;
        using var directory = new TemporaryDirectory();
        var executable = Path.Combine(directory.Path, "sh");
        await File.WriteAllTextAsync(executable, "#!/bin/sh\nexec /bin/sleep 30\n");
        File.SetUnixFileMode(executable, UnixFileMode.UserRead | UnixFileMode.UserWrite | UnixFileMode.UserExecute);
        var timedOut = await ShellResolver.CheckAsync(new("agent", new("custom", executable)));
        Assert.Equal("shell_probe_timeout", timedOut.ErrorCode);
        await File.WriteAllTextAsync(executable, "#!/bin/sh\n/usr/bin/yes x | /usr/bin/head -c 20000\n");
        var excessive = await ShellResolver.CheckAsync(new("agent", new("custom", executable)));
        Assert.Equal("shell_probe_failed", excessive.ErrorCode);
    }

    private sealed class TemporaryDirectory : IDisposable
    {
        public string Path { get; } = System.IO.Path.Combine(System.IO.Path.GetTempPath(), "suncode-shell-test-" + Guid.NewGuid().ToString("N"));
        public TemporaryDirectory() => Directory.CreateDirectory(Path);
        public void Dispose() => Directory.Delete(Path, true);
    }
}
