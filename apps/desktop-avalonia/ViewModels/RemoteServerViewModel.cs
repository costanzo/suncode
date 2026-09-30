using Avalonia.Threading;
using SunCode.Desktop.Infrastructure;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

public sealed class RemoteServerViewModel : ObservableObject, IDisposable
{
    private readonly IViewModelHost _host;
    private readonly DispatcherTimer _statusTimer = new() { Interval = TimeSpan.FromSeconds(2) };
    private bool _configured;
    private bool _connected;
    private bool _connecting;
    private bool _disposed;

    internal RemoteServerViewModel(IViewModelHost host) => _host = host;

    public sealed record LoadResult(RemoteServerConfiguration Configuration, RemoteServerStatus Status);

    public bool Configured { get => _configured; private set => SetProperty(ref _configured, value); }
    public bool Connected { get => _connected; private set => SetProperty(ref _connected, value); }
    public bool Connecting { get => _connecting; private set => SetProperty(ref _connecting, value); }
    public string StatusText => Connected
        ? LocalizationService.GetString("Loc_RemoteConnected", "Remote connected")
        : Connecting
            ? LocalizationService.GetString("Loc_RemoteConnecting", "Remote connecting")
            : LocalizationService.GetString("Loc_RemoteDisconnected", "Remote disconnected");

    public async Task<LoadResult> LoadAsync()
    {
        if (!await _host.EnsureSdkReadyAsync()) return new LoadResult(new RemoteServerConfiguration(string.Empty, string.Empty), new RemoteServerStatus(false, false, false, null, null, "Local agent unavailable"));
        var sdk = _host.Sdk!;
        var status = await sdk.GetRemoteServerStatusAsync();
        ApplyStatus(status);
        return new LoadResult(await sdk.GetRemoteServerConfigurationAsync(), status);
    }

    public async Task<RemoteServerStatus> SaveAsync(string? serverUrl, string? pairingCode)
    {
        if (!await _host.EnsureSdkReadyAsync()) return new RemoteServerStatus(false, false, false, null, null, "Local agent unavailable");
        var sdk = _host.Sdk!;
        try
        {
            await sdk.SaveRemoteServerConfigurationAsync(new RemoteServerConfiguration(serverUrl?.Trim() ?? string.Empty, pairingCode?.Trim() ?? string.Empty));
            var status = await sdk.ConnectRemoteServerAsync();
            ApplyStatus(status);
            return status;
        }
        catch (Exception exception)
        {
            _host.ReportPresentationError(exception.Message);
            var status = await sdk.GetRemoteServerStatusAsync();
            ApplyStatus(status);
            return status;
        }
    }

    public async Task<RemoteServerStatus> DisconnectAsync()
    {
        if (_host.Sdk is not { } sdk) return new RemoteServerStatus(false, false, false, null, null, null);
        var status = await sdk.DisconnectRemoteServerAsync();
        ApplyStatus(status);
        return status;
    }

    public async Task<RemoteServerStatus> ClearAsync()
    {
        if (_host.Sdk is not { } sdk) return new RemoteServerStatus(false, false, false, null, null, null);
        var status = await sdk.ClearRemoteServerConfigurationAsync();
        ApplyStatus(status);
        return status;
    }

    // Loads the current status and polls it while the owning window lives.
    internal async Task StartPollingAsync()
    {
        if (_disposed || _host.Sdk is not { } sdk) return;
        ApplyStatus(await sdk.GetRemoteServerStatusAsync());
        _statusTimer.Tick -= StatusTick;
        _statusTimer.Tick += StatusTick;
        _statusTimer.Start();
    }

    internal void RefreshLocalizedText() => OnPropertyChanged(nameof(StatusText));

    internal void ApplyStatus(RemoteServerStatus status)
    {
        Configured = status.Configured;
        Connected = status.Connected;
        Connecting = status.Connecting;
        OnPropertyChanged(nameof(StatusText));
    }

    private async void StatusTick(object? sender, EventArgs e)
    {
        if (_disposed || _host.Sdk is not { } sdk) return;
        try { ApplyStatus(await sdk.GetRemoteServerStatusAsync()); }
        catch (Exception exception) { DiagnosticLog.Error("remote.status", exception, "operation=poll"); }
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;
        _statusTimer.Stop();
        _statusTimer.Tick -= StatusTick;
    }
}
