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
    private static readonly object Gate = new();
    private static StreamWriter? _file;
    private static DiagnosticLogLevel _minimumLevel = DiagnosticLogLevel.Info;
    private static bool _initialized;

    public static void Initialize()
    {
        lock (Gate)
        {
            if (_initialized) return;
            _initialized = true;
            _minimumLevel = ParseLevel(Environment.GetEnvironmentVariable("SUNCODE_LOG_LEVEL"));
            try
            {
                var directory = Environment.GetEnvironmentVariable("SUNCODE_LOG_DIRECTORY");
                if (string.IsNullOrWhiteSpace(directory))
                {
                    directory = Environment.GetEnvironmentVariable("SUNCODE_DATA_DIRECTORY");
                }
                if (string.IsNullOrWhiteSpace(directory))
                {
                    directory = Path.Combine(
                        Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".suncode");
                }

                var logDirectory = Path.Combine(directory, "logs");
                Directory.CreateDirectory(logDirectory);
                var path = Path.Combine(logDirectory, "desktop.log");
                _file = new StreamWriter(new FileStream(
                    path, FileMode.Append, FileAccess.Write, FileShare.ReadWrite), Encoding.UTF8)
                {
                    AutoFlush = true
                };
            }
            catch (Exception exception)
            {
                Console.Error.WriteLine($"[suncode][logger][ERROR] file_init_failed type={exception.GetType().Name} message={exception.Message}");
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
