using Avalonia;
using Avalonia.Threading;
using SunCode.Desktop.Infrastructure;

namespace SunCode.Desktop;

internal static class Program
{
    internal static DesktopInstanceCoordinator? InstanceCoordinator { get; private set; }

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
            var activation = ParseActivation(args) ?? DesktopActivationRequest.Application();
            InstanceCoordinator = DesktopInstanceCoordinator.Acquire(AppDataPaths.DataDirectory);
            if (!InstanceCoordinator.IsPrimary)
            {
                if (!InstanceCoordinator.ForwardAsync(activation).GetAwaiter().GetResult())
                    DiagnosticLog.Warn("desktop.ipc", "secondary_forward_failed=true");
                return;
            }
            InstanceCoordinator.StartServer();
            BuildAvaloniaApp().StartWithClassicDesktopLifetime(args);
        }
        catch (Exception exception)
        {
            DiagnosticLog.Error("app.main", exception, "lifetime=classic_desktop");
            throw;
        }
        finally
        {
            InstanceCoordinator?.Dispose();
            InstanceCoordinator = null;
            DiagnosticLog.Info("app", "stopped");
        }
    }

    private static DesktopActivationRequest? ParseActivation(string[] args)
    {
        for (var index = 0; index < args.Length; index++)
        {
            var value = args[index];
            if (value.StartsWith("suncode://activate/", StringComparison.OrdinalIgnoreCase))
            {
                var uri = new Uri(value);
                var payload = uri.Query.TrimStart('?')
                    .Split('&', StringSplitOptions.RemoveEmptyEntries)
                    .Select(part => part.Split('=', 2))
                    .FirstOrDefault(part => part.Length == 2 && part[0] == "payload")?[1];
                if (DesktopActivationRequest.TryParseLaunchArgument(Uri.UnescapeDataString(payload ?? string.Empty), out var uriRequest))
                    return uriRequest;
            }
            if (value == "--suncode-activate" && index + 1 < args.Length
                && DesktopActivationRequest.TryParseLaunchArgument(args[index + 1], out var request))
                return request;
        }
        return null;
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
