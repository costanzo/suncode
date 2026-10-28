namespace SunCode.Desktop.Infrastructure.Git;

internal sealed class GitService(Func<IEnumerable<string>>? candidates = null) : IGitService
{
    internal const int MaxFiles = 2_000;
    internal const int MaxPatchBytes = 2 * 1024 * 1024;
    internal const int MaxLines = 12_000;
    internal const int MaxHunks = 2_000;
    private static GitCommandRunner? _discoveredDefault;
    private GitCommandRunner? _runner;
    private readonly SemaphoreSlim _discovery = new(1);

    public async Task<bool> CheckAvailabilityAsync(CancellationToken cancellation = default)
    {

        if (candidates is null && Volatile.Read(ref _discoveredDefault) is { } cached && File.Exists(cached.Executable))
        {
            _runner = cached;
            return true;
        }
        await _discovery.WaitAsync(cancellation);
        try
        {
            GitCommandRunner? found = null;
            foreach (var candidate in (candidates?.Invoke() ?? GitCommandRunner.Candidates()).Distinct())
            {
                cancellation.ThrowIfCancellationRequested();
                if (!Path.IsPathFullyQualified(candidate) || !File.Exists(candidate)) continue;
                var runner = new GitCommandRunner(candidate);
                try
                {
                    var version = await runner.RunAsync(Path.GetTempPath(), ["--version"], cancellation,
                        4096, TimeSpan.FromSeconds(3));
                    if (version.ExitCode == 0 && version.Text.StartsWith("git version ", StringComparison.Ordinal))
                    {
                        var parts = version.Text[12..].Trim().Split('.');
                        if (parts.Length >= 2 && int.TryParse(parts[0], out var major) && int.TryParse(parts[1], out var minor)
                            && (major > 2 || major == 2 && minor >= 11)) { found = runner; break; }
                    }
                }
                catch (GitException) { }
            }
            _runner = found;
            if (candidates is null) Volatile.Write(ref _discoveredDefault, found);
            return found is not null;
        }
        finally { _discovery.Release(); }
    }

    public async Task<GitStatusResult> StatusAsync(string projectRoot, CancellationToken cancellation = default)
    {
        var runner = RequireRunner();
        using var deadline = CancellationTokenSource.CreateLinkedTokenSource(cancellation);
        deadline.CancelAfter(TimeSpan.FromSeconds(30));
        var context = await OpenAsync(runner, projectRoot, deadline.Token);
        var snapshot = await ReadStatusAsync(runner, context, deadline.Token);
        var files = snapshot.Files.Take(MaxFiles).ToList();
        var stats = await DiffCommandAsync(runner, context, "all", [context.Prefix.Length == 0 ? "." : context.Prefix],
            true, deadline.Token);
        RequireComplete(stats);
        var numbers = GitOutputParser.Numstat(stats.Output);
        var truncated = snapshot.Files.Count > MaxFiles;
        for (var index = 0; index < files.Count; index++)
        {
            var file = files[index];
            if (file.Status == "untracked")
            {
                var diff = await UntrackedDiffAsync(runner, context, file.Path, deadline.Token);
                files[index] = file with { Additions = diff.Additions, Deletions = diff.Deletions, Binary = diff.Binary };
                truncated |= diff.Truncated;
            }
            else if (numbers.TryGetValue(context.Prefix + file.Path, out var number))
                files[index] = file with { Additions = number.Additions, Deletions = number.Deletions, Binary = number.Binary };
        }
        // A bounded list has bounded totals too; the UI marks this snapshot as limited.
        return new(snapshot.Branch, files, snapshot.Files.Count, files.Sum(file => file.Additions),
            files.Sum(file => file.Deletions), truncated || snapshot.UnsupportedPaths);
    }

    public async Task<GitDiffResult> DiffAsync(string projectRoot, string scope, string path, CancellationToken cancellation = default)
    {
        if (scope is not ("all" or "staged" or "unstaged")) throw new ArgumentException("Unknown Git diff scope.");
        ValidateRelative(path);
        var runner = RequireRunner();
        var context = await OpenAsync(runner, projectRoot, cancellation);
        var snapshot = await ReadStatusAsync(runner, context, cancellation);
        var file = snapshot.Files.FirstOrDefault(file => file.Path == path)
            ?? throw new GitException("diff_not_found", "This file no longer has changes. Refresh the repository.");
        if (scope == "staged" && !file.Staged || scope == "unstaged" && !file.Unstaged)
            throw new GitException("diff_not_found", "This file has no changes in the selected scope.");
        if (file.Status == "untracked") return await UntrackedDiffAsync(runner, context, path, cancellation);
        var paths = new List<string> { context.Prefix + path };
        if (file.OldPath is { } oldPath) paths.Add(context.Prefix + oldPath);
        var patch = await DiffCommandAsync(runner, context, scope, paths, false, cancellation);
        RequireSuccess(patch);
        var diff = GitOutputParser.Patch(patch.Text, patch.Truncated);
        // Merge conflicts have no ordinary two-way patch; show the HEAD-to-worktree content.
        if (file.Conflicted && scope == "unstaged" && diff.Hunks.Count == 0)
        {
            patch = await DiffCommandAsync(runner, context, "all", paths, false, cancellation);
            RequireSuccess(patch);
            diff = GitOutputParser.Patch(patch.Text, patch.Truncated);
        }
        return diff;
    }

    private GitCommandRunner RequireRunner() => _runner ?? throw new GitException("git_missing", "Git is not installed or cannot be executed.");
    private sealed record RepositoryContext(string Root, string Project, string Prefix, bool HasHead, IReadOnlyList<string> FilterOverrides);

    private static async Task<RepositoryContext> OpenAsync(GitCommandRunner runner, string projectRoot, CancellationToken cancellation)
    {
        var project = PhysicalDirectory(projectRoot);
        var probe = await runner.RunAsync(project, ["rev-parse", "--show-toplevel"], cancellation);
        if (probe.ExitCode != 0)
        {
            if (probe.Error.Contains("not a git repository", StringComparison.Ordinal))
                throw new GitException("not_repository", "This project is not inside a Git repository.");
            if (probe.Error.Contains("must be run in a work tree", StringComparison.Ordinal))
                throw new GitException("unsupported_repository", "Bare Git repositories are not supported.");
            RequireSuccess(probe);
        }
        RequireComplete(probe);
        // Remove the one line terminator, preserving whitespace in directory names.
        var root = PhysicalDirectory(probe.Text.TrimEnd('\r', '\n'));
        var prefix = Path.GetRelativePath(root, project).Replace(Path.DirectorySeparatorChar, '/');
        if (prefix != ".") ValidateRelative(prefix);
        var head = await runner.RunAsync(root, ["rev-parse", "--verify", "HEAD"], cancellation);
        if (head.ExitCode != 0 && head.ExitCode != 128) RequireSuccess(head);
        var filters = await runner.RunAsync(root, ["config", "--null", "--name-only", "--get-regexp",
            @"^filter\..*\.(clean|process|required)$"], cancellation);
        RequireSuccess(filters, allowDifference: true);
        if (filters.Truncated) throw new GitException("output_limit", "Git filter configuration exceeds the reading limit.");
        var overrides = new List<string>();
        // A diff may otherwise start repository-configured clean/filter processes while reading a worktree file.
        foreach (var name in filters.Text.Split('\0', StringSplitOptions.RemoveEmptyEntries))
            overrides.AddRange(["-c", name + (name.EndsWith(".required", StringComparison.OrdinalIgnoreCase) ? "=false" : "=")]);
        return new(root, project, prefix == "." ? "" : prefix + "/", head.ExitCode == 0, overrides);
    }

    private static async Task<GitOutputParser.StatusSnapshot> ReadStatusAsync(GitCommandRunner runner, RepositoryContext context, CancellationToken cancellation)
    {
        var status = await runner.RunAsync(context.Root, context.FilterOverrides.Concat(new[] {
            "status", "--porcelain=v2", "-z", "--branch", "--untracked-files=all", "--ignore-submodules=none",
             "--", context.Prefix.Length == 0 ? "." : context.Prefix }), cancellation);
        RequireComplete(status);
        return GitOutputParser.Status(status.Output, context.Prefix);
    }

    private static async Task<GitCommandResult> DiffCommandAsync(GitCommandRunner runner, RepositoryContext context,
        string scope, IEnumerable<string> paths, bool numstat, CancellationToken cancellation)
    {
        var args = context.FilterOverrides.Concat(new[] { "diff", "--no-ext-diff", "--no-textconv", "--no-color", "--find-renames",
            "--ignore-submodules=none", "--submodule=short" }).ToList();
        if (numstat) args.AddRange(["--numstat", "-z"]);
        else args.AddRange(["--patch", "--unified=3", "--src-prefix=a/", "--dst-prefix=b/"]);
        if (scope == "staged") args.Add("--cached");
        if (scope != "unstaged")
        {
            // Git's empty tree is computed locally for repositories with no initial commit, including SHA-256 repos.
            if (context.HasHead) args.Add("HEAD");
            else
            {
                var empty = await runner.RunAsync(context.Root, ["hash-object", "-t", "tree", "--stdin"], cancellation);
                RequireComplete(empty);
                args.Add(empty.Text.Trim());
            }
        }
        args.Add("--"); args.AddRange(paths);
        var result = await runner.RunAsync(context.Root, args, cancellation, numstat ? 8 * 1024 * 1024 : MaxPatchBytes);
        return result;
    }

    private static async Task<GitDiffResult> UntrackedDiffAsync(GitCommandRunner runner, RepositoryContext context,
        string path, CancellationToken cancellation)
    {
        ValidateRelative(path);
        var fullPath = Path.Combine(context.Project, path);
        // Reject symlinked parents. Git no-index uses lstat for a leaf symlink and displays its target name.
        for (var entry = Path.GetDirectoryName(fullPath)!; entry != context.Project; entry = Path.GetDirectoryName(entry)!)
            if ((File.GetAttributes(entry) & FileAttributes.ReparsePoint) != 0)
                throw new GitException("unsupported_path", "Untracked symbolic links cannot be displayed as text.");
        if ((File.GetAttributes(fullPath) & FileAttributes.ReparsePoint) == 0 && new FileInfo(fullPath).Length > MaxPatchBytes)
            return new("", false, 0, 0, [], true);
        var diff = await runner.RunAsync(context.Root, context.FilterOverrides.Concat(new[] {
            "diff", "--no-index", "--no-ext-diff", "--no-textconv", "--no-color", "--unified=3", "--",
             OperatingSystem.IsWindows() ? "NUL" : "/dev/null", context.Prefix + path }), cancellation, MaxPatchBytes);
        RequireSuccess(diff, allowDifference: true);
        return GitOutputParser.Patch(diff.Text, diff.Truncated);
    }

    internal static void ValidateRelative(string path)
    {
        if (string.IsNullOrEmpty(path) || Path.IsPathRooted(path) || path.Contains('\0')
            || path.Split('/').Any(part => part is ".." or "." or "")
            || OperatingSystem.IsWindows() && (path.Contains('\\') || path.Contains(':')))
            throw new GitException("scope_denied", "Git paths must stay inside the opened project.");
    }

    private static string PhysicalDirectory(string path)
    {
        var full = NormalizePath(path);
        if (!Directory.Exists(full)) throw new GitException("project_unavailable", "The project directory is unavailable.");
        var current = Path.GetPathRoot(full)!;
        foreach (var part in full[current.Length..].Split(Path.DirectorySeparatorChar, StringSplitOptions.RemoveEmptyEntries))
        {
            var entry = new DirectoryInfo(Path.Combine(current, part));
            current = entry.ResolveLinkTarget(returnFinalTarget: true)?.FullName ?? entry.FullName;
        }
        return Path.TrimEndingDirectorySeparator(current);
    }

    private static string NormalizePath(string path)
        => PathNormalization.FullPath(path);

    private static void RequireSuccess(GitCommandResult result, bool allowDifference = false)
    {
        if (result.ExitCode != 0 && !(allowDifference && result.ExitCode == 1))
            throw new GitException("read_failed", string.IsNullOrWhiteSpace(result.Error)
                ? "Git could not read this repository. Refresh to retry." : result.Error.Trim());
    }
    private static void RequireComplete(GitCommandResult result)
    {
        RequireSuccess(result);
        if (result.Truncated) throw new GitException("output_limit", "Git repository metadata exceeds the reading limit.");
    }
}
