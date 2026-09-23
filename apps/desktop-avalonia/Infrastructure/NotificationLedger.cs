using System.Text.Json;

namespace SunCode.Desktop.Infrastructure;

internal enum NotificationDisposition
{
    Delivered,
    SuppressedForeground
}

internal readonly record struct NotificationLedgerKey(string Kind, string CorrelationId)
{
    public override string ToString() => $"{Kind}:{CorrelationId}";
}

internal sealed record NotificationLedgerEntry(
    string Key,
    NotificationDisposition Disposition,
    DateTimeOffset HandledAt);

internal sealed class NotificationLedger
{
    private const int CurrentVersion = 1;
    private const int MaximumEntries = 2048;
    private static readonly TimeSpan Retention = TimeSpan.FromDays(8);
    private readonly string _path;
    private readonly Dictionary<string, NotificationLedgerEntry> _entries = new(StringComparer.Ordinal);
    private readonly object _gate = new();

    private sealed record FileModel(int Version, IReadOnlyList<NotificationLedgerEntry> Entries);

    public NotificationLedger(string dataDirectory)
    {
        _path = Path.Combine(dataDirectory, "desktop-notification-ledger.json");
        Load();
    }

    public bool Contains(NotificationLedgerKey key)
    {
        lock (_gate) return _entries.ContainsKey(key.ToString());
    }

    public void Record(NotificationLedgerKey key, NotificationDisposition disposition, DateTimeOffset? handledAt = null)
    {
        var value = key.ToString();
        if (string.IsNullOrWhiteSpace(key.Kind) || string.IsNullOrWhiteSpace(key.CorrelationId) || value.Length > 320) return;
        lock (_gate)
        {
            var now = handledAt ?? DateTimeOffset.UtcNow;
            _entries[value] = new NotificationLedgerEntry(value, disposition, now);
            Prune(now);
            Save();
        }
    }

    internal IReadOnlyList<NotificationLedgerEntry> Snapshot()
    {
        lock (_gate) return _entries.Values.OrderBy(entry => entry.HandledAt).ToArray();
    }

    private void Load()
    {
        try
        {
            if (!File.Exists(_path)) return;
            var file = JsonSerializer.Deserialize<FileModel>(File.ReadAllText(_path));
            if (file?.Version != CurrentVersion) return;
            foreach (var entry in file.Entries.TakeLast(MaximumEntries))
                if (!string.IsNullOrWhiteSpace(entry.Key) && entry.Key.Length <= 320)
                    _entries[entry.Key] = entry;
            Prune(DateTimeOffset.UtcNow);
        }
        catch (Exception exception) when (exception is IOException or JsonException or UnauthorizedAccessException)
        {
            DiagnosticLog.Warn("notification.ledger", $"load_failed=true error={exception.Message}");
        }
    }

    private void Prune(DateTimeOffset now)
    {
        foreach (var key in _entries.Values
                     .Where(entry => now - entry.HandledAt > Retention)
                     .Select(entry => entry.Key).ToArray())
            _entries.Remove(key);
        foreach (var key in _entries.Values.OrderByDescending(entry => entry.HandledAt)
                     .Skip(MaximumEntries).Select(entry => entry.Key).ToArray())
            _entries.Remove(key);
    }

    private void Save()
    {
        try
        {
            Directory.CreateDirectory(Path.GetDirectoryName(_path)!);
            var temporary = _path + ".tmp";
            File.WriteAllText(temporary, JsonSerializer.Serialize(new FileModel(
                CurrentVersion, _entries.Values.OrderBy(entry => entry.HandledAt).ToArray())));
            File.Move(temporary, _path, true);
        }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException)
        {
            DiagnosticLog.Warn("notification.ledger", $"save_failed=true error={exception.Message}");
        }
    }
}

internal static class NotificationForegroundEvaluator
{
    public static bool IsForeground(IEnumerable<(bool IsActive, bool IsVisible, bool IsMinimized)> windows) =>
        windows.Any(window => window.IsActive && window.IsVisible && !window.IsMinimized);
}
