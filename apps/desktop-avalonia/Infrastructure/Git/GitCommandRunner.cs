using System.ComponentModel;
using System.Diagnostics;
using System.Text;
using SunCode.Desktop.Infrastructure;

namespace SunCode.Desktop.Infrastructure.Git;

internal sealed record GitCommandResult(int ExitCode, byte[] Output, string Error, bool Truncated)
{
    public string Text => Encoding.UTF8.GetString(Output);
}

// Only fixed read-only Git requests reach this runner. No shell or generic process API.
internal sealed class GitCommandRunner(string executable)
{
    internal string Executable { get; } = Path.GetFullPath(executable);
    private static readonly string[] Configuration = [
        "--no-pager", "--literal-pathspecs", "-c", "color.ui=false", "-c", "core.fsmonitor=false",
        "-c", "core.untrackedCache=false", "-c", "diff.external=", "-c", "diff.algorithm=myers"
    ];

    public async Task<GitCommandResult> RunAsync(string directory, IEnumerable<string> arguments,
        CancellationToken cancellation = default, int outputLimit = 8 * 1024 * 1024,
        TimeSpan? timeout = null, bool allowDifference = false)
    {
        cancellation.ThrowIfCancellationRequested();
        if (!File.Exists(Executable)) throw new GitException("git_missing", "The Git executable is unavailable.");
        if (!Directory.Exists(directory)) throw new GitException("project_unavailable", "The project directory is unavailable.");
        using var deadline = CancellationTokenSource.CreateLinkedTokenSource(cancellation);
        deadline.CancelAfter(timeout ?? TimeSpan.FromSeconds(30));
        var start = new ProcessStartInfo(Executable)
        {
            WorkingDirectory = directory, UseShellExecute = false, CreateNoWindow = true,
            RedirectStandardOutput = true, RedirectStandardError = true, RedirectStandardInput = true
        };
        var commandArguments = Configuration.Concat(arguments).ToArray();
        foreach (var argument in commandArguments) start.ArgumentList.Add(argument);
        // A host's Git overrides must not redirect an opened project's query into another repository.
        foreach (var name in start.Environment.Keys.Where(key => key.StartsWith("GIT_", StringComparison.OrdinalIgnoreCase)).ToArray())
            start.Environment.Remove(name);
        start.Environment["GIT_OPTIONAL_LOCKS"] = "0";
        start.Environment["GIT_TERMINAL_PROMPT"] = "0";
        start.Environment["GIT_PAGER"] = "cat";
        start.Environment["LC_ALL"] = "C";
        using var process = new Process { StartInfo = start };
        try
        {
            process.Start();
            process.StandardInput.Close();
            using var stop = deadline.Token.Register(() =>
            {
                try { if (!process.HasExited) process.Kill(entireProcessTree: true); }
                catch (InvalidOperationException) { }
                catch (Win32Exception) { }
            });
            var outputTask = DrainAsync(process.StandardOutput.BaseStream, outputLimit, deadline.Token);
            var errorTask = DrainAsync(process.StandardError.BaseStream, 16 * 1024, deadline.Token);
            // Observe both readers even on cancellation; no pipe can block the other pipe.
            await Task.WhenAll(outputTask, errorTask, process.WaitForExitAsync(deadline.Token));
            var output = await outputTask;
            var error = await errorTask;
            var errorText = Encoding.UTF8.GetString(error.Bytes);
            var result = new GitCommandResult(process.ExitCode, output.Bytes, errorText, output.Truncated);
            var expectedDifference = allowDifference && result.ExitCode == 1;
            if ((!expectedDifference && result.ExitCode != 0) || !string.IsNullOrWhiteSpace(errorText) || output.Truncated || error.Truncated)
            {
                var level = result.ExitCode == 0 ? "warning" : "failure";
                DiagnosticLog.Warn("git.command",
                    $"outcome={level} executable={Sanitize(Executable)} directory={Sanitize(directory)} "
                    + $"args={Sanitize(string.Join(' ', commandArguments))} exit_code={result.ExitCode} "
                    + $"stdout_truncated={output.Truncated} stderr_truncated={error.Truncated} "
                    + $"stderr={Sanitize(errorText, 4096)}");
            }
            return result;
        }
        catch (OperationCanceledException) when (!cancellation.IsCancellationRequested)
        {
            DiagnosticLog.Warn("git.command",
                $"outcome=timeout executable={Sanitize(Executable)} directory={Sanitize(directory)} "
                + $"args={Sanitize(string.Join(' ', commandArguments))}");
            throw new GitException("timeout", "Git did not respond in time. Refresh to retry.");
        }
        catch (Win32Exception exception)
        {
            DiagnosticLog.Error("git.command", exception,
                $"outcome=start_failed executable={Sanitize(Executable)} directory={Sanitize(directory)} "
                + $"args={Sanitize(string.Join(' ', commandArguments))}");
            throw new GitException(File.Exists(Executable) ? "read_failed" : "git_missing",
                "The Git query could not start. Refresh to retry.");
        }
    }

    private static async Task<(byte[] Bytes, bool Truncated)> DrainAsync(Stream stream, int limit, CancellationToken cancellation)
    {
        using var retained = new MemoryStream();
        var buffer = new byte[8192];
        var truncated = false;
        int read;
        while ((read = await stream.ReadAsync(buffer, cancellation)) != 0)
        {
            var keep = Math.Min(read, Math.Max(0, limit - (int)retained.Length));
            retained.Write(buffer, 0, keep);
            truncated |= keep < read;
        }
        return (retained.ToArray(), truncated);
    }

    internal static IEnumerable<string> Candidates()
    {
        var paths = (Environment.GetEnvironmentVariable("PATH") ?? "").Split(Path.PathSeparator);
        var name = OperatingSystem.IsWindows() ? "git.exe" : "git";
        foreach (var directory in paths.Where(Path.IsPathFullyQualified))
        {
            var candidate = Path.Combine(directory, name);
            // Apple's /usr/bin/git is a developer-tools installer shim. Probe real tools instead.
            if (OperatingSystem.IsMacOS() && candidate == "/usr/bin/git") continue;
            yield return candidate;
        }
        if (OperatingSystem.IsWindows())
        {
            foreach (var root in new[] { Environment.GetFolderPath(Environment.SpecialFolder.ProgramFiles),
                         Environment.GetFolderPath(Environment.SpecialFolder.ProgramFilesX86),
                         Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Programs") })
                if (!string.IsNullOrEmpty(root)) yield return Path.Combine(root, "Git", "cmd", "git.exe");
        }
        else
        {
            foreach (var candidate in new[] { "/opt/homebrew/bin/git", "/usr/local/bin/git", "/usr/bin/git", "/bin/git" })
                if (!OperatingSystem.IsMacOS() || candidate is not ("/usr/bin/git" or "/bin/git")) yield return candidate;
            if (OperatingSystem.IsMacOS())
            {
                yield return "/Library/Developer/CommandLineTools/usr/bin/git";
                yield return "/Applications/Xcode.app/Contents/Developer/usr/bin/git";
            }
        }
    }

    private static string Sanitize(string? value, int maxLength = 512)
    {
        if (string.IsNullOrEmpty(value)) return "(none)";
        var normalized = value.Replace('\r', ' ').Replace('\n', ' ').Replace('\t', ' ');
        return normalized.Length <= maxLength ? normalized : normalized[..maxLength] + "...";
    }
}
