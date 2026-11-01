using System.Runtime.InteropServices;

namespace SunCode.Desktop.Infrastructure.Updates;

internal sealed record UpdateEnvironment(string Target, string OsVersion, string? GlibcVersion)
{
    public static UpdateEnvironment Current()
    {
        var os = OperatingSystem.IsWindows() ? "win" : OperatingSystem.IsMacOS() ? "osx" : OperatingSystem.IsLinux() ? "linux" : "unknown";
        var architecture = RuntimeInformation.ProcessArchitecture switch { Architecture.X64 => "x64", Architecture.Arm64 => "arm64", _ => "unknown" };
        string? glibc = null;
        if (OperatingSystem.IsLinux())
        {
            try { glibc = Marshal.PtrToStringAnsi(GnuGetLibcVersion()); }
            catch (Exception e) when (e is DllNotFoundException or EntryPointNotFoundException) { }
        }
        return new($"{os}-{architecture}", Environment.OSVersion.Version.ToString(), glibc);
    }
    [DllImport("libc", EntryPoint = "gnu_get_libc_version")]
    private static extern IntPtr GnuGetLibcVersion();
}

internal static class UpdatePlanner
{
    public static UpdatePlan? Select(VerifiedCatalog verified, UpdateInstallation installed, UpdateEnvironment environment, bool sdkHealthy)
    {
        var c = verified.Catalog;
        if (installed.Target != environment.Target || c.Target != environment.Target) throw new UpdateException("target_mismatch", "The update does not match this installation.");
        var d = c.Components.Desktop; var s = c.Components.Sdk;
        CheckIdentity(installed.Desktop, d, d.RequiredSdkAbi);
        if (installed.Sdk is { } localSdk) CheckIdentity(localSdk, s, s.Abi);
        var candidates = new List<UpdatePlan>();
        foreach (var changeDesktop in new[] { false, true })
        foreach (var changeSdk in new[] { false, true })
        {
            if (!changeDesktop && !changeSdk) continue;
            if (!changeSdk && (!sdkHealthy || installed.Sdk is null)) continue;
            var desktopVersion = changeDesktop ? d.Version : installed.Desktop.Version;
            var sdkVersion = changeSdk ? s.Version : installed.Sdk!.Version;
            if (changeDesktop && ReleaseVersion.Parse(d.Version).CompareTo(ReleaseVersion.Parse(installed.Desktop.Version)) <= 0) continue;
            if (changeSdk && installed.Sdk is { } existing)
            {
                var compare = ReleaseVersion.Parse(s.Version).CompareTo(ReleaseVersion.Parse(existing.Version));
                if (compare < 0 || (compare == 0 && sdkHealthy)) continue;
            }
            if (changeDesktop && !Supports(d.Package, environment) || changeSdk && !Supports(s.Package, environment)) continue;
            if (!c.CompatiblePairs.Contains(new CompatiblePair { Desktop = desktopVersion, Sdk = sdkVersion })) continue;
            var desktopAbi = changeDesktop ? d.RequiredSdkAbi : installed.Desktop.Abi;
            var sdkAbi = changeSdk ? s.Abi : installed.Sdk!.Abi;
            if (desktopAbi != sdkAbi) continue;
            candidates.Add(new(verified, installed, changeDesktop, changeSdk, changeSdk && !sdkHealthy));
        }
        var plan = candidates.OrderByDescending(p => ReleaseVersion.Parse(p.DesktopVersion)).ThenByDescending(p => ReleaseVersion.Parse(p.SdkVersion)).FirstOrDefault();
        if (plan is not null) return plan;
        if (sdkHealthy && installed.Desktop.Version == d.Version && installed.Sdk?.Version == s.Version) return null;
        throw new UpdateException("no_compatible_update", "No compatible update is available for this installation.");
    }
    private static void CheckIdentity(InstalledComponent installed, ComponentRelease release, int abi)
    {
        if (installed.Version == release.Version && (installed.ArchiveSha256 != release.Package.Sha256 || installed.Abi != abi))
            throw new UpdateException("release_conflict", "A published release does not match its installed identity.");
    }
    internal static bool Supports(UpdatePackage package, UpdateEnvironment environment)
        => (package.MinimumOsVersion is null || AtLeast(environment.OsVersion, package.MinimumOsVersion))
            && (package.MinimumGlibcVersion is null || environment.GlibcVersion is { } glibc && AtLeast(glibc, package.MinimumGlibcVersion));
    private static bool AtLeast(string actual, string required)
    {
        var a = actual.Split('.'); var b = required.Split('.');
        for (var i = 0; i < Math.Max(a.Length, b.Length); i++)
        {
            if (!ulong.TryParse(i < a.Length ? a[i] : "0", out var av) || !ulong.TryParse(i < b.Length ? b[i] : "0", out var bv)) return false;
            if (av != bv) return av > bv;
        }
        return true;
    }
    internal static bool Healthy(InstalledComponent? component)
    {
        if (component?.Files is not { Count: > 0 }) return false;
        try
        {
            foreach (var relative in component.Files.Keys) _ = UpdateArchive.Inside(component.Directory, relative);
            var actual = UpdateArchive.Manifest(component.Directory);
            return actual.Count == component.Files.Count && component.Files.All(pair => actual.GetValueOrDefault(pair.Key) == pair.Value);
        }
        catch (Exception e) when (e is IOException or UnauthorizedAccessException or UpdateException) { return false; }
    }
}
