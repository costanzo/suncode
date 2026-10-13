using System.Text;
using System.Text.RegularExpressions;

namespace SunCode.Desktop.Infrastructure.Git;

internal static partial class GitOutputParser
{
    private static readonly UTF8Encoding StrictUtf8 = new(false, true);
    internal sealed record StatusSnapshot(string? Branch, IReadOnlyList<GitChange> Files, bool UnsupportedPaths);
    internal sealed record Numbers(int Additions, int Deletions, bool Binary);

    internal static StatusSnapshot Status(byte[] output, string prefix)
    {
        var records = SplitZero(output);
        var files = new List<GitChange>();
        string? branch = null;
        var unsupported = false;
        for (var index = 0; index < records.Count; index++)
        {
            // Rename old-path records are consumed even when their UTF-8 representation is unsupported.
            var kind = records[index].Length == 0 ? '\0' : (char)records[index][0];
            var oldBytes = kind == '2' && index + 1 < records.Count ? records[index + 1] : null;
            try
            {
                var record = StrictUtf8.GetString(records[index]);
                if (record.StartsWith("# branch.head ", StringComparison.Ordinal))
                {
                    var head = record[14..];
                    branch = head == "(detached)" ? null : head;
                    continue;
                }
                if (kind is not ('1' or '2' or 'u' or '?')) continue;
                var fields = kind == '?' ? [] : record.Split(' ', kind == '1' ? 9 : kind == '2' ? 10 : 11);
                var repoPath = kind == '?' ? record[2..] : fields[^1];
                if (!repoPath.StartsWith(prefix, StringComparison.Ordinal)) continue;
                var path = repoPath[prefix.Length..];
                GitService.ValidateRelative(path);
                string? oldPath = null;
                if (oldBytes is not null)
                {
                    var old = StrictUtf8.GetString(oldBytes);
                    if (old.StartsWith(prefix, StringComparison.Ordinal))
                    {
                        oldPath = old[prefix.Length..];
                        GitService.ValidateRelative(oldPath);
                    }
                }
                var xy = kind == '?' ? "??" : fields[1];
                var conflicted = kind == 'u';
                var status = conflicted ? "conflicted" : kind == '?' ? "untracked"
                    : xy.Contains('R') ? "renamed" : xy.Contains('D') ? "deleted"
                    : xy.Contains('A') ? "added" : xy.Contains('T') ? "typechanged" : "modified";
                files.Add(new(path, status, kind != '?' && xy[0] != '.', kind == '?' || xy[1] != '.', conflicted, oldPath));
            }
            catch (DecoderFallbackException) { unsupported = true; }
            catch (GitException) { unsupported = true; }
            finally { if (kind == '2') index++; }
        }
        return new(branch, files.OrderByDescending(file => file.Conflicted).ThenBy(file => file.Path, StringComparer.Ordinal).ToArray(), unsupported);
    }

    internal static Dictionary<string, Numbers> Numstat(byte[] output)
    {
        var records = SplitZero(output);
        var result = new Dictionary<string, Numbers>(StringComparer.Ordinal);
        for (var index = 0; index < records.Count; index++)
        {
            try
            {
                var record = StrictUtf8.GetString(records[index]);
                var fields = record.Split('\t', 3);
                if (fields.Length != 3) continue;
                var path = fields[2];
                if (path.Length == 0)
                {
                    // -z rename: counts TAB NUL old-path NUL new-path NUL.
                    if (index + 2 >= records.Count) break;
                    index += 2;
                    path = StrictUtf8.GetString(records[index]);
                }
                var binary = fields[0] == "-" || fields[1] == "-";
                result[path] = new(binary ? 0 : int.Parse(fields[0]), binary ? 0 : int.Parse(fields[1]), binary);
            }
            catch (DecoderFallbackException) { }
        }
        return result;
    }

    internal static GitDiffResult Patch(string patch, bool truncated = false)
    {
        var hunks = new List<GitDiffHunk>();
        List<GitDiffLine>? lines = null;
        var oldLine = 0;
        var newLine = 0;
        var oldRemaining = 0;
        var newRemaining = 0;
        var additions = 0;
        var deletions = 0;
        var lineCount = 0;
        var binary = false;
        foreach (var raw in patch.Split('\n'))
        {
            var text = raw.TrimEnd('\r');
            var match = HunkHeader().Match(text);
            if (match.Success)
            {
                if (hunks.Count >= GitService.MaxHunks) { truncated = true; break; }
                oldLine = int.Parse(match.Groups[1].Value);
                oldRemaining = match.Groups[2].Success ? int.Parse(match.Groups[2].Value) : 1;
                newLine = int.Parse(match.Groups[3].Value);
                newRemaining = match.Groups[4].Success ? int.Parse(match.Groups[4].Value) : 1;
                lines = [];
                hunks.Add(new(text, lines));
                continue;
            }
            if (text.StartsWith("diff --", StringComparison.Ordinal)) { lines = null; continue; }
            if (lines is null)
            {
                binary |= text.StartsWith("Binary files ", StringComparison.Ordinal) || text == "GIT binary patch";
                continue;
            }
            if (text.Length == 0 || oldRemaining == 0 && newRemaining == 0 && text[0] != '\\') continue;
            if (lineCount >= GitService.MaxLines) { truncated = true; break; }
            switch (text[0])
            {
                case '+': lines.Add(new("addition", null, newLine++, text[1..])); additions++; newRemaining--; break;
                case '-': lines.Add(new("deletion", oldLine++, null, text[1..])); deletions++; oldRemaining--; break;
                case ' ': lines.Add(new("context", oldLine++, newLine++, text[1..])); oldRemaining--; newRemaining--; break;
                case '\\': lines.Add(new("meta", null, null, text)); break;
                default: continue;
            }
            lineCount++;
        }
        return new(patch, binary, additions, deletions, hunks, truncated);
    }

    private static List<byte[]> SplitZero(byte[] bytes)
    {
        var records = new List<byte[]>();
        var start = 0;
        for (var index = 0; index < bytes.Length; index++)
        {
            if (bytes[index] != 0) continue;
            records.Add(bytes[start..index]); start = index + 1;
        }
        if (start < bytes.Length) records.Add(bytes[start..]);
        return records;
    }

    [GeneratedRegex(@"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@")]
    private static partial Regex HunkHeader();
}
