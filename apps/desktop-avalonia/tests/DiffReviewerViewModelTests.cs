using SunCode.Desktop.Infrastructure.Git;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Tests;

public sealed class DiffReviewerViewModelTests
{
    [Fact]
    public async Task RangeTreeRefreshAndLifetimeAreIndependentFromDrawerSelection()
    {
        var service = new ReviewerGitService();
        using var model = new DiffReviewerViewModel(service, new("project", "Project", Path.GetTempPath()));
        await model.OpenAsync("all", "src/b.txt");
        Assert.Equal("src/b.txt", model.SelectedPath);
        Assert.Equal(2, model.FileCount);
        Assert.True(service.IncludeUnchanged);
        model.MoveChange(1);
        Assert.Equal("2 / 2", model.ChangePosition);
        model.Roots[0].Entry.IsExpanded = false;
        await model.RefreshAsync();
        Assert.Equal("2 / 2", model.ChangePosition);
        Assert.False(model.Roots[0].Entry.IsExpanded);
        await model.SetScopeAsync("staged");
        Assert.Equal(1, model.FileCount);
        Assert.Equal("src/a.txt", model.SelectedPath);
        Assert.Equal("1 / 2", model.ChangePosition);
        Assert.False(model.CanPrevious);
        var closed = 0;
        model.CloseRequested += () => closed++;
        model.Dispose(); model.Dispose();
        Assert.Equal(1, closed);
    }

    [Fact]
    public async Task SupersededReadsCannotReplaceNewerSelectionOrPublishAfterDisposal()
    {
        var service = new ReviewerGitService();
        using var model = new DiffReviewerViewModel(service, new("project", "Project", Path.GetTempPath()));
        await model.OpenAsync("all", null);
        var delayed = new TaskCompletionSource<GitDiffResult>();
        service.Pending = delayed;
        var older = model.SelectAsync(model.Roots[0].Children[0]);
        service.Pending = null;
        await model.SelectAsync(model.Roots[0].Children[1]);
        var document = model.Document;
        delayed.SetResult(service.Diff);
        await older;
        Assert.Same(document, model.Document);
        Assert.Equal("src/b.txt", model.SelectedPath);
        delayed = new(); service.Pending = delayed;
        older = model.RefreshAsync();
        model.Dispose(); delayed.SetResult(service.Diff);
        await older;
        Assert.Same(document, model.Document);
    }

    internal sealed class ReviewerGitService : IGitService
    {
        public bool IncludeUnchanged;
        public TaskCompletionSource<GitDiffResult>? Pending;
        public GitDiffResult Diff = DiffReviewDocumentTests.Patch([
            new("deletion", 1, null, "old"), new("addition", null, 1, "new"), new("context", 2, 2, "same"), new("addition", null, 3, "extra")
        ]);
        public Task<bool> CheckAvailabilityAsync(CancellationToken cancellation = default) => Task.FromResult(true);
        public Task<GitStatusResult> StatusAsync(string projectRoot, CancellationToken cancellation = default) => Task.FromResult(new GitStatusResult("main",
            [new("src/a.txt", "modified", true, false, false), new("src/b.txt", "modified", false, true, false)], 2, 4, 2, false));
        public Task<GitDiffResult> DiffAsync(string projectRoot, string scope, string path, CancellationToken cancellation = default, bool includeUnchanged = false)
        {
            IncludeUnchanged = includeUnchanged;
            return Pending?.Task ?? Task.FromResult(Diff);
        }
    }
}
