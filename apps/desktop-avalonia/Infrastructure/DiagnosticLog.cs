using System.Globalization;
using System.Text;

namespace SunCode.Desktop.Infrastructure;

internal enum DiagnosticLogLevel
{
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Off = 5
}

internal static class DiagnosticLog
{
    private const long DefaultMaxBytes = 10 * 1024 * 1024;
    private const int DefaultRetention = 5;
    private static readonly object Gate = new();
    private static StreamWriter? _file;
    private static string? _filePath;
    private static long _maxBytes = DefaultMaxBytes;
    private static int _retention = DefaultRetention;
    private static DiagnosticLogLevel _minimumLevel = DiagnosticLogLevel.Info;
    private static bool _initialized;

    public static void Initialize()
    {
        lock (Gate)
        {
            if (_initialized) return;
            _initialized = true;
        }
    }

    public static void Configure(string level, string? directory, long maxBytes, int retention)
    {
        Initialize();
        lock (Gate)
        {
            _minimumLevel = ParseLevel(level);
            _maxBytes = maxBytes >= 1024 ? maxBytes : DefaultMaxBytes;
            _retention = retention is >= 0 and <= 100 ? retention : DefaultRetention;
            _file?.Dispose();
            _file = null;
            try
            {
                var logDirectory = string.IsNullOrWhiteSpace(directory)
                    ? DefaultLogDirectory()
                    : directory;
                Directory.CreateDirectory(logDirectory);
                _filePath = Path.Combine(logDirectory, "desktop.log");
                _file = OpenFile(_filePath);
            }
            catch (Exception exception)
            {
                Console.Error.WriteLine($"[suncode][logger][ERROR] file_configure_failed type={exception.GetType().Name} message={exception.Message}");
            }
        }
    }

    public static void Trace(string area, string message) => Write(DiagnosticLogLevel.Trace, area, message);
    public static void Debug(string area, string message) => Write(DiagnosticLogLevel.Debug, area, message);
    public static void Info(string area, string message) => Write(DiagnosticLogLevel.Info, area, message);
    public static void Warn(string area, string message) => Write(DiagnosticLogLevel.Warn, area, message);
    public static void Error(string area, string message) => Write(DiagnosticLogLevel.Error, area, message);

    // Compatibility entry point for callers that do not need a more specific level.
    public static void Write(string area, string message)
        => Write(DiagnosticLogLevel.Info, area, message);

    public static void Write(DiagnosticLogLevel level, string area, string message)
    {
        Initialize();
        if (level < _minimumLevel || _minimumLevel == DiagnosticLogLevel.Off) return;

        var line = $"[suncode][{DateTimeOffset.Now.ToString("yyyy-MM-dd'T'HH:mm:ss.fffzzz", CultureInfo.InvariantCulture)}]"
            + $"[{LevelName(level)}][pid={Environment.ProcessId}][tid={Environment.CurrentManagedThreadId}][{area}] {message}";
        lock (Gate)
        {
            try
            {
                RotateIfNeeded(line);
                _file?.WriteLine(line);
            }
            catch (Exception exception)
            {
                Console.Error.WriteLine($"[suncode][logger][ERROR] file_write_failed type={exception.GetType().Name} message={exception.Message}");
            }

            Console.Error.WriteLine(line);
            Console.Error.Flush();
        }
    }

    private static DiagnosticLogLevel ParseLevel(string? value) => value?.Trim().ToUpperInvariant() switch
    {
        "TRACE" => DiagnosticLogLevel.Trace,
        "DEBUG" => DiagnosticLogLevel.Debug,
        "INFO" or null or "" => DiagnosticLogLevel.Info,
        "WARN" or "WARNING" => DiagnosticLogLevel.Warn,
        "ERROR" => DiagnosticLogLevel.Error,
        "OFF" or "NONE" => DiagnosticLogLevel.Off,
        _ => DiagnosticLogLevel.Info
    };

    private static string DefaultLogDirectory()
    {
        var dataDirectory = Environment.GetEnvironmentVariable("SUNCODE_DATA_DIRECTORY");
        if (string.IsNullOrWhiteSpace(dataDirectory))
        {
            dataDirectory = Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".suncode");
        }
        return Path.Combine(dataDirectory, "logs");
    }

    private static StreamWriter OpenFile(string path) => new(new FileStream(
        path, FileMode.Append, FileAccess.Write, FileShare.ReadWrite), Encoding.UTF8)
    {
        AutoFlush = true
    };

    private static void RotateIfNeeded(string line)
    {
        if (_file is null || _filePath is null) return;
        _file.Flush();
        var incomingBytes = Encoding.UTF8.GetByteCount(line) + Encoding.UTF8.GetByteCount(Environment.NewLine);
        if (_file.BaseStream.Length + incomingBytes <= _maxBytes) return;

        _file.Dispose();
        _file = null;
        if (_retention == 0)
        {
            File.Delete(_filePath);
        }
        else
        {
            var oldest = $"{_filePath}.{_retention}";
            if (File.Exists(oldest)) File.Delete(oldest);
            for (var index = _retention - 1; index >= 1; index--)
            {
                var source = $"{_filePath}.{index}";
                if (File.Exists(source)) File.Move(source, $"{_filePath}.{index + 1}");
            }
            if (File.Exists(_filePath)) File.Move(_filePath, $"{_filePath}.1");
        }
        _file = OpenFile(_filePath);
    }

    private static string LevelName(DiagnosticLogLevel level) => level switch
    {
        DiagnosticLogLevel.Trace => "TRACE",
        DiagnosticLogLevel.Debug => "DEBUG",
        DiagnosticLogLevel.Info => "INFO",
        DiagnosticLogLevel.Warn => "WARN",
        DiagnosticLogLevel.Error => "ERROR",
        _ => "OFF"
    };
}
