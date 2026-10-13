namespace SunCode.Desktop.Infrastructure.Git;

internal sealed record GitChange(string Path, string Status, bool Staged, bool Unstaged,
    bool Conflicted, string? OldPath = null, bool Binary = false, int Additions = 0, int Deletions = 0);
internal sealed record GitStatusResult(string? Branch, IReadOnlyList<GitChange> Files,
    int ChangedFiles, int Additions, int Deletions, bool Truncated);
internal sealed record GitDiffLine(string Kind, int? OldLine, int? NewLine, string Text);
internal sealed record GitDiffHunk(string Header, IReadOnlyList<GitDiffLine> Lines);
internal sealed record GitDiffResult(string Patch, bool Binary, int Additions, int Deletions,
    IReadOnlyList<GitDiffHunk> Hunks, bool Truncated);

internal sealed class GitException(string code, string message) : Exception(message)
{
    public string Code { get; } = code;
}

internal interface IGitService
{
    Task<bool> CheckAvailabilityAsync(CancellationToken cancellation = default);
    Task<GitStatusResult> StatusAsync(string projectRoot, CancellationToken cancellation = default);
    Task<GitDiffResult> DiffAsync(string projectRoot, string scope, string path, CancellationToken cancellation = default);
}
