using System.Collections.ObjectModel;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Sdk;

namespace SunCode.Desktop.ViewModels;

// Git status, file filtering, and per-file diff state for one project window.
// Drawer visibility and list sizing stay with the window layout.
public sealed class GitReviewViewModel : ObservableObject
{
    private readonly IViewModelHost _host;
    private GitFileItem? _selectedFile;
    private string _state = "idle";
    private string _error = string.Empty;
    private string _diffState = "idle";
    private string _diffError = string.Empty;
    private string _patch = string.Empty;
    private string _scope = "all";
    private string _filter = string.Empty;
    private string _branch = string.Empty;
    private bool _statusTruncated;
    private bool _diffBinary;
    private bool _diffTruncated;
    private int _diffAdditions;
    private int _diffDeletions;
    private int _changedFiles;
    private int _additions;
    private int _deletions;

    internal GitReviewViewModel(IViewModelHost host) => _host = host;

    public ObservableCollection<GitFileItem> Files { get; } = [];
    public ObservableCollection<GitFileItem> FilteredFiles { get; } = [];
    public ObservableCollection<DiffLineItem> DiffLines { get; } = [];

    public GitFileItem? SelectedFile
    {
        get => _selectedFile;
        set
        {
            if (SetProperty(ref _selectedFile, value)) OnPropertyChanged(nameof(SelectedPath));
        }
    }

    public string State
    {
        get => _state;
        private set
        {
            if (!SetProperty(ref _state, value)) return;
            OnPropertyChanged(nameof(IsReady));
            OnPropertyChanged(nameof(IsClean));
            OnPropertyChanged(nameof(IsDirty));
            OnPropertyChanged(nameof(IsLoading));
            OnPropertyChanged(nameof(FooterBranchText));
            OnPropertyChanged(nameof(EmptyMessage));
        }
    }
    public string Error { get => _error; private set => SetProperty(ref _error, value); }
    public string DiffState
    {
        get => _diffState;
        private set
        {
            if (!SetProperty(ref _diffState, value)) return;
            NotifyDiffPresentationChanged();
        }
    }
    public string DiffError { get => _diffError; private set => SetProperty(ref _diffError, value); }
    public string Patch { get => _patch; private set => SetProperty(ref _patch, value); }
    public string Scope { get => _scope; private set => SetProperty(ref _scope, value); }
    public string Filter { get => _filter; private set { if (SetProperty(ref _filter, value)) OnPropertyChanged(nameof(EmptyMessage)); } }
    public string Branch { get => _branch; private set { if (SetProperty(ref _branch, value)) { OnPropertyChanged(nameof(FooterBranchText)); } } }
    public int ChangedFiles { get => _changedFiles; private set { if (SetProperty(ref _changedFiles, value)) { OnPropertyChanged(nameof(ChangeSummary)); OnPropertyChanged(nameof(IsClean)); OnPropertyChanged(nameof(IsDirty)); } } }
    public int Additions { get => _additions; private set => SetProperty(ref _additions, value); }
    public int Deletions { get => _deletions; private set => SetProperty(ref _deletions, value); }
    public bool StatusTruncated { get => _statusTruncated; private set => SetProperty(ref _statusTruncated, value); }
    public bool DiffBinary { get => _diffBinary; private set { if (SetProperty(ref _diffBinary, value)) NotifyDiffPresentationChanged(); } }
    public bool DiffTruncated { get => _diffTruncated; private set => SetProperty(ref _diffTruncated, value); }
    public int DiffAdditions { get => _diffAdditions; private set => SetProperty(ref _diffAdditions, value); }
    public int DiffDeletions { get => _diffDeletions; private set => SetProperty(ref _diffDeletions, value); }

    public bool HasFilteredFiles => FilteredFiles.Count > 0;
    public string FileCountText => $"{FilteredFiles.Count} {(FilteredFiles.Count == 1 ? "file" : "files")}";
    public bool IsReady => State == "ready";
    public bool IsClean => IsReady && ChangedFiles == 0;
    public bool IsDirty => IsReady && ChangedFiles > 0;
    public bool IsLoading => State == "loading" || DiffState == "loading";
    public bool HasDiffLines => DiffState == "ready" && !DiffBinary && DiffLines.Count > 0;
    public bool ShowDiffEmpty => !HasDiffLines;
    public bool ShowDiffStats => DiffState == "ready" && !DiffBinary;
    public string SelectedPath => SelectedFile?.Path ?? "No file selected";
    public string FooterBranchText => State switch
    {
        "loading" => "Reading Git...",
        "not_repository" => "Not a Git repository",
        "error" => "Git unavailable",
        _ => string.IsNullOrWhiteSpace(Branch) ? "Detached HEAD" : Branch
    };
    public string ChangeSummary => ChangedFiles == 0 ? "Clean" : $"{ChangedFiles} changed";
    public string EmptyMessage
    {
        get
        {
            if (State == "loading") return "Reading repository changes...";
            if (State == "not_repository") return "This project is not inside a Git repository.";
            if (State == "error") return string.IsNullOrWhiteSpace(Error) ? "Git status is unavailable." : Error;
            if (FilteredFiles.Count == 0 && Filter.Length > 0) return "No changed files match this filter.";
            if (FilteredFiles.Count == 0 && State == "ready") return Scope == "all" ? "Working tree clean." : $"No {Scope} changes.";
            if (DiffState == "loading") return "Loading diff...";
            if (DiffState == "error") return string.IsNullOrWhiteSpace(DiffError) ? "This diff is unavailable." : DiffError;
            if (DiffBinary) return "Binary files cannot be displayed as text.";
            return "Select a changed file to inspect its diff.";
        }
    }
    public bool ScopeAll => Scope == "all";
    public bool ScopeStaged => Scope == "staged";
    public bool ScopeUnstaged => Scope == "unstaged";

    public void SetScope(string scope)
    {
        Scope = scope is "staged" or "unstaged" ? scope : "all";
        OnPropertyChanged(nameof(ScopeAll));
        OnPropertyChanged(nameof(ScopeStaged));
        OnPropertyChanged(nameof(ScopeUnstaged));
        ApplyFilter();
        if (SelectedFile is not null && FilteredFiles.Contains(SelectedFile))
            _ = LoadDiffAsync(SelectedFile, Scope);
    }

    public void SetFilter(string filter)
    {
        Filter = filter ?? string.Empty;
        ApplyFilter();
        if (SelectedFile is not null) _ = LoadDiffAsync(SelectedFile, Scope);
    }

    public async Task RefreshAsync()
    {
        if (!_host.EnsureSdk() || _host.SelectedProject is null)
        {
            Clear();
            return;
        }
        State = "loading";
        Error = string.Empty;
        try
        {
            var status = await _host.Sdk!.GitStatusAsync(_host.SelectedProject.ProjectId);
            Branch = status.Branch ?? string.Empty;
            ChangedFiles = status.ChangedFiles;
            Additions = (int)status.Additions;
            Deletions = (int)status.Deletions;
            StatusTruncated = status.Truncated;
            Files.Clear();
            foreach (var node in status.Files)
            {
                Files.Add(new GitFileItem(
                    node.Path, node.Status, node.Staged, node.Unstaged,
                    node.Conflicted, (int)node.Additions, (int)node.Deletions,
                    node.OldPath ?? string.Empty, node.Binary));
            }
            ApplyFilter();
            State = "ready";
        }
        catch (SdkException exception) when (exception.Code == "not_git_repository")
        {
            Clear();
            State = "not_repository";
            Error = exception.Message;
        }
        catch (Exception exception)
        {
            State = "error";
            Error = exception.Message;
        }
    }

    public async Task LoadDiffAsync(GitFileItem file, string scope = "all")
    {
        if (!_host.EnsureSdk() || _host.SelectedProject is null) return;
        SelectedFile = file;
        DiffLines.Clear();
        Patch = string.Empty;
        DiffError = string.Empty;
        DiffBinary = false;
        DiffTruncated = false;
        DiffAdditions = 0;
        DiffDeletions = 0;
        DiffState = "loading";
        try
        {
            var diff = await _host.Sdk!.GitDiffAsync(_host.SelectedProject.ProjectId, scope, file.Path);
            Patch = diff.Patch;
            DiffBinary = diff.Binary;
            DiffTruncated = diff.Truncated;
            DiffAdditions = diff.Additions;
            DiffDeletions = diff.Deletions;
            foreach (var hunk in diff.Hunks)
            {
                DiffLines.Add(new DiffLineItem("hunk", hunk.Header, string.Empty, string.Empty));
                foreach (var line in hunk.Lines)
                {
                    DiffLines.Add(new DiffLineItem(
                        line.Kind, line.Text,
                        line.OldLine?.ToString() ?? string.Empty,
                        line.NewLine?.ToString() ?? string.Empty));
                }
            }
            OnPropertyChanged(nameof(HasDiffLines));
            OnPropertyChanged(nameof(ShowDiffEmpty));
            DiffState = "ready";
        }
        catch (Exception exception)
        {
            DiffState = "error";
            DiffError = exception.Message;
        }
    }

    internal void Clear()
    {
        Files.Clear();
        FilteredFiles.Clear();
        DiffLines.Clear();
        SelectedFile = null;
        State = "idle";
        Error = string.Empty;
        DiffState = "idle";
        DiffError = string.Empty;
        Patch = string.Empty;
        Branch = string.Empty;
        ChangedFiles = 0;
        Additions = 0;
        Deletions = 0;
        StatusTruncated = false;
        DiffBinary = false;
        DiffTruncated = false;
        DiffAdditions = 0;
        DiffDeletions = 0;
    }

    // Session changes drop the rendered diff without resetting repository status.
    internal void ClearDiffLines() => DiffLines.Clear();

    // Applies the current scope and filter; exposed for tests that seed Files directly.
    internal void ApplyFilter()
    {
        var selectedPath = SelectedFile?.Path;
        FilteredFiles.Clear();
        foreach (var file in Files.Where(file =>
                     (Scope == "all" || Scope == "staged" && file.Staged || Scope == "unstaged" && file.Unstaged) &&
                     (Filter.Length == 0 || file.Path.Contains(Filter, StringComparison.OrdinalIgnoreCase))))
        {
            FilteredFiles.Add(file);
        }
        SelectedFile = FilteredFiles.FirstOrDefault(file => file.Path == selectedPath)
            ?? FilteredFiles.FirstOrDefault();
        OnPropertyChanged(nameof(HasFilteredFiles));
        OnPropertyChanged(nameof(FileCountText));
        OnPropertyChanged(nameof(EmptyMessage));
    }

    private void NotifyDiffPresentationChanged()
    {
        OnPropertyChanged(nameof(HasDiffLines));
        OnPropertyChanged(nameof(ShowDiffEmpty));
        OnPropertyChanged(nameof(ShowDiffStats));
        OnPropertyChanged(nameof(EmptyMessage));
    }
}
