using System.Windows.Input;
using SunCode.Desktop.Infrastructure;
using SunCode.Sdk.Models;
using static SunCode.Desktop.ViewModels.SettingsFormatting;

namespace SunCode.Desktop.ViewModels;

// Edit state of the Remote server settings page: the connection form, the
// latest connection status line, and the phone pairing link.
public sealed class RemoteServerEditorViewModel : ObservableObject, IDisposable
{
    private readonly RemoteServerViewModel _remote;
    private readonly Action<Action> _post;
    private ISettingsDialogs? _dialogs;
    private string? _serverUrl;
    private string? _pairingCode;
    private bool _e2eEnabled = true;
    private string _statusText = string.Empty;
    private bool _isConnected;
    private bool _isConnectedOrConnecting;
    private string? _pairingUrl;
    private string _pairingMetadata = string.Empty;

    // post marshals SDK status callbacks onto the UI thread.
    internal RemoteServerEditorViewModel(RemoteServerViewModel remote, Action<Action> post)
    {
        _remote = remote;
        _post = post;
        _remote.StatusChanged += RemoteStatusChanged;
        SaveCommand = new AsyncRelayCommand(SaveAsync);
        DisconnectCommand = new AsyncRelayCommand(async () => Apply(await _remote.DisconnectAsync()));
        ClearCommand = new AsyncRelayCommand(ClearAsync);
        CopyPairingCommand = new AsyncRelayCommand(CopyPairingAsync);
    }

    public ICommand SaveCommand { get; }
    public ICommand DisconnectCommand { get; }
    public ICommand ClearCommand { get; }
    public ICommand CopyPairingCommand { get; }

    public string? ServerUrl { get => _serverUrl; set => SetProperty(ref _serverUrl, value); }
    public string? PairingCode { get => _pairingCode; set => SetProperty(ref _pairingCode, value); }
    public bool E2eEnabled { get => _e2eEnabled; set => SetProperty(ref _e2eEnabled, value); }
    public string StatusText { get => _statusText; private set => SetProperty(ref _statusText, value); }

    public bool IsSaveVisible => !_isConnected;
    public bool IsDisconnectVisible => _isConnectedOrConnecting;

    // Prefer the fully qualified QR URL from the SDK; the legacy payload keeps
    // older installations pairing until they migrate.
    public string? PairingUrl { get => _pairingUrl; private set => SetProperty(ref _pairingUrl, value); }
    public bool HasPairing => !string.IsNullOrWhiteSpace(PairingUrl);
    public string PairingMetadata { get => _pairingMetadata; private set => SetProperty(ref _pairingMetadata, value); }

    internal void AttachDialogs(ISettingsDialogs? dialogs) => _dialogs = dialogs;

    public async Task LoadAsync()
    {
        var loaded = await _remote.LoadAsync();
        ServerUrl = loaded.Configuration.ServerUrl;
        PairingCode = loaded.Configuration.PairingCode;
        E2eEnabled = loaded.Configuration.E2eEnabled;
        Apply(loaded.Status);
    }

    // Applies a status snapshot to the status line, buttons, and pairing section.
    internal void Apply(RemoteServerStatus status)
    {
        _isConnected = status.Connected;
        _isConnectedOrConnecting = status.Connected || status.Connecting;
        OnPropertyChanged(nameof(IsSaveVisible));
        OnPropertyChanged(nameof(IsDisconnectVisible));
        PairingUrl = status.MobilePairingUrl ?? status.MobilePairingPayload;
        OnPropertyChanged(nameof(HasPairing));
        PairingMetadata = FormatPairingMetadata(status.HostId, status.AccessTokenExpiresAt);
        StatusText = status.Error ?? (status.Connected
            ? L("LocConnected", "Connected")
            : status.Configured ? L("LocDisconnected", "Disconnected") : string.Empty);
    }

    internal static string FormatPairingMetadata(string? hostId, string? accessTokenExpiresAt)
    {
        var parts = new List<string>();
        if (!string.IsNullOrWhiteSpace(hostId)) parts.Add($"Host ID: {hostId}");
        if (!string.IsNullOrWhiteSpace(accessTokenExpiresAt)) parts.Add($"Access token expires: {accessTokenExpiresAt}");
        return string.Join(Environment.NewLine, parts);
    }

    private async Task SaveAsync()
    {
        var status = await _remote.SaveAsync(ServerUrl, PairingCode, E2eEnabled);
        Apply(status);
        StatusText = status.Error ?? (status.Connected
            ? L("LocConnected", "Connected")
            : L("LocSavedConnectionUnavailable", "Saved; connection unavailable"));
    }

    private async Task ClearAsync()
    {
        var status = await _remote.ClearAsync();
        ServerUrl = string.Empty;
        PairingCode = string.Empty;
        E2eEnabled = true;
        Apply(status);
    }

    private async Task CopyPairingAsync()
    {
        if (string.IsNullOrWhiteSpace(PairingUrl) || _dialogs is null) return;
        if (await _dialogs.CopyTextAsync(PairingUrl)) StatusText = L("LocCopiedToClipboard", "Copied to clipboard.");
    }

    private void RemoteStatusChanged(object? sender, RemoteServerStatus status) => _post(() => Apply(status));

    public void Dispose()
    {
        _remote.StatusChanged -= RemoteStatusChanged;
        _dialogs = null;
    }
}
