using SunCode.Desktop.Infrastructure.Git;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Tests;

public sealed class DiffReviewerViewModelTests
{
    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task DeepDirectoriesCompactForGitAndRetainedTurns(bool retainedTurn)
    {
        const string path = "ceps-applet-web/src/main/java/com/electricity/payment/wechat/management/controller/CheckPaymentController.java";
        var service = new ReviewerGitService { Files = [new(path, "modified", false, true, false)] };
        var snapshot = new TurnDiff("session", "turn", [new(path, "modified", 0, 0, false, [])]);
        using var model = retainedTurn
            ? new DiffReviewerViewModel(new("project", "Project", Path.GetTempPath()), snapshot, () => Task.FromResult(snapshot))
            : new DiffReviewerViewModel(service, new("project", "Project", Path.GetTempPath()));
        await model.OpenAsync("all", path);

        var directory = Assert.Single(model.Roots);
        Assert.Equal(path[..path.LastIndexOf('/')], directory.Entry.Name);
        Assert.Equal(directory.Entry.Name, directory.Entry.Path);
        Assert.Equal(directory.Entry.Path, directory.ToolTipText);
        Assert.Equal(10, directory.DirectoryPaths.Count);
        var file = Assert.Single(directory.Children);
        Assert.Equal("CheckPaymentController.java", file.Entry.Name);
        Assert.Equal("CheckPaymentController.java", file.ToolTipText);
        Assert.Equal(path, file.File!.Path);
        Assert.Equal(path, model.SelectedPath);
        Assert.Equal(1, model.FileCount);
        if (!retainedTurn) Assert.Equal(path, service.LastPath);
    }

    [Fact]
    public async Task CompactionStopsAtBranchesAndDirectoriesContainingFiles()
    {
        var service = new ReviewerGitService
        {
            Files = [new("apps/desktop/Resources/Localization/en.axaml", "modified", true, false, false),
                new("apps/desktop/Resources/Localization/zh.axaml", "modified", false, true, false),
                new("apps/cli/src/main.rs", "modified", false, true, false),
                new("apps/cli/README.md", "modified", false, true, false),
                new("README.md", "modified", false, true, false)]
        };
        using var model = new DiffReviewerViewModel(service, new("project", "Project", Path.GetTempPath()));
        await model.OpenAsync("all", null);

        Assert.Equal(2, model.Roots.Count);
        var apps = model.Roots.Single(node => node.Entry.Path == "apps");
        var cli = apps.Children.Single(node => node.Entry.Path == "apps/cli");
        Assert.Equal("cli", cli.Entry.Name);
        Assert.Equal(2, cli.Children.Count);
        Assert.Equal("src", cli.Children.Single(node => node.Entry.IsDirectory).Entry.Name);
        var resources = apps.Children.Single(node => node.Entry.Name == "desktop/Resources/Localization");
        Assert.Equal(new[] { "en.axaml", "zh.axaml" }, resources.Children.Select(node => node.Entry.Name));
        Assert.Equal(5, model.FileCount);

        await model.SetScopeAsync("staged");
        Assert.Equal("apps/desktop/Resources/Localization", Assert.Single(model.Roots).Entry.Name);
        Assert.Equal("apps/desktop/Resources/Localization/en.axaml", model.SelectedPath);
        Assert.Equal(1, model.FileCount);
    }

    [Fact]
    public async Task RefreshPreservesExpansionAndSelectionWhenCompactChainsSplitAndMerge()
    {
        const string path = "src/main/java/controller/Payment.java";
        var file = new GitChange(path, "modified", false, true, false);
        var service = new ReviewerGitService { Files = [file] };
        using var model = new DiffReviewerViewModel(service, new("project", "Project", Path.GetTempPath()));
        await model.OpenAsync("all", path);
        model.MoveChange(1);
        model.Roots[0].Entry.IsExpanded = false;
        service.Files = [file, new("src/main/resources/config.xml", "modified", false, true, false)];
        await model.RefreshAsync();
        var root = Assert.Single(model.Roots);
        Assert.Equal("src/main", root.Entry.Name);
        Assert.False(root.Entry.IsExpanded);
        var controller = root.Children.Single(node => node.Entry.Name == "java/controller");
        Assert.False(controller.Entry.IsExpanded);
        Assert.Equal(path, model.SelectedPath);
        Assert.Equal("2 / 2", model.ChangePosition);

        root.Entry.IsExpanded = true;
        service.Files = [file];
        await model.RefreshAsync();
        Assert.Equal("src/main/java/controller", model.Roots[0].Entry.Name);
        Assert.False(model.Roots[0].Entry.IsExpanded);
        model.Roots[0].Entry.IsExpanded = true;
        await model.RefreshAsync();
        Assert.True(model.Roots[0].Entry.IsExpanded);
        Assert.Equal(path, model.SelectedPath);
    }

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
        public string? LastPath;
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
            LastPath = path;
            ReadCount++;
            return Pending?.Task ?? Task.FromResult(Diff);
        }
    }
}
