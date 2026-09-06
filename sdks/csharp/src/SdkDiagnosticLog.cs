namespace SunCode.Sdk;

internal static class SdkDiagnosticLog
{
    public static void Trace(string area, string message) => Write("TRACE", area, message);
    public static void Debug(string area, string message) => Write("DEBUG", area, message);
    public static void Info(string area, string message) => Write("INFO", area, message);
    public static void Error(string area, string message) => Write("ERROR", area, message);
    public static void Error(string area, Exception exception, string? context = null)
        => Write("ERROR", area, $"{context ?? "exception"} type={exception.GetType().Name} message={exception.Message}");

    private static void Write(string level, string area, string message)
        => Console.Error.WriteLine($"[suncode][{level}][{area}] {message}");
}
