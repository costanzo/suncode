using System.Text;
using SunCode.Desktop.Infrastructure.Git;

namespace SunCode.Desktop.Tests;

public sealed class GitServiceTests
{
    [Fact]
    public async Task MissingGitIsOptionalAndCanBeDetectedLater()
    {
        string[] candidates = [];
        var service = new GitService(() => candidates);
        Assert.False(await service.CheckAvailabilityAsync());
        Assert.Equal("git_missing", (await Assert.ThrowsAsync<GitException>(() => service.StatusAsync(Path.GetTempPath()))).Code);
        candidates = GitCommandRunner.Candidates().ToArray();
        Assert.True(await service.CheckAvailabilityAsync());
    }

    [Fact]
    public async Task NonRepositoryAndBareRepositoryAreDistinctFromMissingGit()
    {
        using var repo = await Repository.CreateAsync();
        var outside = Path.Combine(repo.Root, "outside");
        Directory.CreateDirectory(outside);
        // A separate directory outside the repository is required (discovery walks ancestors).
        var plain = Path.Combine(Path.GetTempPath(), "suncode-plain-" + Guid.NewGuid());
        Directory.CreateDirectory(plain);
        try
        {
            Assert.Equal("not_repository", (await Assert.ThrowsAsync<GitException>(() => repo.Service.StatusAsync(plain))).Code);
            await repo.Run("init", "--bare", outside);
            Assert.Equal("unsupported_repository", (await Assert.ThrowsAsync<GitException>(() => repo.Service.StatusAsync(outside))).Code);
        }
        finally { Directory.Delete(plain, true); }
    }

    [Fact]
    public async Task ThreeScopesUseNetChangesInsteadOfAddingStagedAndUnstagedCounts()
    {
        using var repo = await Repository.CreateAsync();
        repo.Write("file.txt", "base\n"); await repo.Commit();
        repo.Write("file.txt", "staged\n"); await repo.Run("add", "file.txt");
        repo.Write("file.txt", "base\nextra\n");
        var status = await repo.Service.StatusAsync(repo.Root);
        var file = Assert.Single(status.Files);
        Assert.True(file.Staged && file.Unstaged);
        Assert.Equal(1, status.Additions); Assert.Equal(0, status.Deletions);
        var all = await repo.Service.DiffAsync(repo.Root, "all", "file.txt");
        var staged = await repo.Service.DiffAsync(repo.Root, "staged", "file.txt");
        var unstaged = await repo.Service.DiffAsync(repo.Root, "unstaged", "file.txt");
        Assert.Equal((1, 0), (all.Additions, all.Deletions));
        Assert.Equal((1, 1), (staged.Additions, staged.Deletions));
        Assert.Equal((2, 1), (unstaged.Additions, unstaged.Deletions));
        Assert.Equal(2, all.Hunks[0].Lines.Last().NewLine);
    }

    [Fact]
    public async Task UnbornRepositoryIncludesIndexAndCurrentWorkingContent()
    {
        using var repo = await Repository.CreateAsync();
        repo.Write("staged.txt", "index\n"); await repo.Run("add", "staged.txt");
        repo.Write("staged.txt", "working\nextra\n"); repo.Write("new.txt", "new\n");
        var status = await repo.Service.StatusAsync(repo.Root);
        Assert.Equal("main", status.Branch); Assert.Equal(2, status.ChangedFiles);
        Assert.Equal(3, status.Additions);
        Assert.Equal(2, (await repo.Service.DiffAsync(repo.Root, "all", "staged.txt")).Additions);
        Assert.Equal(1, (await repo.Service.DiffAsync(repo.Root, "staged", "staged.txt")).Additions);
        Assert.Equal(1, (await repo.Service.DiffAsync(repo.Root, "unstaged", "new.txt")).Additions);
        await Assert.ThrowsAsync<GitException>(() => repo.Service.DiffAsync(repo.Root, "staged", "new.txt"));
    }

    [Fact]
    public async Task LiteralPathsAndRenameRecordsPreserveUnicodeSpacesTabsAndNewlines()
    {
        using var repo = await Repository.CreateAsync();
        var old = OperatingSystem.IsWindows() ? "旧 file.txt" : "旧 file\tline\n.txt";
        const string renamed = ":(glob)* new.txt";
        // ':' is not a valid Windows file name; the wildcard and leading dash still exercise argv handling there.
        var next = OperatingSystem.IsWindows() ? "-new file.txt" : renamed;
        repo.Write(old, "one\ntwo\n"); await repo.Commit();
        await repo.Run("mv", "--", old, next);
        var status = await repo.Service.StatusAsync(repo.Root);
        var file = Assert.Single(status.Files);
        Assert.Equal(next, file.Path); Assert.Equal(old, file.OldPath); Assert.Equal("renamed", file.Status);
        var diff = await repo.Service.DiffAsync(repo.Root, "staged", next);
        Assert.Contains("rename from", diff.Patch);
        repo.Write(next, "one\ntwo\nthree\n");
        Assert.Equal(1, (await repo.Service.DiffAsync(repo.Root, "all", next)).Additions);
    }

    [Fact]
    public async Task SubdirectoryQueriesStayInsideOpenedProject()
    {
        using var repo = await Repository.CreateAsync();
        repo.Write("inside/a.txt", "old\n"); repo.Write("outside.txt", "old\n"); await repo.Commit();
        repo.Write("inside/a.txt", "new\n"); repo.Write("outside.txt", "new\n"); repo.Write("inside/new.txt", "new\n");
        var root = Path.Combine(repo.Root, "inside");
        var status = await repo.Service.StatusAsync(root);
        Assert.Equal(["a.txt", "new.txt"], status.Files.Select(file => file.Path));
        Assert.Contains("+new", (await repo.Service.DiffAsync(root, "all", "a.txt")).Patch);
        await Assert.ThrowsAsync<GitException>(() => repo.Service.DiffAsync(root, "all", "../outside.txt"));
        await Assert.ThrowsAsync<GitException>(() => repo.Service.DiffAsync(root, "all", Path.Combine(repo.Root, "outside.txt")));
    }

    [Fact]
    public async Task ExtendedWindowsPathsAreNormalizedBeforeRepositoryScoping()
    {
        if (!OperatingSystem.IsWindows()) return;
        using var repo = await Repository.CreateAsync();
        repo.Write("file.txt", "base\n"); await repo.Commit();
        repo.Write("file.txt", "changed\n");

        var extendedRoot = @"\\?\" + repo.Root;
        var status = await repo.Service.StatusAsync(extendedRoot);

        Assert.Equal("file.txt", Assert.Single(status.Files).Path);
    }

    [Fact]
    public async Task LinkedWorktreeAndDetachedHeadAreSupported()
    {
        using var repo = await Repository.CreateAsync();
        repo.Write("a.txt", "base\n"); await repo.Commit();
        var linked = Path.Combine(repo.Root, "linked");
        await repo.Run("worktree", "add", "--detach", linked);
        File.WriteAllText(Path.Combine(linked, "a.txt"), "changed\n");
        var status = await repo.Service.StatusAsync(linked);
        Assert.Null(status.Branch); Assert.Equal("a.txt", Assert.Single(status.Files).Path);
        Assert.Equal(1, (await repo.Service.DiffAsync(linked, "all", "a.txt")).Additions);
    }

    [Fact]
    public async Task ConflictsAndBinaryFilesRemainExplicit()
    {
        using var repo = await Repository.CreateAsync();
        repo.Write("a.txt", "base\n"); await repo.Commit();
        await repo.Run("checkout", "-b", "other"); repo.Write("a.txt", "other\n"); await repo.Commit();
        await repo.Run("checkout", "main"); repo.Write("a.txt", "main\n"); await repo.Commit();
        Assert.Equal(1, (await repo.RunAllowFailure("merge", "other")).ExitCode);
        File.WriteAllBytes(Path.Combine(repo.Root, "binary.bin"), [0, 1, 2, 0]);
        var status = await repo.Service.StatusAsync(repo.Root);
        Assert.True(status.Files[0].Conflicted);
        Assert.True(status.Files.Single(file => file.Path == "binary.bin").Binary);
        Assert.True((await repo.Service.DiffAsync(repo.Root, "all", "binary.bin")).Binary);
        Assert.Contains("<<<<<<<", (await repo.Service.DiffAsync(repo.Root, "unstaged", "a.txt")).Patch);
    }

    [Fact]
    public async Task DeletedFilesModeChangesAndMissingFinalNewlineAreReadable()
    {
        using var repo = await Repository.CreateAsync();
        repo.Write("gone.txt", "gone\n"); repo.Write("a.txt", "old"); await repo.Commit();
        File.Delete(Path.Combine(repo.Root, "gone.txt")); repo.Write("a.txt", "new");
        var deleted = await repo.Service.DiffAsync(repo.Root, "all", "gone.txt");
        Assert.Equal(1, deleted.Deletions);
        var changed = await repo.Service.DiffAsync(repo.Root, "all", "a.txt");
        Assert.Equal((1, 1), (changed.Additions, changed.Deletions));
        Assert.Contains(changed.Hunks.SelectMany(hunk => hunk.Lines), line => line.Kind == "meta");
        if (!OperatingSystem.IsWindows())
        {
            await repo.Run("checkout", "--", "a.txt");
            File.SetUnixFileMode(Path.Combine(repo.Root, "a.txt"), UnixFileMode.UserRead | UnixFileMode.UserWrite | UnixFileMode.UserExecute);
            Assert.Contains("mode", (await repo.Service.DiffAsync(repo.Root, "all", "a.txt")).Patch);
        }
    }

    [Fact]
    public async Task QueriesDoNotRefreshIndexOrInvokeExternalDiffTextconvOrFsmonitor()
    {
        using var repo = await Repository.CreateAsync();
        repo.Write("a.txt", "old\n"); repo.Write(".gitattributes", "*.txt diff=custom filter=custom\n"); await repo.Commit();
        await repo.Run("config", "diff.external", "suncode-must-not-execute");
        await repo.Run("config", "diff.custom.textconv", "suncode-must-not-execute");
        await repo.Run("config", "filter.custom.clean", "suncode-must-not-execute");
        await repo.Run("config", "filter.custom.process", "suncode-must-not-execute");
        await repo.Run("config", "filter.custom.required", "true");
        await repo.Run("config", "core.fsmonitor", "suncode-must-not-execute");
        var indexPath = Path.Combine(repo.Root, ".git", "index");
        var before = File.ReadAllBytes(indexPath); var time = File.GetLastWriteTimeUtc(indexPath);
        repo.Write("a.txt", "new\n");
        Assert.Equal(1, (await repo.Service.StatusAsync(repo.Root)).ChangedFiles);
        Assert.Contains("+new", (await repo.Service.DiffAsync(repo.Root, "all", "a.txt")).Patch);
        Assert.Equal(before, File.ReadAllBytes(indexPath)); Assert.Equal(time, File.GetLastWriteTimeUtc(indexPath));
    }

    [Fact]
    public async Task LargeFilesAndManyLinesAreMarkedLimited()
    {
        using var repo = await Repository.CreateAsync();
        repo.Write("large.txt", new string('a', GitService.MaxPatchBytes + 1));
        Assert.True((await repo.Service.StatusAsync(repo.Root)).Truncated);
        Assert.True((await repo.Service.DiffAsync(repo.Root, "all", "large.txt")).Truncated);
        File.Delete(Path.Combine(repo.Root, "large.txt"));
        repo.Write("lines.txt", string.Concat(Enumerable.Repeat("line\n", GitService.MaxLines + 5)));
        var diff = await repo.Service.DiffAsync(repo.Root, "all", "lines.txt");
        Assert.True(diff.Truncated); Assert.Equal(GitService.MaxLines, diff.Hunks.Sum(hunk => hunk.Lines.Count));
    }

    [Fact]
    public async Task FileCountLimitIsExplicitAndKeepsTotalsConsistentWithVisibleFiles()
    {
        using var repo = await Repository.CreateAsync();
        for (var index = 0; index <= GitService.MaxFiles; index++) repo.Write($"file-{index:D4}.txt", "line\n");
        await repo.Commit();
        foreach (var path in Directory.EnumerateFiles(repo.Root, "*.txt")) File.Delete(path);
        var status = await repo.Service.StatusAsync(repo.Root);
        Assert.True(status.Truncated); Assert.Equal(GitService.MaxFiles + 1, status.ChangedFiles);
        Assert.Equal(GitService.MaxFiles, status.Files.Count); Assert.Equal(GitService.MaxFiles, status.Deletions);
    }

    [Fact]
    public async Task LeafSymlinksShowTheLinkWithoutReadingItsTarget()
    {
        if (OperatingSystem.IsWindows()) return;
        using var repo = await Repository.CreateAsync();
        var external = Path.Combine(Path.GetTempPath(), "suncode-link-target-" + Guid.NewGuid());
        try
        {
            File.WriteAllText(external, "outside contents must not appear\n");
            File.CreateSymbolicLink(Path.Combine(repo.Root, "link.txt"), external);
            var status = await repo.Service.StatusAsync(repo.Root);
            Assert.Equal("link.txt", Assert.Single(status.Files).Path);
            var diff = await repo.Service.DiffAsync(repo.Root, "all", "link.txt");
            Assert.Contains("120000", diff.Patch); Assert.DoesNotContain("outside contents", diff.Patch);
        }
        finally { File.Delete(external); }
    }

    [Fact]
    public async Task RemovedProjectDoesNotDisableInstalledGit()
    {
        using var repo = await Repository.CreateAsync();
        var exception = await Assert.ThrowsAsync<GitException>(() => repo.Service.StatusAsync(Path.Combine(repo.Root, "missing")));
        Assert.Equal("project_unavailable", exception.Code); Assert.True(await repo.Service.CheckAvailabilityAsync());
    }

    [Fact]
    public async Task OutputAndCancellationAreBounded()
    {
        using var repo = await Repository.CreateAsync();
        repo.Write("a.txt", "text\n");
        var result = await repo.Runner.RunAsync(repo.Root, ["status", "--porcelain=v2", "-z"], outputLimit: 4);
        Assert.True(result.Truncated); Assert.Equal(4, result.Output.Length);
        using var canceled = new CancellationTokenSource(); canceled.Cancel();
        await Assert.ThrowsAnyAsync<OperationCanceledException>(() => repo.Service.StatusAsync(repo.Root, canceled.Token));
        if (!OperatingSystem.IsWindows())
            Assert.Equal("timeout", (await Assert.ThrowsAsync<GitException>(() => repo.Runner.RunAsync(repo.Root,
                ["-c", "alias.delay=!sleep 5", "delay"], timeout: TimeSpan.FromMilliseconds(100)))).Code);
    }

    [Fact]
    public void InvalidUtf8StatusDoesNotMisalignRenameRecords()
    {
        var output = Encoding.UTF8.GetBytes("# branch.head main\0" +
            "2 R. N... 100644 100644 100644 abc abc R100 next.txt\0old").Concat(new byte[] { 255, 0 })
            .Concat(Encoding.UTF8.GetBytes("? valid.txt\0")).ToArray();
        var status = GitOutputParser.Status(output, "");
        Assert.True(status.UnsupportedPaths); Assert.Equal("valid.txt", Assert.Single(status.Files).Path);
    }

    private sealed class Repository : IDisposable
    {
        public string Root { get; } = Path.Combine(Path.GetTempPath(), "suncode-git-" + Guid.NewGuid());
        public GitService Service { get; } = new();
        public GitCommandRunner Runner { get; private set; } = null!;
        public static async Task<Repository> CreateAsync()
        {
            var repo = new Repository(); Directory.CreateDirectory(repo.Root);
            Assert.True(await repo.Service.CheckAvailabilityAsync(), "These integration tests require installed Git.");
            foreach (var path in GitCommandRunner.Candidates().Where(File.Exists))
            {
                var runner = new GitCommandRunner(path);
                var version = await runner.RunAsync(repo.Root, ["--version"]);
                if (version.ExitCode != 0) continue;
                repo.Runner = runner; break;
            }
            await repo.Run("init", "-b", "main");
            await repo.Run("config", "user.name", "Test"); await repo.Run("config", "user.email", "test@example.invalid");
            await repo.Run("config", "commit.gpgsign", "false");
            return repo;
        }
        public void Write(string path, string text)
        {
            var full = Path.Combine(Root, path); Directory.CreateDirectory(Path.GetDirectoryName(full)!); File.WriteAllText(full, text);
        }
        public async Task Commit() { await Run("add", "--all"); await Run("commit", "-m", "test"); }
        public async Task Run(params string[] args)
        {
            var result = await RunAllowFailure(args); Assert.True(result.ExitCode == 0, result.Error);
        }
        public Task<GitCommandResult> RunAllowFailure(params string[] args) => Runner.RunAsync(Root, args);
        public void Dispose()
        {
            if (OperatingSystem.IsWindows())
                foreach (var file in Directory.EnumerateFiles(Root, "*", SearchOption.AllDirectories)) File.SetAttributes(file, FileAttributes.Normal);
            Directory.Delete(Root, true);
        }
    }
}
