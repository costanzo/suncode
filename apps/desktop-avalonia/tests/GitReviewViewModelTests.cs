using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Infrastructure.Git;

namespace SunCode.Desktop.Tests;

public sealed class GitReviewViewModelTests
{
    [Fact]
    public void ScopeAndFilterNarrowFilesAndKeepSelectionWhenPossible()
    {
        var viewModel = Seeded();

        viewModel.ApplyFilter();
        Assert.Equal(3, viewModel.FilteredFiles.Count);
        Assert.Equal("src/a.cs", viewModel.SelectedFile?.Path);

        viewModel.SelectedFile = viewModel.FilteredFiles[1];
        viewModel.SetScope("unstaged");
        Assert.Equal(["src/b.cs", "docs/readme.md"], viewModel.FilteredFiles.Select(file => file.Path));
        Assert.Equal("src/b.cs", viewModel.SelectedFile?.Path);
        Assert.True(viewModel.ScopeUnstaged);
        Assert.False(viewModel.ScopeAll);

        viewModel.SetFilter("README");
        Assert.Equal("docs/readme.md", Assert.Single(viewModel.FilteredFiles).Path);
        Assert.Equal("docs/readme.md", viewModel.SelectedFile?.Path);
        Assert.Equal("1 file", viewModel.FileCountText);
    }

    [Fact]
    public void EmptyMessageReflectsFilterAndScope()
    {
        var viewModel = Seeded();

        viewModel.SetFilter("missing");
        Assert.False(viewModel.HasFilteredFiles);
        Assert.Null(viewModel.SelectedFile);
        Assert.Equal("No changed files match this filter.", viewModel.EmptyMessage);
    }

    [Fact]
    public async Task RefreshWithoutSdkClearsStatus()
    {
        var viewModel = Seeded();
        viewModel.ApplyFilter();

        await viewModel.RefreshAsync();

        Assert.Empty(viewModel.Files);
        Assert.Empty(viewModel.FilteredFiles);
        Assert.Null(viewModel.SelectedFile);
        Assert.Equal("idle", viewModel.State);
        Assert.False(viewModel.IsReady);
        Assert.Equal("No file selected", viewModel.SelectedPath);
        Assert.Equal("Clean", viewModel.ChangeSummary);
    }

    [Fact]
    public void WindowViewModelExposesGitChild()
    {
        using var window = new DesktopViewModel();

        Assert.Same(window.Git, window.Git);
        Assert.Equal("idle", window.Git.State);
        Assert.Empty(window.Git.DiffLines);
    }

    [Fact]
    public async Task GitDoesNotRequireAnAgentSdkConnection()
    {
        var service = new FakeGitService();
        var host = new FakeViewModelHost { SelectedProject = new("project", "Project", Path.GetTempPath()) };
        using var model = new GitReviewViewModel(host, service);
        await model.RefreshAsync();
        Assert.True(model.IsAvailable); Assert.Equal("ready", model.State);
        Assert.Equal("a.txt", Assert.Single(model.Files).Path);
        Assert.Equal("patch", model.Patch);
        Assert.True(model.CanCopyPatch);
    }

    [Fact]
    public async Task MissingGitClearsDataAndSkipsRepositoryQueries()
    {
        var service = new FakeGitService { Available = false };
        var host = new FakeViewModelHost { SelectedProject = new("project", "Project", Path.GetTempPath()) };
        using var model = new GitReviewViewModel(host, service);
        await model.RefreshAsync();
        Assert.False(model.IsAvailable); Assert.Equal(0, service.StatusCalls);
        service.Available = true; await model.CheckAvailabilityAsync(); await model.RefreshAsync();
        Assert.True(model.IsAvailable); Assert.NotEmpty(model.Files);
        service.Available = false; await model.CheckAvailabilityAsync();
        Assert.False(model.IsAvailable); Assert.Empty(model.Files); Assert.Empty(model.Patch);
        Assert.False(model.CanCopyPatch);
    }

    [Fact]
    public async Task RepositoryFailureKeepsGitAvailable()
    {
        var service = new FakeGitService { ReadError = new GitException("not_repository", "Not a repository") };
        var host = new FakeViewModelHost { SelectedProject = new("project", "Project", Path.GetTempPath()) };
        using var model = new GitReviewViewModel(host, service);
        await model.RefreshAsync();
        Assert.True(model.IsAvailable); Assert.Equal("not_repository", model.State);
        service.ReadError = new GitException("read_failed", "Repository error"); await model.RefreshAsync();
        Assert.True(model.IsAvailable); Assert.Equal("error", model.State);
        service.ReadError = null; await model.RefreshAsync(); Assert.Equal("ready", model.State);
    }

    [Fact]
    public async Task SavedGitDrawerIsIgnoredWithoutGitAndClosesIfExecutableDisappears()
    {
        var directory = Path.Combine(Path.GetTempPath(), "suncode-git-ui-" + Guid.NewGuid());
        try
        {
            using var store = new UiStateStore(Path.Combine(directory, "ui-state.json"));
            store.UpdateProject("project", saved => saved.BottomDrawer = "git");
            var service = new FakeGitService { Available = false };
            using var window = new DesktopViewModel(store, gitService: service);
            window.SetSelectedProjectForTests(new("project", "Project", directory));
            await window.Git.CheckAvailabilityAsync(); window.RestoreProjectPresentationState();
            Assert.False(window.Layout.GitVisible);
            service.Available = true; await window.Git.CheckAvailabilityAsync();
            store.UpdateProject("project", saved => saved.BottomDrawer = "git");
            window.RestoreProjectPresentationState(); Assert.True(window.Layout.GitVisible);
            service.Available = false; await window.Git.CheckAvailabilityAsync(); Assert.False(window.Layout.GitVisible);
        }
        finally { if (Directory.Exists(directory)) Directory.Delete(directory, true); }
    }

    [Fact]
    public async Task LateRefreshCannotReplaceNewerStatus()
    {
        var service = new FakeGitService();
        var firstStarted = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
        var firstResult = new TaskCompletionSource<GitStatusResult>(TaskCreationOptions.RunContinuationsAsynchronously);
        service.ReadStatus = call =>
        {
            if (call == 1) { firstStarted.SetResult(); return firstResult.Task; }
            return Task.FromResult(Status("new.txt"));
        };
        var host = new FakeViewModelHost { SelectedProject = new("project", "Project", Path.GetTempPath()) };
        using var model = new GitReviewViewModel(host, service);
        var first = model.RefreshAsync(); await firstStarted.Task.WaitAsync(TimeSpan.FromSeconds(5));
        await model.RefreshAsync(); firstResult.SetResult(Status("old.txt")); await first;
        Assert.Equal("new.txt", Assert.Single(model.Files).Path);
    }

    [Fact]
    public async Task LateDiffAndClosedWindowCannotRestoreClearedPatch()
    {
        var service = new FakeGitService();
        var delayed = new TaskCompletionSource<GitDiffResult>(TaskCreationOptions.RunContinuationsAsynchronously);
        var host = new FakeViewModelHost { SelectedProject = new("project", "Project", Path.GetTempPath()) };
        using var model = new GitReviewViewModel(host, service);
        await model.CheckAvailabilityAsync();
        service.ReadDiff = path => path == "a.txt" ? delayed.Task : Task.FromResult(Diff("new patch"));
        var first = model.LoadDiffAsync(File("a.txt", false, true));
        await model.LoadDiffAsync(File("b.txt", false, true));
        delayed.SetResult(Diff("old patch")); await first;
        Assert.Equal("new patch", model.Patch);
        delayed = new(TaskCreationOptions.RunContinuationsAsynchronously);
        first = model.LoadDiffAsync(File("a.txt", false, true));
        model.Dispose(); delayed.SetResult(Diff("closed patch")); await first;
        Assert.Empty(model.Patch); Assert.Empty(model.DiffLines);
    }

    private static GitStatusResult Status(string path) => new("main", [new(path, "untracked", false, true, false)], 1, 1, 0, false);
    private static GitDiffResult Diff(string patch) => new(patch, false, 1, 0, [], false);
    private sealed class FakeGitService : IGitService
    {
        public bool Available { get; set; } = true;
        public int StatusCalls { get; private set; }
        public Exception? ReadError { get; set; }
        public Func<int, Task<GitStatusResult>>? ReadStatus { get; set; }
        public Func<string, Task<GitDiffResult>>? ReadDiff { get; set; }
        public Task<bool> CheckAvailabilityAsync(CancellationToken cancellation = default) => Task.FromResult(Available);
        public Task<GitStatusResult> StatusAsync(string projectRoot, CancellationToken cancellation = default)
        {
            StatusCalls++;
            return ReadError is not null ? Task.FromException<GitStatusResult>(ReadError)
                : ReadStatus?.Invoke(StatusCalls) ?? Task.FromResult(Status("a.txt"));
        }
        public Task<GitDiffResult> DiffAsync(string projectRoot, string scope, string path, CancellationToken cancellation = default) =>
            ReadDiff?.Invoke(path) ?? Task.FromResult(Diff("patch"));
    }

    private static GitReviewViewModel Seeded()
    {
        var viewModel = new GitReviewViewModel(new FakeViewModelHost());
        viewModel.Files.Add(File("src/a.cs", staged: true, unstaged: false));
        viewModel.Files.Add(File("src/b.cs", staged: true, unstaged: true));
        viewModel.Files.Add(File("docs/readme.md", staged: false, unstaged: true));
        return viewModel;
    }

    private static GitFileItem File(string path, bool staged, bool unstaged) =>
        new(path, "modified", staged, unstaged, false, 1, 0, string.Empty, false);
}
