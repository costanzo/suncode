using Avalonia;
using Avalonia.Threading;
using SunCode.Desktop.Infrastructure;

namespace SunCode.Desktop;

internal static class Program
{
    [STAThread]
    public static void Main(string[] args)
    {
        DiagnosticLog.Initialize();
        AppDomain.CurrentDomain.UnhandledException += OnUnhandledException;
        TaskScheduler.UnobservedTaskException += OnUnobservedTaskException;
        Dispatcher.UIThread.UnhandledException += OnDispatcherUnhandledException;
        DiagnosticLog.Info("app", "started");
        try
        {
            BuildAvaloniaApp().StartWithClassicDesktopLifetime(args);
        }
        catch (Exception exception)
        {
            DiagnosticLog.Error("app.main", exception, "lifetime=classic_desktop");
            throw;
        }
        finally
        {
            DiagnosticLog.Info("app", "stopped");
        }
    }

    private static void OnUnhandledException(object? sender, UnhandledExceptionEventArgs args)
        => DiagnosticLog.Error("app.unhandled", args.ExceptionObject as Exception ?? new Exception(args.ExceptionObject?.ToString() ?? "unknown exception"), $"is_terminating={args.IsTerminating}");

    private static void OnUnobservedTaskException(object? sender, UnobservedTaskExceptionEventArgs args)
    {
        DiagnosticLog.Error("task.unobserved", args.Exception, "observed=false");
        args.SetObserved();
    }

    private static void OnDispatcherUnhandledException(object? sender, DispatcherUnhandledExceptionEventArgs args)
    {
        DiagnosticLog.Error("ui.dispatcher", args.Exception, "handled=true");
        args.Handled = true;
    }

    public static AppBuilder BuildAvaloniaApp() =>
        AppBuilder.Configure<App>()
            .UsePlatformDetect()
            .With(new MacOSPlatformOptions { DisableSetProcessName = true, ShowInDock = true })
            .WithInterFont()
            .LogToTrace();
}
