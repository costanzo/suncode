using System.Reflection;
using SunCode.Desktop.Models;
using SunCode.Sdk;

namespace SunCode.Desktop.Infrastructure.Updates;

internal enum UpdatePhase { Unconfigured, Idle, Checking, Current, Available, Downloading, Verifying, Ready, Restarting, Failed }
internal enum UpdateOperation { Check, Download, Verification, Installation }
internal sealed record UpdateSnapshot(UpdatePhase Phase, UpdatePlan? Plan = null, long Downloaded = 0, long Total = 0, string? ErrorCode = null, UpdateOperation FailedOperation = UpdateOperation.Check);

internal sealed class UpdateCoordinator : IDisposable
{
    private readonly UpdateDistribution _distribution;
    private readonly UpdateStateStore _store;
    private readonly UpdateEnvironment _environment;
    private readonly UpdateTransport _transport;
    private readonly UpdateStager _stager;
    private readonly UpdateActivation _activation;
    private readonly SemaphoreSlim _gate = new(1, 1);
    private readonly CancellationTokenSource _lifetime = new();
    private CancellationTokenSource? _operation;
    private Task? _automaticTask;
    private bool _automaticChecks = true;
    private bool _disposed;
    public event Action? Changed;
    public UpdateSnapshot Snapshot { get; private set; }
    public bool AutomaticChecks => _automaticChecks;
    public UpdateInstallation? Installation { get; private set; }
    public bool? SdkHealthy { get; private set; }
    public bool? SdkRuntimeAvailable { get; private set; }
    internal void RecordSdkHealth(bool healthy) { SdkHealthy = healthy && SdkRuntimeAvailable != false; Changed?.Invoke(); }
    internal void RecordSdkRuntimeAvailability(bool available) { SdkRuntimeAvailable = available; if (!available) SdkHealthy = false; Changed?.Invoke(); }
    public string? StartupError { get; set; }
    internal Func<Task<bool>> IsBusyAsync { get; set; } = () => Task.FromResult(false);
    internal Func<Task<RunningUpdateVersions>> ReadRunningVersionsAsync { get; set; } = ReadActualVersionsAsync;
    internal Func<Task> ShutdownAsync { get; set; } = () => throw new UpdateException("restart_unavailable", "Restart is unavailable in this host.");

    public UpdateCoordinator(UpdateDistribution distribution, UpdateStateStore store, UpdateEnvironment environment,
        IUpdatePlatformVerifier? platform = null, HttpMessageHandler? handler = null, TimeProvider? time = null)
    {
        _distribution = distribution; _store = store; _environment = environment;
        _transport = new(distribution, store, environment, handler, time);
        var verifier = platform ?? new PlatformUpdateVerifier(distribution);
        _stager = new(_transport, store, verifier); _activation = new(distribution, store, environment, verifier, time);
        Snapshot = new(distribution.Enabled ? UpdatePhase.Idle : UpdatePhase.Unconfigured);
        try
        {
            var state = store.Read();
            _automaticChecks = state.AutomaticChecks; Installation = state.Installation;
            if (state.Staged is not null && distribution.Enabled) Snapshot = new(UpdatePhase.Idle);
            if (state.LastResult == "install_failed") Snapshot = new(UpdatePhase.Failed, ErrorCode: "install_failed");
        }
        catch (UpdateException e) { Snapshot = new(UpdatePhase.Failed, ErrorCode: e.Code); }
    }

    private void Set(UpdateSnapshot snapshot) { Snapshot = snapshot; Changed?.Invoke(); }
    public void SetAutomaticChecks(bool value)
    {
        try { _store.Change(state => state.AutomaticChecks = value); _automaticChecks = value; Changed?.Invoke(); }
        catch (Exception e) { Fail(e); }
    }
    public void StartAutomaticChecks()
    {
        if (_automaticTask is not null || !_distribution.Enabled) return;
        _automaticTask = AutomaticLoopAsync();
    }
    private async Task AutomaticLoopAsync()
    {
        try
        {
            await Task.Delay(TimeSpan.FromSeconds(5), _lifetime.Token);
            while (!_lifetime.IsCancellationRequested)
            {
                if (_automaticChecks && Snapshot.Phase is not (UpdatePhase.Ready or UpdatePhase.Downloading or UpdatePhase.Verifying or UpdatePhase.Restarting))
                    await CheckAsync();
                await Task.Delay(TimeSpan.FromHours(4), _lifetime.Token);
            }
        }
        catch (OperationCanceledException) { }
        catch (ObjectDisposedException) when (_disposed) { }
    }
    public async Task CheckAsync()
    {
        if (!await _gate.WaitAsync(0)) return;
        try
        {
            _operation = CancellationTokenSource.CreateLinkedTokenSource(_lifetime.Token);
            Set(new(UpdatePhase.Checking));
            if (StartupError is { } startup) throw new UpdateException(startup, "The installation requires repair.");
            var verified = await Task.Run(() => _transport.CheckAsync(_operation.Token), _operation.Token);
            var installed = _store.Read().Installation;
            Installation = installed;
            if (installed is null)
            {
                var running = await ReadRunningVersionsAsync();
                _operation.Token.ThrowIfCancellationRequested();
                var sdkAvailable = running.SdkAbi == running.DesktopAbi && !string.IsNullOrWhiteSpace(running.SdkVersion);
                RecordSdkRuntimeAvailability(sdkAvailable);
                // Informational version checks do not enroll this build for
                // installation or infer archive identities from local files.
                var catalog = verified.Catalog;
                if (sdkAvailable && running.DesktopVersion == catalog.Components.Desktop.Version
                    && running.SdkVersion == catalog.Components.Sdk.Version
                    && running.DesktopAbi == catalog.Components.Desktop.RequiredSdkAbi
                    && running.SdkAbi == catalog.Components.Sdk.Abi)
                {
                    Set(new(UpdatePhase.Current));
                    return;
                }
                throw new UpdateException("installation_unmanaged", "Automatic installation requires a managed distribution.");
            }
            if (!await Task.Run(() => UpdatePlanner.Healthy(installed.Desktop), _operation.Token)) throw new UpdateException("desktop_damaged", "Reinstall SunCode to repair the Desktop installation.");
            var healthy = await Task.Run(() => UpdatePlanner.Healthy(installed.Sdk), _operation.Token);
            RecordSdkHealth(healthy);
            var plan = UpdatePlanner.Select(verified, installed, _environment, healthy && SdkRuntimeAvailable != false);
            var staged = _store.Read().Staged;
            if (plan is not null && staged is not null && SamePlan(plan, staged)) Set(new(UpdatePhase.Ready, plan));
            else Set(new(plan is null ? UpdatePhase.Current : UpdatePhase.Available, plan));
        }
        catch (OperationCanceledException) when (_operation?.IsCancellationRequested == true) { Set(new(UpdatePhase.Idle)); }
        catch (Exception e) { Fail(e); }
        finally { _operation?.Dispose(); _operation = null; _gate.Release(); }
    }
    public async Task DownloadAsync()
    {
        if (!await _gate.WaitAsync(0)) return;
        var previous = Snapshot.Plan;
        try
        {
            if (previous is null) return;
            _operation = CancellationTokenSource.CreateLinkedTokenSource(_lifetime.Token);
            var token = _operation.Token;
            var verified = await Task.Run(() => _transport.Revalidate(previous.Verified.Envelope), token);
            var installed = _store.Read().Installation ?? throw new UpdateException("installation_unmanaged", "Reinstall SunCode before updating.");
            if (!UpdateActivation.SameInstallation(installed, previous.Installed)) throw new UpdateException("installation_changed", "Check for updates again.");
            var healthy = await Task.Run(() => UpdatePlanner.Healthy(installed.Sdk), token);
            RecordSdkHealth(healthy);
            var plan = UpdatePlanner.Select(verified, installed, _environment, healthy && SdkRuntimeAvailable != false);
            if (plan is null) { Set(new(UpdatePhase.Current)); return; }
            Set(new(UpdatePhase.Downloading, plan, 0, plan.SizeBytes));
            var lastProgress = System.Diagnostics.Stopwatch.GetTimestamp();
            await Task.Run(() => _stager.StageAsync(plan,
                (count, total) =>
                {
                    if (count != total && System.Diagnostics.Stopwatch.GetElapsedTime(lastProgress) < TimeSpan.FromMilliseconds(100)) return;
                    lastProgress = System.Diagnostics.Stopwatch.GetTimestamp();
                    Set(new(UpdatePhase.Downloading, plan, count, total));
                },
                () => Set(new(UpdatePhase.Verifying, plan, plan.SizeBytes, plan.SizeBytes)), token), token);
            Set(new(UpdatePhase.Ready, plan, plan.SizeBytes, plan.SizeBytes));
        }
        catch (OperationCanceledException) when (_operation?.IsCancellationRequested == true) { Set(new(previous is null ? UpdatePhase.Idle : UpdatePhase.Available, previous)); }
        catch (Exception e) { Fail(e, previous, Snapshot.Phase == UpdatePhase.Verifying ? UpdateOperation.Verification : UpdateOperation.Download); }
        finally { _operation?.Dispose(); _operation = null; _gate.Release(); }
    }
    public void CancelDownload() => _operation?.Cancel();
    public async Task RestartAsync()
    {
        if (!await _gate.WaitAsync(0)) return;
        var plan = Snapshot.Plan;
        ActivationJournal? journal = null;
        try
        {
            if (Snapshot.Phase != UpdatePhase.Ready || plan is null) return;
            if (await IsBusyAsync()) throw new UpdateException("active_work", "Finish active turns and close running terminals before restarting.");
            Set(new(UpdatePhase.Restarting, plan));
            journal = await Task.Run(() => _activation.PrepareAsync(_lifetime.Token), _lifetime.Token);
            await _activation.LaunchHelperAsync(journal, _lifetime.Token);
            if (await IsBusyAsync()) throw new UpdateException("active_work", "Active work started before restart. Try again after it finishes.");
            await ShutdownAsync();
        }
        catch (Exception e)
        {
            if (journal is not null) _activation.CancelRestart(journal.Id);
            if (e is UpdateException { Code: "active_work" or "restart_cancelled" }) Set(new(UpdatePhase.Ready, plan, ErrorCode: "active_work"));
            else Fail(e, plan, UpdateOperation.Installation);
        }
        finally { _gate.Release(); }
    }
    internal UpdateActivation Activation => _activation;
    private static Task<RunningUpdateVersions> ReadActualVersionsAsync() => Task.Run(async () =>
    {
        var desktopVersion = typeof(AppInfo).Assembly.GetCustomAttribute<AssemblyInformationalVersionAttribute>()?.InformationalVersion?.Split('+', 2)[0] ?? "";
        var desktopAbi = AgentSdk.RequiredNativeAbiVersion;
        try
        {
            var sdkAbi = AgentSdk.GetNativeAbiVersion();
            if (sdkAbi != desktopAbi) return new RunningUpdateVersions(desktopVersion, desktopAbi, SdkAbi: sdkAbi);
            var sdkVersion = (await AgentSdk.GetVersionAsync()).Version;
            return new RunningUpdateVersions(desktopVersion, desktopAbi, sdkVersion, sdkAbi);
        }
        catch (Exception) { return new RunningUpdateVersions(desktopVersion, desktopAbi); }
    });
    internal void ReloadInstallation() => Installation = _store.Read().Installation;
    private static bool SamePlan(UpdatePlan plan, StagedUpdate staged)
    {
        if (plan.Desktop != staged.Desktop || plan.Sdk != staged.Sdk || !UpdateActivation.SameInstallation(plan.Installed, staged.Baseline)) return false;
        try
        {
            using var old = System.Text.Json.JsonDocument.Parse(staged.Envelope);
            var catalog = UpdateJson.Read<UpdateCatalog>(System.Text.Encoding.UTF8.GetBytes(old.RootElement.GetProperty("signed").GetRawText()));
            return (!plan.Desktop || plan.Verified.Catalog.Components.Desktop == catalog.Components.Desktop
                    || plan.Verified.Catalog.Components.Desktop.Package.Sha256 == catalog.Components.Desktop.Package.Sha256 && plan.Verified.Catalog.Components.Desktop.Version == catalog.Components.Desktop.Version)
                && (!plan.Sdk || plan.Verified.Catalog.Components.Sdk.Package.Sha256 == catalog.Components.Sdk.Package.Sha256 && plan.Verified.Catalog.Components.Sdk.Version == catalog.Components.Sdk.Version);
        }
        catch (Exception e) when (e is System.Text.Json.JsonException or KeyNotFoundException) { return false; }
    }
    private void Fail(Exception e, UpdatePlan? plan = null, UpdateOperation operation = UpdateOperation.Check)
    {
        var code = e is UpdateException update ? update.Code : e is HttpRequestException or IOException or OperationCanceledException ? "network_failed" : "update_failed";
        if (code.StartsWith("catalog_", StringComparison.Ordinal) || code is "signature_invalid" or "invalid_catalog" or "release_conflict" or "configuration_invalid" or "schema_unsupported") plan = null;
        Set(new(UpdatePhase.Failed, plan, ErrorCode: code, FailedOperation: operation));
        DiagnosticLog.Warn("updates", $"operation_failed code={code}");
    }
    public void Dispose()
    {
        if (_disposed) return; _disposed = true;
        _lifetime.Cancel(); _operation?.Cancel(); _transport.Dispose();
        _lifetime.Dispose();
    }
}
