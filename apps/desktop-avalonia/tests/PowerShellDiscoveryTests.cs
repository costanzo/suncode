using SunCode.Desktop.Infrastructure;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Tests;

public sealed class PowerShellDiscoveryTests
{
    [Fact]
    public void SearchesAbsolutePathEntriesAndCommonInstallationsWithoutDuplicates()
    {
        var root = Path.GetTempPath();
        var portable = Path.Combine(root, "portable");
        var programFiles = Path.Combine(root, "program-files");
        var local = Path.Combine(root, "local");
        var environment = new Dictionary<string, string>
        {
            ["PATH"] = $";relative;\"{portable}\";{portable.ToUpperInvariant()}",
            ["ProgramW6432"] = programFiles, ["ProgramFiles"] = programFiles,
            ["LOCALAPPDATA"] = local
        };
        var candidates = PowerShellDiscovery.Candidates(key => environment.GetValueOrDefault(key)).ToArray();
        Assert.Equal(Path.Combine(portable, "pwsh.exe"), candidates[0]);
        Assert.Contains(Path.Combine(programFiles, "PowerShell", "7", "pwsh.exe"), candidates);
        Assert.Contains(Path.Combine(local, "Programs", "PowerShell", "7", "pwsh.exe"), candidates);
        Assert.Contains(Path.Combine(local, "Microsoft", "WindowsApps", "pwsh.exe"), candidates);
        Assert.All(candidates, path => Assert.True(Path.IsPathFullyQualified(path)));
        Assert.Equal(candidates.Length, candidates.Distinct(StringComparer.OrdinalIgnoreCase).Count());
    }

    [Fact]
    public async Task SkipsMissingBrokenAndOldExecutablesAndReusesTheFirstWorkingVersion()
    {
        var directory = Path.Combine(Path.GetTempPath(), "suncode-discovery-" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(directory);
        try
        {
            var paths = new[] { "missing", "broken", "old", "working", "later" }.Select(name => Path.Combine(directory, name, "pwsh.exe")).ToArray();
            foreach (var path in paths.Skip(1)) { Directory.CreateDirectory(Path.GetDirectoryName(path)!); await File.WriteAllTextAsync(path, "fixture"); }
            var checkedPaths = new List<string>();
            var result = await PowerShellDiscovery.FindAsync([paths[0], paths[1], paths[1], .. paths.Skip(2)], (request, _) =>
            {
                var path = request.Configuration!.Path;
                checkedPaths.Add(path);
                return Task.FromResult(path == paths[1]
                    ? new ShellCheckResult(request.Configuration, path, "unavailable", "Failed", null, "shell_probe_failed")
                    : Available(path, path == paths[2] ? "PowerShell 5.1" : "PowerShell 7.4.6"));
            }, default);
            Assert.Equal(paths[3], result?.Shell?.Path);
            Assert.Equal(new[] { paths[1], paths[2], paths[3] }, checkedPaths);
        }
        finally { Directory.Delete(directory, true); }
    }

    [Fact]
    public async Task CancellationDoesNotBecomeAMissingInstallation()
    {
        using var cancellation = new CancellationTokenSource();
        cancellation.Cancel();
        await Assert.ThrowsAnyAsync<OperationCanceledException>(() => PowerShellDiscovery.FindAsync([], (_, _) => throw new InvalidOperationException(), cancellation.Token));
    }

    [Theory]
    [InlineData("PowerShell 5.1", false)]
    [InlineData("PowerShell invalid", false)]
    [InlineData("PowerShell 7.4.6", true)]
    [InlineData("PowerShell 7.6.0", true)]
    public void AcceptsProbedVersionsWithoutRequiringTheManagedRelease(string version, bool expected) =>
        Assert.Equal(expected, PowerShellDiscovery.IsPowerShell7(Available("pwsh.exe", version)));

    internal static ShellCheckResult Available(string path, string version = "PowerShell 7.4.6") =>
        new(new("custom", path), path, "available", version, new ResolvedShell(path, "powershell", version, ["-NoLogo"]), null);
}
