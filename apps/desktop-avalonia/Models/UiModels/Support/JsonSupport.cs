using System.Text.Encodings.Web;
using System.Text.Json;

namespace SunCode.Desktop.Models;

internal static class DisplayJson
{
    // This text is rendered in a read-only desktop code view, not emitted to HTML or a script.
    public static JsonSerializerOptions Options { get; } = new()
    {
        WriteIndented = true,
        Encoder = JavaScriptEncoder.UnsafeRelaxedJsonEscaping
    };
}
