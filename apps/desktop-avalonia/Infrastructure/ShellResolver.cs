using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;
using SunCode.Sdk;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Infrastructure;

// Host setup only. Agent commands never enter this service.
internal static class ShellResolver
{
    internal const string PowerShellVersion = "7.6.0";
    internal static string Architecture => RuntimeInformation.OSArchitecture switch
    {
        System.Runtime.InteropServices.Architecture.Arm64 => "arm64",
        System.Runtime.InteropServices.Architecture.X64 => "x64",
        _ => throw new SdkException("shell_architecture_unsupported", "PowerShell installation supports Windows x64 and ARM64")
    };
    internal static string ManagedDirectory
    {
        get
        {
            if (!OperatingSystem.IsWindows()) throw new SdkException("shell_platform_unsupported", "PowerShell installation is available on Windows");
            var local = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
            if (!Path.IsPathFullyQualified(local)) throw new SdkException("shell_install_directory_unavailable", "The local application directory is unavailable");
            return Path.Combine(local, "SunCode", "runtimes", "powershell", $"{PowerShellVersion}-{Architecture}");
        }
    }
    internal static string ReleaseDigest(string architecture) => architecture switch
    {
        "x64" => "9e725837af682b87bb212cd1efe3657c06c540404203810857ec2516ae2ca322",
        "arm64" => "c2ac26385210ebff5b0696bfb26bc74a9867fd9d561a75ea52635e553974f0cb",
        _ => throw new SdkException("shell_architecture_unsupported", "Unsupported PowerShell architecture")
    };
    internal static string SystemPath(bool terminal) => OperatingSystem.IsWindows()
        ? Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.Windows), "System32", "WindowsPowerShell", "v1.0", "powershell.exe")
        : terminal ? Environment.GetEnvironmentVariable("SHELL") is { Length: > 0 } login ? login : LoginShell() ?? "/bin/sh" : "/bin/sh";

    internal static async Task<ShellCheckResult> CheckAsync(ShellCheckRequest request, CancellationToken cancellationToken = default)
    {
        var configuration = (request.Configuration ?? new()) with { Resolved = null };
        if (configuration.Mode == "custom") configuration = configuration with { Path = configuration.Path.Trim() };
        try
        {
            var shell = await ResolveAsync(configuration, request.Target == "terminal", cancellationToken).ConfigureAwait(false);
            return new(configuration, shell.Path, "available", shell.Version, shell, null);
        }
        catch (OperationCanceledException) when (cancellationToken.IsCancellationRequested) { throw; }
        catch (Exception error)
        {
            return new(configuration, configuration.Path, "unavailable", error.Message, null, (error as SdkException)?.Code ?? "shell_probe_failed");
        }
    }
    internal static async Task<ResolvedShell> ResolveAsync(ShellConfiguration configuration, bool terminal, CancellationToken cancellationToken = default)
    {
        if (configuration.Mode == "custom" && (string.IsNullOrWhiteSpace(configuration.Path) || configuration.Path.Length > 4096 || configuration.Path.Any(char.IsControl))
            || configuration.Mode != "custom" && configuration.Path.Length != 0)
            throw new SdkException("invalid_arguments", "Shell configuration is invalid");
        var requested = configuration.Mode switch
        {
            "system" => SystemPath(terminal),
            "custom" => configuration.Path.Trim(),
            "managed" => Path.Combine(ManagedDirectory, "pwsh.exe"),
            _ => throw new SdkException("invalid_arguments", "Shell configuration is invalid")
        };
        if (!Path.IsPathFullyQualified(requested)) throw new SdkException("invalid_arguments", "Shell executable path must be absolute");
        if (configuration.Mode == "managed") ValidateManifest(ManagedDirectory);
        var family = Path.GetFileName(requested).ToLowerInvariant() switch
        {
            "powershell.exe" or "pwsh.exe" when OperatingSystem.IsWindows() => "powershell",
            "pwsh" when !OperatingSystem.IsWindows() => "powershell",
            "sh" or "dash" or "bash" or "zsh" when !OperatingSystem.IsWindows() => Path.GetFileName(requested).ToLowerInvariant(),
            _ => throw new SdkException("shell_unsupported", "Choose a supported PowerShell, sh, bash, or zsh executable")
        };
        if (!File.Exists(requested)) throw new SdkException("shell_executable_not_found", "Shell executable was not found");
        var path = CanonicalPath(requested);
        string[] interactive = family == "powershell" ? ["-NoLogo"] : family == "sh" && Path.GetFileName(path) == "busybox" ? ["sh", "-il"] : ["-il"];
        var shell = new ResolvedShell(path, family, "", interactive);
        var script = family switch
        {
            "powershell" => "Write-Output ('SunCodeShell:PowerShell:' + $PSVersionTable.PSVersion.ToString()); exit 0",
            "bash" => "printf 'SunCodeShell:bash:%s\\n' \"$BASH_VERSION\"",
            "zsh" => "printf 'SunCodeShell:zsh:%s\\n' \"$ZSH_VERSION\"",
            _ => "printf 'SunCodeShell:POSIX sh\\n'"
        };
        var version = await ProbeAsync(shell, script, cancellationToken).ConfigureAwait(false);
        if (configuration.Mode == "managed" && version != $"PowerShell {PowerShellVersion}") throw new SdkException("shell_managed_invalid", "The managed PowerShell version does not match its manifest");
        return shell with { Version = version };
    }
    internal static string[] CommandArguments(ResolvedShell shell, string script) => shell.Family switch
    {
        "powershell" => ["-NoLogo", "-NoProfile", "-NonInteractive", "-Command", "[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false); $OutputEncoding = [Console]::OutputEncoding; " + script],
        "zsh" => ["-f", "-c", script],
        "sh" when Path.GetFileName(shell.Path) == "busybox" => ["sh", "-c", script],
        _ => ["-c", script]
    };
    private static async Task<string> ProbeAsync(ResolvedShell shell, string script, CancellationToken cancellationToken)
    {
        using var timeout = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
        timeout.CancelAfter(TimeSpan.FromSeconds(5));
        var start = new ProcessStartInfo(shell.Path) { UseShellExecute = false, CreateNoWindow = true, RedirectStandardInput = true, RedirectStandardOutput = true, RedirectStandardError = true, WorkingDirectory = Path.GetTempPath(), StandardOutputEncoding = Encoding.UTF8 };
        var inherited = start.Environment.ToDictionary();
        start.Environment.Clear();
        foreach (var key in new[] { "PATH", "HOME", "USER", "LOGNAME", "LANG", "LC_ALL", "TMPDIR", "TEMP", "TMP", "SystemRoot", "WINDIR", "USERPROFILE", "LOCALAPPDATA", "APPDATA", "ProgramFiles", "ProgramFiles(x86)", "COMSPEC", "PATHEXT" })
            if (inherited.TryGetValue(key, out var value)) start.Environment[key] = value;
        foreach (var arg in CommandArguments(shell, script)) start.ArgumentList.Add(arg);
        using var process = new Process { StartInfo = start };
        try
        {
            if (!process.Start()) throw new SdkException("shell_probe_failed", "Cannot start the shell");
            process.StandardInput.Close();
            using var stop = timeout.Token.Register(() =>
            {
                try { process.Kill(entireProcessTree: true); } catch (InvalidOperationException) { } catch (System.ComponentModel.Win32Exception) { }
            });
            var output = ReadBoundedAsync(process.StandardOutput, timeout.Token);
            var errors = ReadBoundedAsync(process.StandardError, timeout.Token);
            await Task.WhenAll(process.WaitForExitAsync(timeout.Token), output, errors).ConfigureAwait(false);
            if (process.ExitCode != 0) throw new SdkException("shell_probe_failed", "Shell validation failed");
            var identity = shell.Family switch { "powershell" => "PowerShell:", "bash" => "bash:", "zsh" => "zsh:", _ => "POSIX sh" };
            var line = (await output).Split('\n').Select(value => value.TrimEnd('\r')).FirstOrDefault(value => value.StartsWith("SunCodeShell:" + identity, StringComparison.Ordinal));
            if (line is null || line.Length > 141 || line.Any(char.IsControl)) throw new SdkException("shell_probe_failed", "The executable did not return a supported shell identity");
            var version = line[13..].Replace(':', ' ');
            if (identity != "POSIX sh" && (version.Length <= identity.Length || !char.IsAsciiDigit(version[identity.Length]))) throw new SdkException("shell_probe_failed", "The executable did not identify a supported shell version");
            return version;
        }
        catch (OperationCanceledException) when (!cancellationToken.IsCancellationRequested) { throw new SdkException("shell_probe_timeout", "The shell did not respond within five seconds"); }
        finally
        {
            try { process.Kill(entireProcessTree: true); } catch (InvalidOperationException) { } catch (System.ComponentModel.Win32Exception) { }
        }
    }
    private static async Task<string> ReadBoundedAsync(StreamReader reader, CancellationToken token)
    {
        var text = new StringBuilder(); var buffer = new char[1024];
        while (true)
        {
            var count = await reader.ReadAsync(buffer.AsMemory(), token).ConfigureAwait(false);
            if (count == 0) return text.ToString();
            if (text.Length + count > 16 * 1024) throw new SdkException("shell_probe_failed", "Shell output exceeded the detection limit");
            text.Append(buffer, 0, count);
        }
    }
    private static void ValidateManifest(string directory)
    {
        var path = Path.Combine(directory, "suncode-runtime.json");
        if (!File.Exists(path) || new FileInfo(path).Length > 4096) throw new SdkException("shell_managed_invalid", "The managed installation is incomplete. Download and install it again");
        using var marker = JsonDocument.Parse(File.ReadAllText(path));
        if (marker.RootElement.GetProperty("version").GetString() != PowerShellVersion || marker.RootElement.GetProperty("architecture").GetString() != Architecture || marker.RootElement.GetProperty("sha256").GetString() != ReleaseDigest(Architecture))
            throw new SdkException("shell_managed_invalid", "The managed installation does not match the expected release");
    }
    private static string CanonicalPath(string path)
    {
        if (OperatingSystem.IsWindows()) return new FileInfo(path).ResolveLinkTarget(true)?.FullName ?? Path.GetFullPath(path);
        var pointer = RealPath(path, IntPtr.Zero);
        if (pointer == IntPtr.Zero) throw new SdkException("shell_executable_not_found", "Shell executable was not found");
        try { return Marshal.PtrToStringUTF8(pointer)!; } finally { Free(pointer); }
    }
    internal static string? LoginShell()
    {
        // Reentrant lookup; passwd field offsets differ between Darwin and Linux.
        var entry = Marshal.AllocHGlobal(128); var buffer = Marshal.AllocHGlobal(16 * 1024);
        try
        {
            if (GetPwUid(GetUid(), entry, buffer, 16 * 1024, out var result) != 0 || result == IntPtr.Zero) return null;
            return Marshal.PtrToStringUTF8(Marshal.ReadIntPtr(result, OperatingSystem.IsMacOS() ? 56 : 40));
        }
        finally { Marshal.FreeHGlobal(entry); Marshal.FreeHGlobal(buffer); }
    }
    [DllImport("libc", EntryPoint = "getuid")] private static extern uint GetUid();
    [DllImport("libc", EntryPoint = "getpwuid_r")] private static extern int GetPwUid(uint uid, IntPtr entry, IntPtr buffer, nuint length, out IntPtr result);
    [DllImport("libc", EntryPoint = "realpath", SetLastError = true)] private static extern IntPtr RealPath([MarshalAs(UnmanagedType.LPUTF8Str)] string path, IntPtr buffer);
    [DllImport("libc", EntryPoint = "free")] private static extern void Free(IntPtr pointer);
}
