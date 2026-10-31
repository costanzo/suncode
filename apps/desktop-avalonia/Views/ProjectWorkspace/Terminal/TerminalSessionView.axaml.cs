using Avalonia;
using Avalonia.Controls;
using Avalonia.Media;
using Avalonia.VisualTree;
using Avalonia.Styling;
using Avalonia.Threading;
using RoyalTerminal.Terminal;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.ProjectWorkspace.Terminal;

public sealed partial class TerminalSessionView : UserControl
{
    private TerminalSessionViewModel? _session;
    private bool _started;
    private bool _stopping;
    private int _generation;
    private readonly SemaphoreSlim _launchGate = new(1, 1);

    public TerminalSessionView()
    {
        InitializeComponent();
        DataContextChanged += (_, _) => BindSession();
        AttachedToVisualTree += (_, _) => QueueStartSession();
        ActualThemeVariantChanged += (_, _) => QueueApplyTheme();
        Terminal.ProcessExited += ProcessExited;
    }

    private void BindSession()
    {
        if (_session is not null) { _session.RestartRequested -= Restart; _session.StopRequested -= StopSession; StopSession(); }
        _session = DataContext as TerminalSessionViewModel;
        if (_session is not null)
        {
            _session.RestartRequested += Restart;
            _session.StopRequested += StopSession;
        }
        QueueApplyTheme();
        if (VisualRoot is not null) QueueStartSession();
    }

    private void QueueApplyTheme() => Dispatcher.UIThread.Post(ApplyTheme, DispatcherPriority.Background);

    private void QueueStartSession()
    {
        Dispatcher.UIThread.Post(StartSession, DispatcherPriority.Background);
    }

    private void ApplyTheme()
    {
        var dark = ActualThemeVariant == ThemeVariant.Dark;
        Terminal.DefaultBackground = Color.Parse(dark ? "#0D1115" : "#F2F5F7");
        Terminal.DefaultForeground = Color.Parse(dark ? "#D9E0E6" : "#26313A");
        Terminal.VtProcessorPreference = VtProcessorPreference.Managed;
    }

    private async void StartSession()
    {
        if (_started || _session is null || string.IsNullOrWhiteSpace(_session.WorkingDirectory)) return;
        _started = true;
        _session.State = "starting";
        var session = _session;
        var generation = ++_generation;
        await _launchGate.WaitAsync();
        try
        {
            var shell = await (session.ResolveShellAsync?.Invoke() ?? throw new InvalidOperationException("Terminal shell resolver is unavailable"));
            if (generation != _generation || !ReferenceEquals(session, _session)) return;
            session.ShellLabel = shell.Family == "powershell" ? "PowerShell" : shell.Family;
            ApplyTheme();
            var options = new PtyTransportOptions(
                Command: new TerminalCommandSpec(FileName: shell.Path, Arguments: shell.InteractiveArgs.ToArray()),
                WorkingDirectory: _session.WorkingDirectory,
                Environment: null,
                Dimensions: new TerminalSessionDimensions(120, 36, 960, 576));
            await Terminal.StartSessionAsync(options);
            if (generation != _generation || !ReferenceEquals(session, _session)) { Terminal.StopPty(); return; }
            session.State = "running";
        }
        catch (Exception exception)
        {
            if (generation == _generation && ReferenceEquals(session, _session)) { session.State = "error"; session.ErrorMessage = exception.Message; _started = false; }
        }
        finally { _launchGate.Release(); }
    }

    private void ProcessExited(object? sender, int exitCode)
    {
        if (_session is null || _stopping) return;
        _session.ExitCode = exitCode;
        _session.State = "exited";
        _started = false;
    }

    private void StopSession()
    {
        ++_generation;
        if (!_started && !Terminal.HasActiveSession) return;
        _stopping = true;
        try { Terminal.StopPty(); }
        catch { /* Process cleanup remains best effort during view disposal. */ }
        finally
        {
            _started = false;
            _stopping = false;
        }
    }

    private void RestartFailedSession(object? sender, Avalonia.Interactivity.RoutedEventArgs e) => Restart();

    private void Restart() => Dispatcher.UIThread.Post(() =>
    {
        StopSession();
        StartSession();
    });
}
