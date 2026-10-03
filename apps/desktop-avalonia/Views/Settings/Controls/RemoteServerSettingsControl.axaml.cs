using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Media.Imaging;
using QRCoder;
using System.IO;

namespace SunCode.Desktop.Views.Settings.Controls;

public sealed partial class RemoteServerSettingsControl : UserControl
{
    public event EventHandler<RoutedEventArgs>? SaveRequested;
    public event EventHandler<RoutedEventArgs>? DisconnectRequested;
    public event EventHandler<RoutedEventArgs>? ClearRequested;
    public TextBox ServerUrlInputControl => ServerUrlInput;
    public TextBox PairingCodeInputControl => PairingCodeInput;
    public ToggleSwitch E2eToggleControl => E2eToggle;
    public TextBlock StatusTextControl => StatusText;
    public Button DisconnectButtonControl => DisconnectButton;
    public StackPanel PairingSectionControl => PairingSection;
    public TextBlock PairingPayloadTextControl => PairingPayloadText;
    public Image PairingQrImageControl => PairingQrImage;
    public void SetPairingMetadata(string? hostId, string? accessTokenExpiresAt)
    {
        var parts = new List<string>();
        if (!string.IsNullOrWhiteSpace(hostId)) parts.Add($"Host ID: {hostId}");
        if (!string.IsNullOrWhiteSpace(accessTokenExpiresAt)) parts.Add($"Access token expires: {accessTokenExpiresAt}");
        PairingMetadataText.Text = string.Join(Environment.NewLine, parts);
    }
    public void SetPairingPayload(string? payload)
    {
        // The Rust remote controller supplies the complete application pairing URL.
        // Keep the control transport-agnostic: both http:// and https:// QR links
        // are valid according to the remote-control contract.
        PairingPayloadText.Text = payload ?? string.Empty;
        if (string.IsNullOrWhiteSpace(payload))
        {
            PairingQrImage.Source = null;
            return;
        }
        if (!Uri.TryCreate(payload, UriKind.Absolute, out var uri)
            || uri.Scheme is not ("http" or "https")
            || string.IsNullOrWhiteSpace(uri.Query))
        {
            PairingQrImage.Source = null;
            return;
        }
        using var generator = new QRCodeGenerator();
        using var data = generator.CreateQrCode(payload, QRCodeGenerator.ECCLevel.Q);
        using var qr = new PngByteQRCode(data);
        using var stream = new MemoryStream(qr.GetGraphic(8));
        PairingQrImage.Source = new Bitmap(stream);
    }
    public RemoteServerSettingsControl() => InitializeComponent();
    private void OnSave(object? sender, RoutedEventArgs e) => SaveRequested?.Invoke(this, e);
    private void OnDisconnect(object? sender, RoutedEventArgs e) => DisconnectRequested?.Invoke(this, e);
    private void OnClear(object? sender, RoutedEventArgs e) => ClearRequested?.Invoke(this, e);
}
