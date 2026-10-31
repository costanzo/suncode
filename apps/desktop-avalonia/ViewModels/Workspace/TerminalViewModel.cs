using System.Collections.ObjectModel;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Sdk;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

public sealed class TerminalSessionViewModel : ObservableObject, IDisposable
{
    private bool _isSelected;
    private string _state = "starting";
    private int? _exitCode;

    internal TerminalSessionViewModel(string id, string workingDirectory)
    {
        Id = id;
        WorkingDirectory = workingDirectory;
    }

    public string Id { get; }
    public string WorkingDirectory { get; }
    private string _shellLabel = "shell";
    public string ShellLabel { get => _shellLabel; internal set => SetProperty(ref _shellLabel, value); }
    internal Func<Task<ResolvedShell>>? ResolveShellAsync { get; init; }
    public bool IsSelected { get => _isSelected; internal set => SetProperty(ref _isSelected, value); }
    public string State { get => _state; internal set { if (SetProperty(ref _state, value)) OnPropertyChanged(nameof(StateText)); OnPropertyChanged(nameof(HasError)); } }
    public int? ExitCode { get => _exitCode; internal set { if (SetProperty(ref _exitCode, value)) OnPropertyChanged(nameof(StateText)); } }
    private string _errorMessage = string.Empty;
    public string ErrorMessage { get => _errorMessage; internal set { if (SetProperty(ref _errorMessage, value)) OnPropertyChanged(nameof(StateText)); } }
    public bool IsRunning => State is "starting" or "running";
    public bool HasError => State == "error";
    public bool HasExited => State is "exited" or "error";
    public string StateText => State switch
    {
        "starting" => "Starting",
        "running" => "Running",
        "exited" => ExitCode is { } code ? $"Exited {code}" : "Exited",
        "error" => string.IsNullOrWhiteSpace(ErrorMessage) ? "Failed to start" : ErrorMessage,
        _ => State
    };

    internal event Action? CloseRequested;
    internal event Action? RestartRequested;
    internal event Action? StopRequested;
    internal void RequestClose() => CloseRequested?.Invoke();
    internal void RequestRestart() => RestartRequested?.Invoke();
    internal void RequestStop() => StopRequested?.Invoke();
    public void Dispose() { StopRequested?.Invoke(); CloseRequested = null; RestartRequested = null; StopRequested = null; }
}

public sealed class TerminalViewModel : ObservableObject, IDisposable
{
    private readonly IViewModelHost _host;
    private TerminalSessionViewModel? _selectedSession;
    private int _nextSessionNumber;

    internal TerminalViewModel(IViewModelHost host) => _host = host;

    public ObservableCollection<TerminalSessionViewModel> Sessions { get; } = [];
    public TerminalSessionViewModel? SelectedSession
    {
        get => _selectedSession;
        private set
        {
            if (ReferenceEquals(_selectedSession, value)) return;
            if (_selectedSession is not null) _selectedSession.IsSelected = false;
            _selectedSession = value;
            if (_selectedSession is not null) _selectedSession.IsSelected = true;
            OnPropertyChanged();
            OnPropertyChanged(nameof(HasSessions));
        }
    }
    public bool HasSessions => Sessions.Count > 0;
    public bool CanOpen => _host.SelectedProject is not null;
    public string WorkingDirectory => _host.SelectedProject?.CanonicalRoot ?? string.Empty;

    public TerminalSessionViewModel OpenSession()
    {
        if (!CanOpen) throw new InvalidOperationException("A project must be open before starting a terminal.");
        var session = new TerminalSessionViewModel(
            $"terminal-{++_nextSessionNumber}",
            WorkingDirectory)
        {
            ShellLabel = DefaultShellLabel(),
            ResolveShellAsync = ResolveShellAsync
        };
        session.CloseRequested += () => CloseSession(session);
        Sessions.Add(session);
        SelectedSession = session;
        return session;
    }

    public void SelectSession(TerminalSessionViewModel session)
    {
        if (Sessions.Contains(session)) SelectedSession = session;
    }

    public void CloseSession(TerminalSessionViewModel session)
    {
        if (!Sessions.Remove(session)) return;
        var next = SelectedSession == session ? Sessions.LastOrDefault() : SelectedSession;
        session.Dispose();
        SelectedSession = next;
    }

    public void CloseAll()
    {
        foreach (var session in Sessions.ToArray()) session.Dispose();
        Sessions.Clear();
        SelectedSession = null;
    }

    private async Task<ResolvedShell> ResolveShellAsync()
    {
        if (!await _host.EnsureSdkReadyAsync() || _host.Sdk is null) throw new InvalidOperationException("Agent SDK is unavailable");
        var result = await ShellSetupService.Current.CheckAsync(new("terminal"));
        return result.Shell ?? throw new SdkException(result.ErrorCode ?? "shell_unavailable", result.Message);
    }

    private static string DefaultShellLabel()
    {
        if (OperatingSystem.IsWindows()) return "PowerShell";
        var shell = Environment.GetEnvironmentVariable("SHELL");
        return string.IsNullOrWhiteSpace(shell) ? "shell" : Path.GetFileName(shell);
    }

    public void Dispose() => CloseAll();
}
