using System.Text.Json;
using SunCode.Sdk;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Infrastructure;

// One Desktop service across project windows. Its own managed SDK reference keeps
// persistence available until the last window releases it and setup has stopped.
internal sealed class ShellSetupService
{
    private static readonly SemaphoreSlim Lifetime = new(1, 1);
    private static ShellSetupService? _instance;
    private static int _references;
    private AgentSdk? _ownedSdk;
    internal static ShellSetupService Current => Volatile.Read(ref _instance) ?? throw new InvalidOperationException("Desktop shell services are unavailable");
    internal static async Task AcquireAsync(string userId)
    {
        await Lifetime.WaitAsync().ConfigureAwait(false);
        try
        {
            if (_instance is null)
            {
                var sdk = await AgentSdk.OpenAsync(userId).ConfigureAwait(false);
                try
                {
                    var service = new ShellSetupService(() => ReadPreferencesAsync(sdk),
                        async request => { await sdk.SaveShellPreferencesAsync(request).ConfigureAwait(false); },
                        () => sdk.GetDownloadNetworkConfigurationAsync()) { _ownedSdk = sdk };
                    await service.InitializeAsync().ConfigureAwait(false);
                    _instance = service;
                }
                catch { sdk.Dispose(); throw; }
            }
            ++_references;
        }
        finally { Lifetime.Release(); }
    }
    internal static async Task ReleaseAsync()
    {
        await Lifetime.WaitAsync().ConfigureAwait(false);
        try
        {
            if (_references == 0 || --_references != 0) return;
            var service = _instance!;
            await service.ShutdownAsync().ConfigureAwait(false);
            service._ownedSdk?.Dispose();
            _instance = null;
        }
        finally { Lifetime.Release(); }
    }
    private readonly Func<Task<(ShellConfiguration Agent, ShellConfiguration Terminal)>> _read;
    private readonly Func<ShellPreferencesRequest, Task> _save;
    private readonly Func<Task<DownloadNetworkConfiguration>> _network;
    private readonly Func<ShellCheckRequest, CancellationToken, Task<ShellCheckResult>> _check;
    private readonly Func<DownloadNetworkConfiguration, Action<string, int>, CancellationToken, Task> _install;
    private readonly Action _ensureInstallSupported;
    internal ShellSetupService(Func<Task<(ShellConfiguration Agent, ShellConfiguration Terminal)>> read,
        Func<ShellPreferencesRequest, Task> save, Func<Task<DownloadNetworkConfiguration>> network,
        Func<ShellCheckRequest, CancellationToken, Task<ShellCheckResult>>? check = null,
        Func<DownloadNetworkConfiguration, Action<string, int>, CancellationToken, Task>? install = null,
        Action? ensureInstallSupported = null)
    {
        _read = read; _save = save; _network = network;
        _check = check ?? ShellResolver.CheckAsync;
        _install = install ?? PowerShellInstaller.InstallAsync;
        _ensureInstallSupported = ensureInstallSupported ?? (() => { _ = ShellResolver.ManagedDirectory; });
    }
    private static readonly JsonSerializerOptions Json = new() { PropertyNameCaseInsensitive = true };
    private readonly SemaphoreSlim _commit = new(1, 1);
    private readonly object _gate = new();
    private CancellationTokenSource? _cancellation;
    private Task? _installation;
    private bool _closed;
    private ShellInstallationStatus _status = new("idle", false, 0, "", null, false);
    internal ShellInstallationStatus Status { get { lock (_gate) return _status; } }

    private static async Task<(ShellConfiguration Agent, ShellConfiguration Terminal)> ReadPreferencesAsync(AgentSdk sdk)
    {
        var settings = (await sdk.GetSettingsAsync(new()).ConfigureAwait(false)).Settings;
        ShellConfiguration Read(string key) => settings.FirstOrDefault(value => value.Key == key)?.Value.Deserialize<ShellConfiguration>(Json) ?? new();
        return (Read("agent_shell"), Read("terminal_shell"));
    }
    internal async Task<ShellCheckResult> CheckAsync(ShellCheckRequest request)
    {
        if (request.Configuration is null)
        {
            var preferences = await _read().ConfigureAwait(false);
            request = request with { Configuration = request.Target == "terminal" ? preferences.Terminal : preferences.Agent };
        }
        return await _check(request, default).ConfigureAwait(false);
    }
    internal async Task<ShellSettingsResult> SettingsAsync()
    {
        var preferences = await _read().ConfigureAwait(false);
        var agent = _check(new("agent", preferences.Agent), default);
        var terminal = _check(new("terminal", preferences.Terminal), default);
        string? directory = null; ShellCheckResult? managed = null;
        if (OperatingSystem.IsWindows())
        {
            directory = ShellResolver.ManagedDirectory;
            if (File.Exists(Path.Combine(directory, "pwsh.exe"))) managed = await _check(new("terminal", new("managed")), default).ConfigureAwait(false);
        }
        return new(await agent.ConfigureAwait(false), await terminal.ConfigureAwait(false), managed, directory,
            OperatingSystem.IsWindows() ? "windows" : OperatingSystem.IsMacOS() ? "macos" : "linux",
            OperatingSystem.IsWindows() ? ShellResolver.Architecture : System.Runtime.InteropServices.RuntimeInformation.OSArchitecture.ToString().ToLowerInvariant());
    }
    internal async Task<ShellCheckResult> SaveAsync(ShellCheckRequest request)
    {
        await _commit.WaitAsync().ConfigureAwait(false);
        try
        {
            lock (_gate)
            {
                if (_closed) throw new ObjectDisposedException(nameof(ShellSetupService));
                if (_status.Running && _status.UseForBoth) throw new SdkException("shell_install_busy", "Shell settings are locked until installation completes");
            }
            return await SaveCheckedAsync(request).ConfigureAwait(false);
        }
        finally { _commit.Release(); }
    }
    private async Task<ShellCheckResult> SaveCheckedAsync(ShellCheckRequest request, CancellationToken token = default)
    {
        var checkedShell = await _check(request, token).ConfigureAwait(false);
        if (checkedShell.Shell is null) return checkedShell;
        var configuration = checkedShell.Configuration with { Resolved = checkedShell.Shell };
        token.ThrowIfCancellationRequested();
        await _save(request.Target switch
        {
            "agent" => new(Agent: configuration), "terminal" => new(Terminal: configuration), "both" => new(configuration, configuration),
            _ => throw new ArgumentException("Shell target must be agent, terminal or both")
        }).ConfigureAwait(false);
        return checkedShell;
    }
    // Refresh launch facts during Desktop initialization before local submissions.
    // Detection failure clears stale facts and leaves the preference available for repair in Settings.
    internal async Task InitializeAsync()
    {
        var preferences = await _read().ConfigureAwait(false);
        var agent = await _check(new("agent", preferences.Agent), default).ConfigureAwait(false);
        await _save(new(Agent: preferences.Agent with { Resolved = agent.Shell })).ConfigureAwait(false);
    }
    internal ShellInstallationStatus StartInstallation(bool useForBoth)
    {
        _ensureInstallSupported();
        lock (_gate)
        {
            if (_closed) throw new ObjectDisposedException(nameof(ShellSetupService));
            if (_status.Running) throw new SdkException("shell_install_busy", "PowerShell installation is already running");
            _cancellation?.Dispose(); _cancellation = new();
            _status = new("downloading", true, 0, "", null, useForBoth);
            _installation = Task.Run(() => InstallAsync(useForBoth, _cancellation.Token));
            return _status;
        }
    }
    internal async Task<ShellInstallationStatus> CancelInstallationAsync()
    {
        await _commit.WaitAsync().ConfigureAwait(false);
        try { lock (_gate) { if (_status.Running) _cancellation?.Cancel(); return _status; } }
        finally { _commit.Release(); }
    }
    private async Task InstallAsync(bool useForBoth, CancellationToken token)
    {
        try
        {
            var network = await _network().ConfigureAwait(false);
            await _install(network, (stage, progress) =>
            {
                lock (_gate) _status = _status with { Stage = stage, Progress = progress };
            }, token).ConfigureAwait(false);
            await _commit.WaitAsync(token).ConfigureAwait(false);
            try
            {
                token.ThrowIfCancellationRequested();
                if (useForBoth)
                {
                    var result = await SaveCheckedAsync(new("both", new("managed")), token).ConfigureAwait(false);
                    if (result.Shell is null) throw new SdkException(result.ErrorCode ?? "shell_probe_failed", result.Message);
                }
                lock (_gate) _status = new("installed", false, 100, useForBoth ? "PowerShell 7 selected for both shells" : "PowerShell 7 installed", null, useForBoth);
            }
            finally { _commit.Release(); }
        }
        catch (Exception error)
        {
            var cancelled = error is OperationCanceledException && token.IsCancellationRequested;
            lock (_gate) _status = _status with { Stage = cancelled ? "cancelled" : "failed", Running = false,
                Message = cancelled ? "Installation cancelled. Shell settings were kept" : error.Message,
                ErrorCode = cancelled ? "shell_install_cancelled" : (error as SdkException)?.Code ?? "shell_install_failed" };
        }
    }
    internal async Task ShutdownAsync()
    {
        Task? task;
        await _commit.WaitAsync().ConfigureAwait(false);
        try { lock (_gate) { _closed = true; _cancellation?.Cancel(); task = _installation; } }
        finally { _commit.Release(); }
        if (task is not null) await task.ConfigureAwait(false);
    }
}
