using Avalonia;
using SunCode.Desktop.Infrastructure;

namespace SunCode.Desktop;

internal static class Program
{
    [STAThread]
    public static void Main(string[] args)
    {
        DiagnosticLog.Initialize();
        DiagnosticLog.Info("app", "started");
        BuildAvaloniaApp().StartWithClassicDesktopLifetime(args);
        DiagnosticLog.Info("app", "stopped");
    }

    public static AppBuilder BuildAvaloniaApp() =>
        AppBuilder.Configure<App>()
            .UsePlatformDetect()
            .With(new MacOSPlatformOptions { DisableSetProcessName = true, ShowInDock = true })
            .WithInterFont()
            .LogToTrace();
}
