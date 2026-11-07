using System.Collections.ObjectModel;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

// Net file diff of one turn: each changed file's first-touch baseline against
// its latest content. Drawer visibility stays with the window layout.
public sealed class TurnDiffViewModel : ObservableObject
{
    private readonly IViewModelHost _host;
    private IReadOnlyDictionary<string, TurnFileDiff> _diffs = new Dictionary<string, TurnFileDiff>(StringComparer.Ordinal);
    private CancellationTokenSource? _load;
    private TurnChangeItem? _selectedfile;
    private string _sessionId = string.Empty;
    private string _turnId = string.Empty;
    private string _state = "idle";
    private string _error = string.Empty;
    
    internal TurnDiffViewModel(IViewModelHost host) => _host = host;
    
    public ObservableCollection<TurnChangeItem> Files { get; } = [];
    public ObservableCollection<DiffLineItem> DiffLines { get; } = [];
    
    public string SessionId => _sessionId;
    public string TurnId => _turnId;

    public TurnChangeItem? SelectedFile
    {
        get => _selectedfile;
        set
        {
            if (!SetProperty(ref _selectedfile, value)) return;
            RenderSelectedFile();
        }
    }

    public string State
    {
        get => _state;
        private set
        {
            if (SetProperty(ref _state, value)) NotifyPresentationChanged();
        }
    }
    
    public string Error { get => _error; private set { if (SetProperty(ref _error, value)) OnPropertyChanged(nameof(EmptyMessage)); } }
    public bool IsLoading => State == "loading";
    public bool HasDiffLines => State == "ready" && DiffLines.Count > 0;
    public bool ShowDiffEmpty => !HasDiffLines;
    public bool ShowDiffStats => State == "ready" && SelectedFile is { Binary: false };
    public string FileCountText => $"{Files.Count} {(Files.Count == 1 ? "file" : "files")}";
    public string SelectedPath => SelectedFile?.Path ?? "No file selected";
    public string SelectedAdditionsText => SelectedFile is { } file ? $"+{file.Additions}" : string.Empty;
    public string SelectedDeletionsText => SelectedFile is { } file ? $"-{file.Deletions}" : string.Empty;
    public string EmptyMessage
    {
        get
        {
            if (State == "loading") return "Loading turn changes...";
            if (State == "error") return string.IsNullOrWhiteSpace(Error) ? "Turn changes are unavailable." : Error;
            if (State == "ready" && Files.Count == 0) return "This turn changed no files, or its changes have expired.";
            if (SelectedFile is { Binary: true }) return "Binary files cannot be displayed as text.";
            if (SelectedFile is not null) return "No textual changes.";
            return "Select a changed file to inspect its diff.";
        }
    }

    /// <summary>Loads the turn's net diff and selects <paramref name="path"/> when present.</summary>
    public async Task LoadAsync(string sessionId, string turnId, string? path = null)
    {
        if (_host.EnsureSdk()) return;
        _load?.Cancel();
        _load?.Dispose();
        var request = new CancellationTokenSource();
        _load = request;
        var selectedPath = path ?? (turnId == _turnId ? SelectedFile?.Path : null);
        _sessionId = sessionId;
        _turnId = turnId;
        OnPropertyChanged(nameof(SessionId));
        OnPropertyChanged(nameof(TurnId));
        State = "loading";
        Error = string.Empty;
        try
        {
            var diff = await _host.Sdk!.GetTurnDiffAsync(sessionId, turnId);
            if (request.IsCancellationRequested) return;
            _diffs = diff.Files.ToDictionary(file => file.Path, StringComparer.Ordinal);
            Files.Clear();
            foreach (var file in diff.Files)
                Files.Add(new TurnChangeItem(file.Path, file.Status, file.Additions, file.Deletions, file.Binary));
            OnPropertyChanged(nameof(FileCountText));
            State = "ready";
            var selected = Files.FirstOrDefault(file => file.Path == selectedPath) ?? Files.FirstOrDefault();
            if (selected == SelectedFile) RenderSelectedFile();
            else SelectedFile = selected;
        }
        catch (Exception exception) when (!request.IsCancellationRequested)
        {
            DiagnosticLog.Error("turn_diff.load", exception, $"session={sessionId} turn={turnId}");
            _diffs = new Dictionary<string, TurnFileDiff>(StringComparer.Ordinal);
            Files.Clear();
            SelectedFile = null;
            OnPropertyChanged(nameof(FileCountText));
            State = "error";
            Error = exception.Message;
        }
        catch (Exception) when (request.IsCancellationRequested) { }
    }

    internal void Clear()
    {
        _load?.Cancel();
        _load?.Dispose();
        _load = null;
        _diffs = new Dictionary<string, TurnFileDiff>(StringComparer.Ordinal);
        _sessionId = string.Empty;
        _turnId = string.Empty;
        Files.Clear();
        SelectedFile = null;
        DiffLines.Clear();
        Error = string.Empty;
        State = "idle";
        OnPropertyChanged(nameof(FileCountText));
        OnPropertyChanged(nameof(SessionId));
        OnPropertyChanged(nameof(TurnId));
    }
    
    private void RenderSelectedFile()
    {
        DiffLines.Clear();
        if (SelectedFile is { Binary:true } file && _diffs.TryGetValue(file.Path, out var diff))
        {
            foreach (var hunk in diff.Hunks)
            {
                DiffLines.Add(new DiffLineItem("hunk",
                        $"@@ -{hunk.OldStart},{hunk.OldLines} +{hunk.NewStart},{hunk.NewLines} @@", string.Empty, string.Empty));
                foreach (var line in hunk.Lines)
                    DiffLines.Add(new DiffLineItem(line.Kind, line.Text,
                        line.OldLine?.ToString() ?? string.Empty, line.NewLine?.ToString() ?? string.Empty));
            }
        }
        OnPropertyChanged(nameof(SelectedPath));
        OnPropertyChanged(nameof(SelectedAdditionsText));
        OnPropertyChanged(nameof(SelectedDeletionsText));
        NotifyPresentationChanged();
    }

    private void NotifyPresentationChanged()
    {
        OnPropertyChanged(nameof(IsLoading));
        OnPropertyChanged(nameof(HasDiffLines));
        OnPropertyChanged(nameof(ShowDiffEmpty));
        OnPropertyChanged(nameof(ShowDiffStats));
        OnPropertyChanged(nameof(EmptyMessage));
    }

}