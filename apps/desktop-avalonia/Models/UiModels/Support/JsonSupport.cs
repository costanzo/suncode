using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Text.Encodings.Web;
using System.Text.Json;
using System.Text.Json.Nodes;
using Avalonia.Media.Imaging;
using SunCode.Desktop.Infrastructure;

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

internal static class JsonExtensions
{
    public static string String(this JsonObject value, params string[] names)
    {
        foreach (var name in names)
        {
            if (value[name] is JsonValue item && item.TryGetValue<string>(out var result))
            {
                return result ?? string.Empty;
            }
        }
        return string.Empty;
    }

    public static int Int(this JsonObject value, string name) =>
        value[name]?.GetValue<int>() ?? 0;

    public static long Long(this JsonObject value, params string[] names)
    {
        foreach (var name in names)
        {
            if (value[name] is JsonValue item && item.TryGetValue<long>(out var result))
            {
                return result;
            }
        }
        return 0;
    }

    public static bool Bool(this JsonObject value, string name) =>
        value[name]?.GetValue<bool>() ?? false;

    public static JsonObject Object(this JsonObject value, string name) =>
        value[name] as JsonObject ?? [];

    public static JsonArray Array(this JsonObject value, params string[] names)
    {
        foreach (var name in names)
        {
            if (value[name] is JsonArray result) return result;
        }
        return [];
    }
}
