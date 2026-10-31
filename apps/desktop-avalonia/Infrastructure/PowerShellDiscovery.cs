namespace SunCode.Desktop.Infrastructure;

// Optional host discovery. Never changes preferences or launches a downloader.
internal static class PowerShellDiscovery
{
    internal static Task<ShellCheckResult?> FindAsync(CancellationToken token) => OperatingSystem.IsWindows()
        ? FindAsync(Candidates(Environment.GetEnvironmentVariable), ShellResolver.CheckAsync, token)
        : Task.FromResult<ShellCheckResult?>(null);

    internal static IEnumerable<string> Candidates(Func<string, string?> environment)
    {
        var paths = new List<string>();
        foreach (var entry in (environment("PATH") ?? "").Split(';'))
            Add(entry.Trim().Trim('"'), "pwsh.exe");
        // ProgramW6432 also finds the native installation from an emulated host.
        foreach (var variable in new[] { "ProgramW6432", "ProgramFiles", "ProgramFiles(x86)" })
            Add(environment(variable), "PowerShell", "7", "pwsh.exe");
        Add(environment("LOCALAPPDATA"), "Programs", "PowerShell", "7", "pwsh.exe");
        Add(environment("LOCALAPPDATA"), "Microsoft", "WindowsApps", "pwsh.exe");
        return paths.Distinct(StringComparer.OrdinalIgnoreCase);

        void Add(string? directory, params string[] suffix)
        {
            if (string.IsNullOrWhiteSpace(directory) || !Path.IsPathFullyQualified(directory)
                || directory.Any(char.IsControl) || directory.Length > 4096) return;
            paths.Add(Path.Combine([directory, .. suffix]));
        }
    }

    internal static bool IsPowerShell7(ShellCheckResult? result) => result is
        { State: "available", Shell.Family: "powershell" }
        && result.Shell.Version.StartsWith("PowerShell ", StringComparison.Ordinal)
        && Version.TryParse(result.Shell.Version[11..], out var version) && version.Major >= 7;

    internal static async Task<ShellCheckResult?> FindAsync(IEnumerable<string> candidates,
        Func<ShellCheckRequest, CancellationToken, Task<ShellCheckResult>> check, CancellationToken token)
    {
        using var deadline = CancellationTokenSource.CreateLinkedTokenSource(token);
        deadline.CancelAfter(TimeSpan.FromSeconds(5));
        try
        {
            foreach (var path in candidates.Distinct(StringComparer.OrdinalIgnoreCase))
            {
                deadline.Token.ThrowIfCancellationRequested();
                if (!File.Exists(path)) continue;
                var result = await check(new("terminal", new("custom", path)), deadline.Token).ConfigureAwait(false);
                if (!IsPowerShell7(result)) continue;
                return result;
            }
        }
        catch (OperationCanceledException) when (!token.IsCancellationRequested) { }
        token.ThrowIfCancellationRequested();
        return null;
    }
}
