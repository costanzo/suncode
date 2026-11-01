namespace SunCode.Desktop.Infrastructure.Updates;

internal sealed class UpdateStager(UpdateTransport transport, UpdateStateStore store, IUpdatePlatformVerifier platform)
{
    public async Task<StagedUpdate> StageAsync(UpdatePlan plan, Action<long, long>? progress, Action? verifying, CancellationToken token)
    {
        var root = Path.Combine(store.DirectoryPath, "staging", Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(root);
        var completed = 0L;
        try
        {
            foreach (var (id, release) in new (string, ComponentRelease)[] { ("desktop", plan.Verified.Catalog.Components.Desktop), ("sdk", plan.Verified.Catalog.Components.Sdk) })
            {
                if (id == "desktop" && !plan.Desktop || id == "sdk" && !plan.Sdk) continue;
                var archive = Path.Combine(root, id + ".archive");
                var previous = completed;
                await transport.DownloadAsync(release.Package, archive, count => progress?.Invoke(previous + count, plan.SizeBytes), token);
                completed += release.Package.SizeBytes;
            }
            verifying?.Invoke();
            foreach (var (id, release) in new (string, ComponentRelease)[] { ("desktop", plan.Verified.Catalog.Components.Desktop), ("sdk", plan.Verified.Catalog.Components.Sdk) })
            {
                if (id == "desktop" && !plan.Desktop || id == "sdk" && !plan.Sdk) continue;
                var extracted = Path.Combine(root, id);
                await UpdateArchive.ExtractAsync(Path.Combine(root, id + ".archive"), release.Package.Format, extracted, token);
                ValidateLayout(extracted, id == "desktop");
                await platform.VerifyAsync(extracted, id == "desktop", token);
            }
            token.ThrowIfCancellationRequested();
            var staged = new StagedUpdate(plan.Verified.Envelope, plan.Desktop, plan.Sdk, plan.Repair, root, plan.Installed);
            store.Change(state => state.Staged = staged);
            return staged;
        }
        catch { if (Directory.Exists(root)) Directory.Delete(root, recursive: true); throw; }
    }
    internal static string DesktopPayload(string root) => OperatingSystem.IsMacOS() ? Path.Combine(root, "SunCode.app") : root;
    internal static string DesktopExecutableDirectory(string root) => OperatingSystem.IsMacOS() ? Path.Combine(root, "Contents", "MacOS") : root;
    internal static string SdkLibraryName => OperatingSystem.IsWindows() ? "suncode_agent.dll" : OperatingSystem.IsMacOS() ? "libsuncode_agent.dylib" : "libsuncode_agent.so";
    internal static void ValidateLayout(string root, bool desktop)
    {
        if (desktop)
        {
            var files = DesktopExecutableDirectory(DesktopPayload(root));
            if (!File.Exists(Path.Combine(files, OperatingSystem.IsWindows() ? "SunCode.exe" : "SunCode"))
                || !File.Exists(Path.Combine(files, "SunCode.dll")) || Directory.EnumerateFiles(root, SdkLibraryName, SearchOption.AllDirectories).Any())
                throw new UpdateException("package_layout", "The Desktop update package has an invalid layout.");
            if (OperatingSystem.IsMacOS() && (Directory.GetFileSystemEntries(root).Length != 1 || !Directory.Exists(DesktopPayload(root))))
                throw new UpdateException("package_layout", "The Desktop update package has an invalid layout.");
        }
        else if (!File.Exists(Path.Combine(root, SdkLibraryName)) || File.Exists(Path.Combine(root, "SunCode.dll")))
            throw new UpdateException("package_layout", "The Agent SDK update package has an invalid layout.");
    }
}
