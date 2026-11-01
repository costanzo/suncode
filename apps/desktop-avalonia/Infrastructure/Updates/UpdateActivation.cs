using System.Diagnostics;

namespace SunCode.Desktop.Infrastructure.Updates;

internal sealed record ActivationJournal(string Id, string Phase, StagedUpdate Staged, UpdateInstallation Candidate,
    int ParentPid, long ParentStarted, int HelperPid = 0, long HelperStarted = 0);

// Every payload directory is immutable. Only the installation pointer is replaced;
// the original app remains a launcher, so interrupted updates never remove its entry point.
internal sealed class UpdateActivation(UpdateDistribution distribution, UpdateStateStore store, UpdateEnvironment environment, IUpdatePlatformVerifier platform, TimeProvider? time = null)
{
    public async Task<ActivationJournal> PrepareAsync(CancellationToken token)
    {
        var state = store.Read();
        if (state.Activation is { Phase: not "committed" }) throw new UpdateException("update_locked", "Another update installation is running.");
        var staged = state.Staged ?? throw new UpdateException("not_staged", "Download an update before restarting.");
        using var transport = new UpdateTransport(distribution, store, environment, time: time);
        var plan = await ValidateStagedAsync(staged, transport, token);
        var id = Guid.NewGuid().ToString("N");
        var root = Path.Combine(store.DirectoryPath, "versions", id);
        Directory.CreateDirectory(root);
        var candidate = staged.Baseline;
        if (staged.Desktop)
        {
            var destination = Path.Combine(root, "desktop");
            await UpdateArchive.ExtractAsync(Path.Combine(staged.Directory, "desktop.archive"), plan.Verified.Catalog.Components.Desktop.Package.Format, destination, token);
            UpdateStager.ValidateLayout(destination, true);
            await platform.VerifyAsync(destination, true, token);
            var payload = UpdateStager.DesktopPayload(destination);
            var release = plan.Verified.Catalog.Components.Desktop;
            candidate = candidate with { Desktop = new(release.Version, release.RequiredSdkAbi, release.Package.Sha256, payload, UpdateArchive.Manifest(payload)) };
        }
        if (staged.Sdk)
        {
            var destination = Path.Combine(root, "sdk");
            await UpdateArchive.ExtractAsync(Path.Combine(staged.Directory, "sdk.archive"), plan.Verified.Catalog.Components.Sdk.Package.Format, destination, token);
            UpdateStager.ValidateLayout(destination, false);
            await platform.VerifyAsync(destination, false, token);
            var release = plan.Verified.Catalog.Components.Sdk;
            candidate = candidate with { Sdk = new(release.Version, release.Abi, release.Package.Sha256, destination, UpdateArchive.Manifest(destination)) };
        }
        token.ThrowIfCancellationRequested();
        using var process = Process.GetCurrentProcess();
        var journal = new ActivationJournal(id, "prepared", staged, candidate, process.Id, process.StartTime.ToUniversalTime().Ticks);
        store.Change(s => s.Activation = journal);
        return journal;
    }

    public async Task LaunchHelperAsync(ActivationJournal journal, CancellationToken token)
    {
        var executable = Executable(journal.Staged.Baseline.Desktop.Directory);
        using var helper = Start(executable, "--suncode-update-apply", journal.Id);
        var ready = Signal(journal.Id, "helper-ready");
        var deadline = DateTime.UtcNow.AddSeconds(40);
        while (!File.Exists(ready))
        {
            token.ThrowIfCancellationRequested();
            if (helper.HasExited || DateTime.UtcNow >= deadline) throw new UpdateException("helper_failed", "The update installer could not start.");
            await Task.Delay(100, token);
        }
    }

    public async Task RunHelperAsync(string id, CancellationToken token)
    {
        ValidateId(id);
        Directory.CreateDirectory(store.DirectoryPath);
        await using var operation = new FileStream(Path.Combine(store.DirectoryPath, "activation.lock"), FileMode.OpenOrCreate, FileAccess.ReadWrite, FileShare.None);
        var journal = Require(id);
        if (journal.Phase != "prepared") throw new UpdateException("activation_invalid", "The pending update is invalid.");
        using var self = Process.GetCurrentProcess();
        journal = journal with { Phase = "waiting", HelperPid = self.Id, HelperStarted = self.StartTime.ToUniversalTime().Ticks };
        store.Change(s => s.Activation = journal);
        using var transport = new UpdateTransport(distribution, store, environment, time: time);
        await ValidateStagedAsync(journal.Staged, transport, token);
        if (!UpdatePlanner.Healthy(journal.Candidate.Desktop) || !UpdatePlanner.Healthy(journal.Candidate.Sdk)) throw Invalid();
        WriteSignal(id, "helper-ready");
        var deadline = DateTime.UtcNow.AddSeconds(60);
        while (ProcessAlive(journal.ParentPid, journal.ParentStarted))
        {
            if (Require(id).Phase == "cancelled" || DateTime.UtcNow >= deadline) throw new UpdateException("restart_cancelled", "The update restart was cancelled.");
            await Task.Delay(100, token);
        }
        journal = journal with { Phase = "preflight" };
        store.Change(s => s.Activation = journal);
        Process? child = null;
        try
        {
            child = Start(Executable(journal.Candidate.Desktop.Directory), "--suncode-update-preflight", id);
            deadline = DateTime.UtcNow.AddSeconds(30);
            while (!File.Exists(Signal(id, "preflight-ready")))
            {
                if (child.HasExited || File.Exists(Signal(id, "preflight-failed")) || DateTime.UtcNow >= deadline) throw Invalid();
                await Task.Delay(100, token);
            }
            // The child is still gated before Agent SDK open/SQLite startup.
            // This is the only commit point for the two-component combination.
            store.Change(s =>
            {
                if (s.Activation?.Id != id || s.Activation.Phase != "preflight") throw Invalid();
                s.Installation = journal.Candidate;
                s.PreviousInstallation = journal.Staged.Baseline;
                s.Activation = journal with { Phase = "committed" };
                s.Staged = null;
                s.LastResult = "installed";
            });
            WriteSignal(id, "commit-ready");
        }
        catch
        {
            // No commit means the child cannot have opened Agent state. Old
            // binaries and their active pointer were never overwritten.
            var active = store.Read().Installation;
            if (active is null || !SameInstallation(active, journal.Candidate))
            {
                if (child is { HasExited: false }) { child.Kill(entireProcessTree: true); await child.WaitForExitAsync(token); }
                store.Change(s => { s.Activation = null; s.LastResult = "install_failed"; });
                using var restored = Start(Executable(journal.Staged.Baseline.Desktop.Directory));
            }
            throw;
        }
        finally { child?.Dispose(); }
    }

    public async Task<UpdateInstallation> PreflightAsync(string id, Func<UpdateInstallation, Task> checkIdentity, CancellationToken token)
    {
        ValidateId(id);
        try
        {
            var journal = Require(id);
            if (journal.Phase != "preflight" || !ProcessAlive(journal.HelperPid, journal.HelperStarted)) throw Invalid();
            if (!UpdatePlanner.Healthy(journal.Candidate.Desktop) || !UpdatePlanner.Healthy(journal.Candidate.Sdk)) throw Invalid();
            await checkIdentity(journal.Candidate);
            WriteSignal(id, "preflight-ready");
            var deadline = DateTime.UtcNow.AddSeconds(35);
            while (true)
            {
                var state = store.Read();
                if (state.Installation is { } active && SameInstallation(active, journal.Candidate) && File.Exists(Signal(id, "commit-ready")))
                    return active;
                if (DateTime.UtcNow >= deadline || !ProcessAlive(journal.HelperPid, journal.HelperStarted)) throw Invalid();
                await Task.Delay(50, token);
            }
        }
        catch { WriteSignal(id, "preflight-failed"); throw; }
    }

    public void RecoverInterrupted()
    {
        var state = store.Read();
        if (state.Activation is not { } journal) return;
        if (journal.Phase == "committed") { store.Change(s => s.Activation = null); return; }
        if (ProcessAlive(journal.HelperPid, journal.HelperStarted))
            throw new UpdateException("update_locked", "SunCode is finishing an update. Try again shortly.");
        // Before commit the previous pointer remains authoritative. Never roll
        // back a committed pointer: the new SDK may already have migrated data.
        store.Change(s => { s.Activation = null; s.LastResult = "install_failed"; });
    }

    public void CancelRestart(string id) => store.Change(s =>
    {
        if (s.Activation?.Id == id && s.Activation.Phase != "committed") s.Activation = s.Activation with { Phase = "cancelled" };
    });

    private async Task<UpdatePlan> ValidateStagedAsync(StagedUpdate staged, UpdateTransport transport, CancellationToken token)
    {
        _ = UpdateArchive.Inside(Path.Combine(store.DirectoryPath, "staging"), Path.GetRelativePath(Path.Combine(store.DirectoryPath, "staging"), staged.Directory).Replace(Path.DirectorySeparatorChar, '/'));
        VerifiedCatalog verified;
        try { verified = transport.Revalidate(staged.Envelope); }
        catch (UpdateException e) when (e.Code is "catalog_expired" or "catalog_rollback") { verified = await transport.CheckAsync(token); }
        var state = store.Read();
        if (state.Installation is not { } installed || !SameInstallation(installed, staged.Baseline)
            || !UpdatePlanner.Healthy(installed.Desktop)) throw new UpdateException("installation_changed", "The installation changed. Check for updates again.");
        var plan = UpdatePlanner.Select(verified, installed, environment, !staged.Repair && UpdatePlanner.Healthy(installed.Sdk));
        if (plan is null || plan.Desktop != staged.Desktop || plan.Sdk != staged.Sdk) throw Invalid();
        using var oldDocument = System.Text.Json.JsonDocument.Parse(staged.Envelope);
        var original = UpdateJson.Read<UpdateCatalog>(System.Text.Encoding.UTF8.GetBytes(oldDocument.RootElement.GetProperty("signed").GetRawText()));
        if (plan.Desktop && (verified.Catalog.Components.Desktop.Package.Sha256 != original.Components.Desktop.Package.Sha256 || verified.Catalog.Components.Desktop.Version != original.Components.Desktop.Version)
            || plan.Sdk && (verified.Catalog.Components.Sdk.Package.Sha256 != original.Components.Sdk.Package.Sha256 || verified.Catalog.Components.Sdk.Version != original.Components.Sdk.Version)) throw Invalid();
        foreach (var (id, release) in new (string, ComponentRelease)[] { ("desktop", verified.Catalog.Components.Desktop), ("sdk", verified.Catalog.Components.Sdk) })
        {
            if (id == "desktop" && !plan.Desktop || id == "sdk" && !plan.Sdk) continue;
            var file = Path.Combine(staged.Directory, id + ".archive");
            if (!File.Exists(file) || new FileInfo(file).Length != release.Package.SizeBytes || UpdateStateStore.HashFile(file) != release.Package.Sha256) throw Invalid();
        }
        return plan;
    }

    internal static bool SameInstallation(UpdateInstallation a, UpdateInstallation b)
        => a.Target == b.Target && SameComponent(a.Desktop, b.Desktop) && SameComponent(a.Sdk, b.Sdk);
    private static bool SameComponent(InstalledComponent? a, InstalledComponent? b)
        => a is null ? b is null : b is not null && a.Directory == b.Directory && a.Version == b.Version && a.Abi == b.Abi
            && a.ArchiveSha256 == b.ArchiveSha256 && a.Files.Count == b.Files.Count && a.Files.All(pair => b.Files.GetValueOrDefault(pair.Key) == pair.Value);

    private ActivationJournal Require(string id) => store.Read().Activation is { } journal && journal.Id == id ? journal : throw Invalid();
    private string Signal(string id, string name) => Path.Combine(store.DirectoryPath, "signals", id, name);
    private void WriteSignal(string id, string name) => UpdateStateStore.AtomicWrite(Signal(id, name), [1]);
    private static void ValidateId(string id) { if (!Guid.TryParseExact(id, "N", out _)) throw Invalid(); }
    internal static bool ProcessAlive(int id, long started)
    {
        if (id <= 0 || started <= 0) return false;
        try { using var process = Process.GetProcessById(id); return !process.HasExited && process.StartTime.ToUniversalTime().Ticks == started; }
        catch (Exception e) when (e is ArgumentException or InvalidOperationException or System.ComponentModel.Win32Exception) { return false; }
    }
    internal static string Executable(string root) => Path.Combine(UpdateStager.DesktopExecutableDirectory(root), OperatingSystem.IsWindows() ? "SunCode.exe" : "SunCode");
    internal static Process Start(string executable, params string[] arguments)
    {
        var start = new ProcessStartInfo(executable) { UseShellExecute = false, WorkingDirectory = Path.GetDirectoryName(executable)! };
        foreach (var argument in arguments) start.ArgumentList.Add(argument);
        return Process.Start(start) ?? throw Invalid();
    }
    private static UpdateException Invalid() => new("activation_invalid", "The staged update is no longer usable. Check for updates again.");
}
