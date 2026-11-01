using System.Reflection;
using SunCode.Desktop.Models;
using SunCode.Sdk;

namespace SunCode.Desktop.Infrastructure.Updates;

internal static class UpdateRuntime
{
    private static UpdateCoordinator? _service;
    public static UpdateCoordinator Service => _service ??= Create();
    internal static string ApplicationRoot
    {
        get
        {
            var path = Path.GetFullPath(AppContext.BaseDirectory).TrimEnd(Path.DirectorySeparatorChar);
            return OperatingSystem.IsMacOS() && Path.GetFileName(path) == "MacOS" && Path.GetFileName(Path.GetDirectoryName(path)) == "Contents"
                ? Path.GetDirectoryName(Path.GetDirectoryName(path))! : path;
        }
    }
    private static UpdateCoordinator Create()
    {
        var distribution = new UpdateDistribution();
        try
        {
            var path = Path.Combine(AppContext.BaseDirectory, "update-distribution.json");
            if (File.Exists(path))
            {
                if (new FileInfo(path).Length > 65536) throw new InvalidDataException();
                distribution = UpdateJson.Read<UpdateDistribution>(File.ReadAllBytes(path));
            }
        }
        catch (Exception e) when (e is IOException or System.Text.Json.JsonException) { }
        return new(distribution, new UpdateStateStore(Path.Combine(AppDataPaths.DataDirectory, "desktop-updates")), UpdateEnvironment.Current());
    }

    // True means this process performed helper/routing work and must not initialize
    // Avalonia, instance IPC, native providers, or Agent SQLite.
    public static bool HandleBootstrap(string[] args)
    {
        var mode = args.FirstOrDefault(a => a.StartsWith("--suncode-update-", StringComparison.Ordinal));
        try
        {
            if (mode == "--suncode-update-apply")
            {
                var index = Array.IndexOf(args, mode);
                if (index + 1 >= args.Length) throw new InvalidDataException();
                Service.Activation.RunHelperAsync(args[index + 1], CancellationToken.None).GetAwaiter().GetResult();
                return true;
            }
            if (mode == "--suncode-update-preflight")
            {
                var index = Array.IndexOf(args, mode);
                if (index + 1 >= args.Length) throw new InvalidDataException();
                Service.Activation.PreflightAsync(args[index + 1], PreflightIdentityAsync, CancellationToken.None).GetAwaiter().GetResult();
                Service.ReloadInstallation();
                Service.Activation.RecoverInterrupted();
                return false;
            }
            if (mode is not null) throw new InvalidDataException("Unsupported update helper mode.");
            Service.Activation.RecoverInterrupted();
            var installed = Service.Installation;
            if (installed is null) return false; // Unmanaged development build: normal bundled probing.
            if (!UpdatePlanner.Healthy(installed.Desktop)) throw new UpdateException("desktop_damaged", "The Desktop installation requires repair.");
            if (!string.Equals(Path.GetFullPath(installed.Desktop.Directory), ApplicationRoot, OperatingSystem.IsWindows() ? StringComparison.OrdinalIgnoreCase : StringComparison.Ordinal))
            {
                using var forwarded = UpdateActivation.Start(UpdateActivation.Executable(installed.Desktop.Directory), args);
                return true;
            }
            ConfigureSdk(installed);
            return false;
        }
        catch (Exception e)
        {
            if (mode is null && e is UpdateException { Code: "update_locked" }) return true;
            if (mode is not null)
            {
                Environment.ExitCode = 1;
                try
                {
                    var store = new UpdateStateStore(Path.Combine(AppDataPaths.DataDirectory, "desktop-updates"));
                    var index = Array.IndexOf(args, mode);
                    var id = index >= 0 && index + 1 < args.Length ? args[index + 1] : null;
                    if (id is not null && Guid.TryParseExact(id, "N", out _))
                        store.Change(s => { if (s.Activation?.Id == id && s.Activation.Phase != "committed") { s.Activation = null; s.LastResult = "install_failed"; } });
                }
                catch (Exception) { }
                return true;
            }
            Service.StartupError = e is UpdateException update ? update.Code : "state_invalid";
            Service.RecordSdkHealth(false);
            Service.RecordSdkRuntimeAvailability(false);
            // Do not silently fall back to a bundled old runtime after a managed
            // activation, as it may not support the current database schema.
            AgentSdk.ConfigureNativeLibrary(Path.Combine(AppDataPaths.DataDirectory, "desktop-updates", "unavailable", UpdateStager.SdkLibraryName));
            return false;
        }
    }
    private static void ConfigureSdk(UpdateInstallation installed)
    {
        var healthy = UpdatePlanner.Healthy(installed.Sdk);
        Service.RecordSdkHealth(healthy);
        var path = installed.Sdk is { } sdk && healthy ? Path.Combine(sdk.Directory, UpdateStager.SdkLibraryName)
            : Path.Combine(AppDataPaths.DataDirectory, "desktop-updates", "unavailable", UpdateStager.SdkLibraryName);
        AgentSdk.ConfigureNativeLibrary(Path.GetFullPath(path));
        try
        {
            Service.RecordSdkRuntimeAvailability(healthy && installed.Sdk is { } current
                && AgentSdk.GetNativeAbiVersion() == current.Abi && current.Abi == AgentSdk.RequiredNativeAbiVersion
                && AgentSdk.GetVersionAsync().GetAwaiter().GetResult().Version == current.Version);
        }
        catch (Exception) { Service.RecordSdkRuntimeAvailability(false); }
    }
    internal static void Cleanup()
    {
        try { UpdateCleanup.Prune(new UpdateStateStore(Path.Combine(AppDataPaths.DataDirectory, "desktop-updates"))); }
        catch (Exception) { DiagnosticLog.Warn("updates.cleanup", "cleanup_incomplete=true"); }
    }
    private static async Task PreflightIdentityAsync(UpdateInstallation installed)
    {
        var version = typeof(AppInfo).Assembly.GetCustomAttribute<AssemblyInformationalVersionAttribute>()?.InformationalVersion?.Split('+', 2)[0];
        if (version != installed.Desktop.Version || AgentSdk.RequiredNativeAbiVersion != installed.Desktop.Abi) throw new UpdateException("identity_invalid", "The staged Desktop identity does not match its release.");
        ConfigureSdk(installed);
        if (AgentSdk.GetNativeAbiVersion() != installed.Sdk?.Abi || (await AgentSdk.GetVersionAsync()).Version != installed.Sdk.Version)
            throw new UpdateException("identity_invalid", "The staged Agent SDK identity does not match its release.");
    }
}
