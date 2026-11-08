using System.Collections.ObjectModel;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Infrastructure.Git;
using SunCode.Desktop.Models;

namespace SunCode.Desktop.ViewModels;

public sealed class DiffReviewerNode(ExplorerNode entry, GitFileItem? file = null)
{
    public ExplorerNode Entry { get; } = entry;
    public GitFileItem? File { get; } = file;
    public ObservableCollection<DiffReviewerNode> Children { get; } = [];
    public string Status => File?.StatusLetter ?? string.Empty;
    public bool IsAdded => File?.IsSuccess ?? false;
    public bool IsDeleted => File?.IsDanger ?? false;
    public bool IsModified => File?.IsWarning ?? false;
}

public sealed class DiffReviewerViewModel : ObservableObject, IDisposable
{
    private readonly IGitService _service;
    private readonly ProjectItem _project;
    private CancellationTokenSource? _request;
    private bool _disposed;
    private string _state = "idle";
    private string _error = string.Empty;
    private string _scope = "all";
    private string _branch = string.Empty;
    private DiffReviewerNode? _selected;
    private int _currentChange;
    private GitDiffResult? _diff;

    internal DiffReviewerViewModel(IGitService service, ProjectItem project)
    {
        _service = service;
        _project = project;
    }

    public event Action? CloseRequested;
    public event Action? DocumentChanged;
    public event Action<int>? ChangeRequested;
    public ObservableCollection<DiffReviewerNode> Roots { get; } = [];
    internal DiffReviewDocument? Document { get; private set; }
    public string ProjectName => _project.DisplayName;
    public string Branch => _branch;
    public string Scope => _scope;
    public bool ScopeAll => Scope == "all";
    public bool ScopeStaged => Scope == "staged";
    public bool ScopeUnstaged => Scope == "unstaged";
    public string BeforeTitle => ScopeUnstaged && _selected?.File?.Conflicted != true
        ? LocalizationService.GetString("LocGitIndex", "Index") : "HEAD";
    public string AfterTitle => Scope == "staged" ? LocalizationService.GetString("LocGitIndex", "Index") : LocalizationService.GetString("LocGitWorkingTree", "Working tree");
    public DiffReviewerNode? SelectedNode
    {
        get => _selected;
        set
        {
            if (value?.File is null || !SetProperty(ref _selected, value)) return;
            _ = SelectAsync(value);
        }
    }
    public string SelectedPath => _selected?.File?.Path ?? string.Empty;
    public string OldPath => _selected?.File?.OldPath ?? string.Empty;
    public bool HasOldPath => OldPath.Length > 0;
    public string AdditionsText => $"+{_diff?.Additions ?? 0}";
    public string DeletionsText => $"−{_diff?.Deletions ?? 0}";
    public int FileCount { get; private set; }
    public string FileCountText => string.Format(LocalizationService.GetString("LocDiffFileCount", "{0} files"), FileCount);
    public bool IsLoading => _state == "loading";
    public bool CanSelect => _state != "loading";
    public bool HasDocument => _state == "ready" && Document is { ChangeCount: > 0 };
    public bool ShowMessage => !HasDocument;
    public bool ShowStats => _state == "ready" && _diff is { Binary: false };
    public bool IsLimited { get; private set; }
    public bool StatusLimited { get; private set; }
    public bool CanPrevious => HasDocument && _currentChange > 0;
    public bool CanNext => HasDocument && _currentChange + 1 < Document!.ChangeCount;
    public string ChangePosition => HasDocument ? $"{_currentChange + 1} / {Document!.ChangeCount}" : "0 / 0";
    public string Message => _state switch
    {
        "loading" => LocalizationService.GetString("LocDiffLoading", "Reading changes…"),
        "error" => _error,
        _ when _selected is null => LocalizationService.GetString(FileCount == 0 ? "LocDiffNoChanges" : "LocDiffSelectFile", FileCount == 0 ? "No changes in this Git range." : "Select a changed file."),
        _ when _diff?.Binary == true => LocalizationService.GetString("LocDiffBinary", "Binary files cannot be displayed as text."),
        _ when IsLimited => LocalizationService.GetString("LocDiffTruncated", "Diff truncated at the review limit"),
        _ => LocalizationService.GetString("LocDiffNoTextChanges", "No textual differences.")
    };

    internal async Task OpenAsync(string scope, string? path)
    {
        _scope = NormalizeScope(scope);
        Notify();
        await RefreshAsync(path);
    }

    public async Task SetScopeAsync(string scope)
    {
        var normalized = NormalizeScope(scope);
        if (_scope == normalized) return;
        _scope = normalized;
        _currentChange = 0;
        Notify();
        await RefreshAsync(SelectedPath);
    }

    private static string NormalizeScope(string scope) => scope is "staged" or "unstaged" ? scope : "all";

    public async Task RefreshAsync(string? path = null)
    {
        if (_disposed) return;
        var token = BeginRequest();
        var selectedPath = path ?? SelectedPath;
        var change = _currentChange;
        _state = "loading";
        Notify();
        try
        {
            if (!await _service.CheckAvailabilityAsync(token))
                throw new GitException("git_missing", LocalizationService.GetString("LocDiffGitMissing", "Git is unavailable."));
            var status = await _service.StatusAsync(_project.CanonicalRoot, token);
            token.ThrowIfCancellationRequested();
            _branch = status.Branch ?? LocalizationService.GetString("LocGitDetachedHead", "Detached HEAD");
            StatusLimited = status.Truncated;
            BuildTree(status.Files);
            _selected = Flatten(Roots).FirstOrDefault(node => node.File?.Path == selectedPath)
                ?? Flatten(Roots).FirstOrDefault(node => node.File is not null);
            await ReadSelectedAsync(token, change);
        }
        catch (Exception exception) { HandleFailure(exception, token); }
    }

    private void BuildTree(IReadOnlyList<GitChange> files)
    {
        var expansion = Flatten(Roots).ToDictionary(node => node.Entry.Path, node => node.Entry.IsExpanded, StringComparer.Ordinal);
        var nodes = new Dictionary<string, DiffReviewerNode>(StringComparer.Ordinal);
        Roots.Clear();
        FileCount = 0;
        foreach (var file in files.Where(file => ScopeAll || (ScopeStaged ? file.Staged : file.Unstaged)).OrderBy(file => file.Path, StringComparer.Ordinal))
        {
            FileCount++;
            var parts = file.Path.Split('/');
            DiffReviewerNode? parent = null;
            for (var index = 0; index < parts.Length; index++)
            {
                var path = string.Join('/', parts.Take(index + 1));
                if (!nodes.TryGetValue(path, out var node))
                {
                    var isFile = index == parts.Length - 1;
                    var entry = new ExplorerNode(parts[index], path, isFile ? "file" : "directory");
                    entry.Children.Clear();
                    entry.IsLoaded = true;
                    entry.IsExpanded = expansion.GetValueOrDefault(path, true);
                    node = new(entry, isFile ? new GitFileItem(file.Path, file.Status, file.Staged, file.Unstaged,
                        file.Conflicted, file.Additions, file.Deletions, file.OldPath ?? string.Empty, file.Binary) : null);
                    nodes.Add(path, node);
                    (parent?.Children ?? Roots).Add(node);
                }
                parent = node;
            }
        }
    }

    private static IEnumerable<DiffReviewerNode> Flatten(IEnumerable<DiffReviewerNode> nodes)
    {
        foreach (var node in nodes)
        {
            yield return node;
            foreach (var child in Flatten(node.Children)) yield return child;
        }
    }

    internal async Task SelectAsync(DiffReviewerNode node)
    {
        if (_disposed) return;
        var token = BeginRequest();
        _selected = node;
        _state = "loading";
        Notify();
        try { await ReadSelectedAsync(token, 0); }
        catch (Exception exception) { HandleFailure(exception, token); }
    }

    private async Task ReadSelectedAsync(CancellationToken token, int change)
    {
        GitDiffResult? diff = null;
        DiffReviewDocument? document = null;
        if (_selected?.File is { } file)
        {
            diff = await _service.DiffAsync(_project.CanonicalRoot, Scope, file.Path, token, includeUnchanged: true);
            document = await Task.Run(() => DiffReviewDocument.Create(diff, token), token);
        }
        token.ThrowIfCancellationRequested();
        _diff = diff;
        Document = document;
        IsLimited = diff?.Truncated ?? false;
        _currentChange = Math.Clamp(change, 0, Math.Max(0, (document?.ChangeCount ?? 0) - 1));
        _state = "ready";
        Notify();
        DocumentChanged?.Invoke();
        if (HasDocument) ChangeRequested?.Invoke(_currentChange);
    }

    public void MoveChange(int direction)
    {
        if (!HasDocument) return;
        _currentChange = Math.Clamp(_currentChange + direction, 0, Document!.ChangeCount - 1);
        Notify();
        ChangeRequested?.Invoke(_currentChange);
    }

    internal void TrackChange(int index)
    {
        if (!HasDocument || index == _currentChange) return;
        _currentChange = Math.Clamp(index, 0, Document!.ChangeCount - 1);
        Notify();
    }

    internal void ToggleContext(DiffReviewBlock block)
    {
        block.Expanded = !block.Expanded;
        Document?.Rebuild();
        DocumentChanged?.Invoke();
    }

    private CancellationToken BeginRequest()
    {
        _request?.Cancel();
        _request?.Dispose();
        _request = new();
        return _request.Token;
    }

    private void HandleFailure(Exception exception, CancellationToken token)
    {
        if (token.IsCancellationRequested || _disposed) return;
        _state = "error";
        _error = exception is OperationCanceledException
            ? LocalizationService.GetString("LocDiffReadFailed", "Could not read this comparison. Refresh to retry.") : exception.Message;
        Document = null;
        _diff = null;
        Notify();
        DocumentChanged?.Invoke();
    }

    internal void Notify()
    {
        foreach (var property in new[] { nameof(Branch), nameof(SelectedNode), nameof(SelectedPath), nameof(OldPath), nameof(HasOldPath),
            nameof(ScopeAll), nameof(ScopeStaged), nameof(ScopeUnstaged), nameof(BeforeTitle), nameof(AfterTitle), nameof(FileCountText),
            nameof(IsLoading), nameof(CanSelect), nameof(HasDocument), nameof(ShowMessage), nameof(ShowStats), nameof(IsLimited), nameof(StatusLimited),
            nameof(CanPrevious), nameof(CanNext), nameof(ChangePosition), nameof(Message), nameof(AdditionsText), nameof(DeletionsText) })
            OnPropertyChanged(property);
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;
        _request?.Cancel();
        _request?.Dispose();
        CloseRequested?.Invoke();
        CloseRequested = null;
        DocumentChanged = null;
        ChangeRequested = null;
    }
}
