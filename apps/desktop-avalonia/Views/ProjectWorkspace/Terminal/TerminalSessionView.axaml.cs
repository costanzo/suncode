using Avalonia;
using Avalonia.Controls;
using Avalonia.VisualTree;
using Avalonia.Styling;
using Avalonia.Threading;
using RoyalTerminal.Terminal;
using RoyalTerminal.Terminal.Theming;
using SkiaSharp;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.ProjectWorkspace.Terminal;

public sealed partial class TerminalSessionView : UserControl
{
    private static readonly Lazy<string> SystemTerminalFont = new(ResolveSystemTerminalFont);

    private static string ResolveSystemTerminalFont()
    {
        string[] preferred = OperatingSystem.IsMacOS()
            ? ["Menlo", "Monaco", "Courier"]
            : OperatingSystem.IsWindows()
                ? ["Consolas", "Cascadia Mono", "Courier New"]
                : ["DejaVu Sans Mono", "Liberation Mono", "Noto Sans Mono"];
        using var manager = SKFontManager.CreateDefault();
        var installed = manager.FontFamilies.ToHashSet(StringComparer.OrdinalIgnoreCase);
        // Check installed families first: a missing name can silently match a
        // proportional system default. Fall back only to another fixed-pitch face.
        foreach (var family in preferred.Concat(installed.OrderBy(name => name, StringComparer.OrdinalIgnoreCase)))
        {
            if (!installed.Contains(family)) continue;
            using var face = manager.MatchFamily(family, SKFontStyle.Normal);
            if (face?.IsFixedPitch == true) return face.FamilyName;
        }
        throw new InvalidOperationException("No system monospace font is available. Install a monospace font and restart SunCode.");
    }

    // ANSI order: black, red, green, yellow, blue, magenta, cyan, white,
    // followed by their bright variants. Keep aligned with DESIGN.md and the terminal specimen.
    private static readonly TerminalTheme LightTheme = TerminalTheme.FromBase16(
        new uint[]
        {
            0xFF26313A, 0xFFB42318, 0xFF287667, 0xFF805B00,
            0xFF245EAD, 0xFF8A3FA0, 0xFF006B78, 0xFF4B5563,
            0xFF65717D, 0xFFC12D22, 0xFF216E39, 0xFF856000,
            0xFF245EAD, 0xFF8A3FA0, 0xFF006B78, 0xFF26313A,
        },
        defaultForeground: 0xFF26313A,
        defaultBackground: 0xFFF2F5F7,
        cursorColor: 0xFF26313A,
        selectionForeground: 0xFF26313A,
        selectionBackground: 0xFFBAC6D0);

    private static readonly TerminalTheme DarkTheme = TerminalTheme.Dark
        .WithDefaultBackground(0xFF0D1115)
        .WithDefaultForeground(0xFFD9E0E6);

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
        Terminal.SizeChanged += (_, _) => QueueStartSession();
        Terminal.EffectiveViewportChanged += (_, _) => QueueStartSession();
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
        Terminal.Theme = dark ? DarkTheme : LightTheme;
        Terminal.VtProcessorPreference = VtProcessorPreference.Managed;
    }

    private async void StartSession()
    {
        if (_started || _session is null || _session.HasExited || string.IsNullOrWhiteSpace(_session.WorkingDirectory)) return;
        if (VisualRoot is null || !Terminal.IsEffectivelyVisible) return;
        Terminal.UpdateLayout();
        // Hidden/restored tabs may be attached before their first measured layout.
        if (!Terminal.IsArrangeValid || Terminal.Bounds.Width <= 0 || Terminal.Bounds.Height <= 0) return;
        _started = true;
        _session.State = "starting";
        var session = _session;
        var generation = ++_generation;
        await _launchGate.WaitAsync();
        try
        {
            Terminal.FontFamilyName = SystemTerminalFont.Value;
            var shell = await (session.ResolveShellAsync?.Invoke() ?? throw new InvalidOperationException("Terminal shell resolver is unavailable"));
            if (generation != _generation || !ReferenceEquals(session, _session)) return;
            session.ShellLabel = shell.Family == "powershell" ? "PowerShell" : shell.Family;
            ApplyTheme();
            Terminal.UpdateLayout();
            var dimensions = GetSessionDimensions();
            var options = new PtyTransportOptions(
                Command: new TerminalCommandSpec(FileName: shell.Path, Arguments: shell.InteractiveArgs.ToArray()),
                WorkingDirectory: _session.WorkingDirectory,
                Environment: null,
                Dimensions: dimensions);
            await Terminal.StartSessionAsync(options);
            if (generation != _generation || !ReferenceEquals(session, _session)) { Terminal.StopPty(); return; }
            // A layout resize during asynchronous transport startup can precede the
            // active PTY. Reconcile that gap; RoyalTerminal owns subsequent resizes.
            Terminal.UpdateLayout();
            if (Terminal.Columns != dimensions.Columns || Terminal.Rows != dimensions.Rows)
                Terminal.Pty?.Resize(Terminal.Columns, Terminal.Rows);
            session.State = "running";
        }
        catch (Exception exception)
        {
            if (generation == _generation && ReferenceEquals(session, _session)) { session.State = "error"; session.ErrorMessage = exception.Message; _started = false; }
        }
        finally { _launchGate.Release(); }
    }

    private TerminalSessionDimensions GetSessionDimensions()
    {
        var renderer = Terminal.Renderer ?? throw new InvalidOperationException("Terminal renderer is unavailable");
        return new TerminalSessionDimensions(
            Terminal.Columns,
            Terminal.Rows,
            Math.Max(1, (int)Math.Round(Terminal.Columns * renderer.CellWidth)),
            Math.Max(1, (int)Math.Round(Terminal.Rows * renderer.CellHeight)));
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
        if (_session is not null) _session.State = "starting";
        StartSession();
    });
}
