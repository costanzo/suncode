using System.Text.Json;

namespace SunCode.Desktop.Infrastructure;

internal sealed class UiStateDocument
{
    public int SchemaVersion { get; set; } = UiStateStore.CurrentSchemaVersion;
    public UiSettingsState Settings { get; set; } = new();
    public Dictionary<string, UiProjectState> Projects { get; set; } = new(StringComparer.Ordinal);
}

internal sealed class UiSettingsState
{
    public string Page { get; set; } = "defaults";
    public string? ProviderId { get; set; }
    public bool ProvidersExpanded { get; set; } = true;
}

internal sealed class UiProjectState
{
    public string LeftRegion { get; set; } = "sessions";
    public string RightRegion { get; set; } = "review";
    public string BottomDrawer { get; set; } = "closed";
    public double? NavigationWidth { get; set; }
    public double? ReviewWidth { get; set; }
    public double? BottomDrawerHeight { get; set; }
    public double? WindowX { get; set; }
    public double? WindowY { get; set; }
    public double? WindowWidth { get; set; }
    public double? WindowHeight { get; set; }
    public string WindowState { get; set; } = "normal";
    public string? CurrentContentKind { get; set; }
    public string? CurrentSessionId { get; set; }
    public string? CurrentDependencyId { get; set; }
    public string? CurrentFilePath { get; set; }
    public string? LastSessionId { get; set; }
    public List<UiRecentContentState> RecentContent { get; set; } = [];
}

internal sealed class UiRecentContentState
{
    public string Kind { get; set; } = string.Empty;
    public string? SessionId { get; set; }
    public string? DependencyId { get; set; }
    public string? Path { get; set; }
    public DateTimeOffset LastViewedAt { get; set; }
}

internal sealed class UiStateStore : IDisposable
{
    internal const int CurrentSchemaVersion = 1;
    private static readonly JsonSerializerOptions JsonOptions = new(JsonSerializerDefaults.General)
    {
        WriteIndented = true,
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase
    };
    private readonly object _gate = new();
    private readonly string _path;
    private UiStateDocument _document;
    private Timer? _flushTimer;
    private bool _dirty;
    private bool _futureVersion;
    private bool _disposed;

    internal UiStateStore(string? path = null)
    {
        _path = path ?? Path.Combine(AppDataPaths.DataDirectory, "ui-state.json");
        _document = Load();
    }

    internal UiSettingsState Settings
    {
        get { lock (_gate) return Clone(_document.Settings); }
    }

    internal UiProjectState Project(string projectId)
    {
        lock (_gate)
        {
            if (!_document.Projects.TryGetValue(projectId, out var value))
            {
                value = new UiProjectState();
                _document.Projects[projectId] = value;
            }
            return Clone(value);
        }
    }

    internal void UpdateSettings(Action<UiSettingsState> update)
    {
        lock (_gate)
        {
            update(_document.Settings);
            MarkDirtyLocked();
        }
    }

    internal void UpdateProject(string projectId, Action<UiProjectState> update)
    {
        if (string.IsNullOrWhiteSpace(projectId)) return;
        lock (_gate)
        {
            if (!_document.Projects.TryGetValue(projectId, out var value))
            {
                value = new UiProjectState();
                _document.Projects[projectId] = value;
            }
            update(value);
            value.RecentContent = value.RecentContent.Take(20).ToList();
            MarkDirtyLocked();
        }
    }

    internal void Flush()
    {
        UiStateDocument snapshot;
        lock (_gate)
        {
            if (!_dirty || _futureVersion || _disposed) return;
            snapshot = Clone(_document);
            _dirty = false;
            _flushTimer?.Dispose();
            _flushTimer = null;
        }
        try
        {
            var directory = Path.GetDirectoryName(_path);
            if (!string.IsNullOrWhiteSpace(directory)) Directory.CreateDirectory(directory);
            var temp = _path + ".tmp";
            File.WriteAllText(temp, JsonSerializer.Serialize(snapshot, JsonOptions));
            File.Move(temp, _path, true);
        }
        catch (Exception exception)
        {
            DiagnosticLog.Error("ui-state.write", exception, $"path={_path}");
        }
    }

    public void Dispose()
    {
        lock (_gate)
        {
            if (_disposed) return;
            _disposed = true;
            _flushTimer?.Dispose();
            _flushTimer = null;
        }
        UiStateDocument snapshot;
        lock (_gate)
        {
            if (!_dirty || _futureVersion) return;
            snapshot = Clone(_document);
            _dirty = false;
        }
        try
        {
            var directory = Path.GetDirectoryName(_path);
            if (!string.IsNullOrWhiteSpace(directory)) Directory.CreateDirectory(directory);
            var temp = _path + ".tmp";
            File.WriteAllText(temp, JsonSerializer.Serialize(snapshot, JsonOptions));
            File.Move(temp, _path, true);
        }
        catch (Exception exception)
        {
            DiagnosticLog.Error("ui-state.write", exception, $"path={_path}");
        }
    }

    private void MarkDirtyLocked()
    {
        if (_futureVersion || _disposed) return;
        _dirty = true;
        _flushTimer ??= new Timer(_ => Flush(), null, TimeSpan.FromSeconds(1), Timeout.InfiniteTimeSpan);
    }

    private UiStateDocument Load()
    {
        try
        {
            if (!File.Exists(_path)) return new UiStateDocument();
            var parsed = JsonSerializer.Deserialize<UiStateDocument>(File.ReadAllText(_path), JsonOptions);
            if (parsed is null) return new UiStateDocument();
            parsed.Settings ??= new UiSettingsState();
            parsed.Projects ??= new(StringComparer.Ordinal);
            if (parsed.SchemaVersion > CurrentSchemaVersion)
            {
                _futureVersion = true;
                return parsed;
            }
            parsed.SchemaVersion = CurrentSchemaVersion;
            foreach (var project in parsed.Projects.Values)
            {
                project.RecentContent ??= [];
                project.RecentContent = project.RecentContent.Take(20).ToList();
                project.LeftRegion = project.LeftRegion is "closed" or "sessions" or "explorer" ? project.LeftRegion : "sessions";
                project.RightRegion = project.RightRegion is "closed" or "review" ? project.RightRegion : "review";
                project.BottomDrawer = project.BottomDrawer is "closed" or "git" or "providerTrace" or "toolActivity" ? project.BottomDrawer : "closed";
                project.WindowState = project.WindowState is "normal" or "maximized" or "fullscreen" ? project.WindowState : "normal";
            }
            return parsed;
        }
        catch (Exception exception)
        {
            DiagnosticLog.Error("ui-state.read", exception, $"path={_path}");
            try
            {
                if (File.Exists(_path)) File.Copy(_path, $"{_path}.corrupt-{DateTimeOffset.UtcNow:yyyyMMddHHmmssfff}", true);
            }
            catch (Exception backupException)
            {
                DiagnosticLog.Error("ui-state.backup", backupException, $"path={_path}");
            }
            return new UiStateDocument();
        }
    }

    private static T Clone<T>(T value) => JsonSerializer.Deserialize<T>(JsonSerializer.Serialize(value, JsonOptions), JsonOptions)!;
}
