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
        try
        {
            ApplyTheme();
            var options = new PtyTransportOptions(
                Command: null,
                WorkingDirectory: _session.WorkingDirectory,
                Environment: null,
                Dimensions: new TerminalSessionDimensions(120, 36, 960, 576));
            await Terminal.StartSessionAsync(options);
            _session.State = "running";
        }
        catch
        {
            _session.State = "error";
            _started = false;
        }
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

    private void Restart() => Dispatcher.UIThread.Post(() =>
    {
        StopSession();
        StartSession();
    });
}
