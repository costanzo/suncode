using System.Text.Json;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

// Typed, fallback-aware reads over one GetSettings result, shared by the
// window view model and feature view models that own individual keys.
internal sealed class SettingsSnapshot(IReadOnlyList<SettingRecord> settings)
{
    public string String(string key, string fallback)
    {
        var setting = Find(key);
        return setting is not null && setting.Value.ValueKind == JsonValueKind.String
            ? setting.Value.GetString() ?? fallback
            : fallback;
    }

    public long Long(string key, long fallback)
    {
        var setting = Find(key);
        return setting is not null && setting.Value.TryGetInt64(out var parsed)
            ? parsed
            : fallback;
    }

    public bool Bool(string key, bool fallback)
    {
        var setting = Find(key);
        return setting is not null && setting.Value.ValueKind is JsonValueKind.True or JsonValueKind.False
            ? setting.Value.GetBoolean()
            : fallback;
    }

    public string[] StringArray(string key)
    {
        var setting = Find(key);
        return setting is not null && setting.Value.ValueKind == JsonValueKind.Array
            ? setting.Value.EnumerateArray()
                .Where(value => value.ValueKind == JsonValueKind.String)
                .Select(value => value.GetString() ?? string.Empty)
                .Where(value => value.Length > 0)
                .ToArray()
            : Array.Empty<string>();
    }

    private SettingRecord? Find(string key) => settings.FirstOrDefault(item => item.Key == key);
}
