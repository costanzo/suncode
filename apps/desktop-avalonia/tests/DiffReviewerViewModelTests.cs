using SunCode.Desktop.Infrastructure.Git;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Tests;

public sealed class DiffReviewerViewModelTests
{
    [Theory]
    [InlineData("assets/data.bin", true)]
    [InlineData("reports/Budget.XLSX", false)]
    [InlineData("reports/summary.pdf", false)]
    [InlineData("reports/notes.docx", false)]
    public async Task UnsupportedFilesSkipDiffReadsAndDocumentConstruction(string path, bool binary)
    {
        var service = new ReviewerGitService { Files = [new(path, "modified", false, true, false, Binary: binary)] };
        using var model = new DiffReviewerViewModel(service, new("project", "Project", Path.GetTempPath()));
        await model.OpenAsync("all", path);
        Assert.Equal(0, service.ReadCount);
        Assert.Null(model.Document);
        Assert.True(model.ShowMessage);
        Assert.False(model.ShowStats);
        Assert.False(model.CanPrevious || model.CanNext);
        Assert.Contains("Cannot compare", model.Message);
    }

    [Fact]
    public async Task BinaryDiffResponseDoesNotBuildTextPresentation()
    {
        var service = new ReviewerGitService();
        service.Diff = service.Diff with { Binary = true };
        using var model = new DiffReviewerViewModel(service, new("project", "Project", Path.GetTempPath()));
        await model.OpenAsync("all", null);
        Assert.Equal(1, service.ReadCount);
        Assert.Null(model.Document);
        Assert.False(model.HasDocument);
        Assert.Contains("Cannot compare", model.Message);
    }

    [Fact]
    public async Task TurnReviewerUsesRetainedHunksAndRefreshesTheSameSource()
    {
        var file = new TurnFileDiff("src/history.cs", "modified", 1, 1, false,
            [new(100, 1, 200, 1, [new("deletion", "retained before", 100, null), new("addition", "retained after", null, 200)])]);
        var snapshot = new TurnDiff("session", "turn", [file, new("report.pdf", "modified", 0, 0, false, file.Hunks)]);
        var reads = 0;
        using var model = new DiffReviewerViewModel(new("project", "Project", Path.GetTempPath()), snapshot,
            () => { reads++; return Task.FromResult(snapshot); });
        await model.OpenAsync("all", file.Path);
        Assert.Equal(0, reads);
        Assert.False(model.IsGitComparison);
        Assert.Equal("Before turn", model.BeforeTitle);
        Assert.Equal("After turn", model.AfterTitle);
        Assert.Equal("retained before", model.Document!.DisplayRows[0].Line(true)!.Text);
        Assert.Equal(100, model.Document.DisplayRows[0].Line(true)!.Number);
        Assert.Equal(200, model.Document.DisplayRows[0].Line(false)!.Number);
        await model.RefreshAsync("report.pdf");
        Assert.Equal(1, reads);
        Assert.Null(model.Document);
        Assert.Contains("Cannot compare", model.Message);
        await model.SetScopeAsync("staged");
        Assert.Equal("all", model.Scope);
        Assert.Equal(1, reads);
    }

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
        public int ReadCount;
        public IReadOnlyList<GitChange> Files = [new("src/a.txt", "modified", true, false, false), new("src/b.txt", "modified", false, true, false)];
        public TaskCompletionSource<GitDiffResult>? Pending;
        public GitDiffResult Diff = DiffReviewDocumentTests.Patch([
            new("deletion", 1, null, "old"), new("addition", null, 1, "new"), new("context", 2, 2, "same"), new("addition", null, 3, "extra")
        ]);
        public Task<bool> CheckAvailabilityAsync(CancellationToken cancellation = default) => Task.FromResult(true);
        public Task<GitStatusResult> StatusAsync(string projectRoot, CancellationToken cancellation = default) => Task.FromResult(new GitStatusResult("main",
            Files, Files.Count, 4, 2, false));
        public Task<GitDiffResult> DiffAsync(string projectRoot, string scope, string path, CancellationToken cancellation = default, bool includeUnchanged = false)
        {
            IncludeUnchanged = includeUnchanged;
            ReadCount++;
            return Pending?.Task ?? Task.FromResult(Diff);
        }
    }
}
