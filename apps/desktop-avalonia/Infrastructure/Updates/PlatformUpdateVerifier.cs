using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using System.Security.Cryptography.X509Certificates;

namespace SunCode.Desktop.Infrastructure.Updates;

internal interface IUpdatePlatformVerifier
{
    Task VerifyAsync(string directory, bool desktop, CancellationToken token);
}

internal sealed class PlatformUpdateVerifier(UpdateDistribution distribution) : IUpdatePlatformVerifier
{
    public async Task VerifyAsync(string directory, bool desktop, CancellationToken token)
    {
        if (OperatingSystem.IsMacOS())
        {
            if (distribution.MacTeamId is not { Length: 10 } team)
                throw new UpdateException("configuration_invalid", "The macOS update signing identity is not configured.");
            var targets = desktop ? new[] { Path.Combine(directory, "SunCode.app") }
                : Directory.GetFiles(directory, "*.dylib", SearchOption.AllDirectories);
            if (targets.Length == 0) throw Invalid();
            foreach (var path in targets)
            {
                await RunAsync("/usr/bin/codesign", ["--verify", "--deep", "--strict", path], token);
                var identity = await RunAsync("/usr/bin/codesign", ["--display", "--verbose=4", path], token);
                if (!identity.Split('\n').Any(line => line.Trim() == $"TeamIdentifier={team}")) throw Invalid();
                if (desktop) await RunAsync("/usr/sbin/spctl", ["--assess", "--type", "execute", path], token);
            }
        }
        else if (OperatingSystem.IsWindows())
        {
            if (distribution.WindowsPublisherSha256 is not { Length: 64 } expected) throw new UpdateException("configuration_invalid", "The Windows update signing identity is not configured.");
            var file = Path.Combine(directory, desktop ? "SunCode.exe" : "suncode_agent.dll");
            if (!File.Exists(file)) throw Invalid();
            VerifyWindows(file, expected);
        }
        else if (!OperatingSystem.IsLinux()) throw new UpdateException("unsupported_platform", "Updates are unavailable on this platform.");
        // Linux authenticates the immutable archive through the signed catalog;
        // it has no equivalent universal application-signing identity.
    }
    private static async Task<string> RunAsync(string program, string[] arguments, CancellationToken token)
    {
        using var timeout = CancellationTokenSource.CreateLinkedTokenSource(token);
        timeout.CancelAfter(TimeSpan.FromSeconds(30));
        var start = new ProcessStartInfo(program) { UseShellExecute = false, RedirectStandardOutput = true, RedirectStandardError = true, CreateNoWindow = true };
        foreach (var arg in arguments) start.ArgumentList.Add(arg);
        using var process = Process.Start(start) ?? throw Invalid();
        var output = process.StandardOutput.ReadToEndAsync(timeout.Token);
        var error = process.StandardError.ReadToEndAsync(timeout.Token);
        try { await process.WaitForExitAsync(timeout.Token); }
        catch { try { process.Kill(entireProcessTree: true); } catch (InvalidOperationException) { } throw; }
        if (process.ExitCode != 0) throw Invalid();
        return await output + await error;
    }
    private static void VerifyWindows(string path, string publisher)
    {
        var file = new WinTrustFile { Size = (uint)Marshal.SizeOf<WinTrustFile>(), File = path };
        var pointer = Marshal.AllocHGlobal(Marshal.SizeOf<WinTrustFile>());
        try
        {
            Marshal.StructureToPtr(file, pointer, false);
            var data = new WinTrustData { Size = (uint)Marshal.SizeOf<WinTrustData>(), UiChoice = 2, RevocationChecks = 1, UnionChoice = 1, File = pointer };
            var action = new Guid("00AAC56B-CD44-11d0-8CC2-00C04FC295EE");
            if (WinVerifyTrust(new IntPtr(-1), ref action, ref data) != 0) throw Invalid();
#pragma warning disable SYSLIB0057
            using var certificate = new X509Certificate2(X509Certificate.CreateFromSignedFile(path));
#pragma warning restore SYSLIB0057
            if (!string.Equals(certificate.GetCertHashString(HashAlgorithmName.SHA256), publisher, StringComparison.OrdinalIgnoreCase)) throw Invalid();
        }
        finally { Marshal.DestroyStructure<WinTrustFile>(pointer); Marshal.FreeHGlobal(pointer); }
    }
    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    private struct WinTrustFile { public uint Size; [MarshalAs(UnmanagedType.LPWStr)] public string File; public IntPtr Handle; public IntPtr KnownSubject; }
    [StructLayout(LayoutKind.Sequential)]
    private struct WinTrustData
    {
        public uint Size; public IntPtr PolicyCallback; public IntPtr SipClient; public uint UiChoice; public uint RevocationChecks;
        public uint UnionChoice; public IntPtr File; public uint StateAction; public IntPtr StateData; public IntPtr UrlReference;
        public uint ProviderFlags; public uint UiContext; public IntPtr SignatureSettings;
    }
    [DllImport("wintrust.dll", ExactSpelling = true, CharSet = CharSet.Unicode)]
    private static extern int WinVerifyTrust(IntPtr window, ref Guid action, ref WinTrustData data);
    private static UpdateException Invalid() => new("platform_signature_invalid", "The update's platform signature could not be verified.");
}
