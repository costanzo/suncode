using System.Globalization;
using System.IO;
using Avalonia.Controls;
using Avalonia.Data.Converters;
using Avalonia.Media.Imaging;
using QRCoder;

namespace SunCode.Desktop.Views.Settings.Controls;

public sealed partial class RemoteServerSettingsControl : UserControl
{
    public RemoteServerSettingsControl() => InitializeComponent();
}

// Renders a pairing link as a QR bitmap. The Rust remote controller supplies
// the complete application URL; both http:// and https:// links are valid
// under the remote-control contract, but a link without a query is not.
public sealed class PairingQrConverter : IValueConverter
{
    public static readonly PairingQrConverter Instance = new();

    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        if (value is not string payload
            || string.IsNullOrWhiteSpace(payload)
            || !Uri.TryCreate(payload, UriKind.Absolute, out var uri)
            || uri.Scheme is not ("http" or "https")
            || string.IsNullOrWhiteSpace(uri.Query))
            return null;

        using var generator = new QRCodeGenerator();
        using var data = generator.CreateQrCode(payload, QRCodeGenerator.ECCLevel.Q);
        using var qr = new PngByteQRCode(data);
        using var stream = new MemoryStream(qr.GetGraphic(8));
        return new Bitmap(stream);
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture) =>
        throw new NotSupportedException();
}
