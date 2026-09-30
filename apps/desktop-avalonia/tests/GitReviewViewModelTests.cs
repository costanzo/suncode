using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

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
