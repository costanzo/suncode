using System.Text;
using DiffPlex;
using SunCode.Desktop.Infrastructure.Git;

namespace SunCode.Desktop.Models;

internal sealed record DiffTextLine(int Number, string Text, bool HasNewline = true);
internal sealed record DiffWordRange(int Start, int Length);
internal sealed record DiffReviewRow(DiffTextLine? Before, DiffTextLine? After,
    IReadOnlyList<DiffWordRange> BeforeWords, IReadOnlyList<DiffWordRange> AfterWords);

internal sealed class DiffReviewBlock(bool changed, IReadOnlyList<DiffReviewRow> rows, int changeIndex)
{
    public bool Changed { get; } = changed;
    public IReadOnlyList<DiffReviewRow> Rows { get; } = rows;
    public int ChangeIndex { get; } = changeIndex;
    public bool CanFold => !Changed && Rows.Count > 3;
    public bool Expanded { get; set; }
}

internal sealed record DiffDisplayRow(DiffReviewBlock Block, DiffReviewRow? Row)
{
    public bool IsFold => Row is null;
    public DiffTextLine? Line(bool before) => before ? Row?.Before : Row?.After;
    public string Text(bool before) => IsFold ? "\uFFFC" : Line(before)?.Text ?? string.Empty;
}

// The comparison source owns the edit script. This projection adds alignment and folding only.
internal sealed class DiffReviewDocument
{
    internal static bool CanCompareFile(string path, bool binary) => !binary && Path.GetExtension(path).ToLowerInvariant() is not
        (".pdf" or ".xls" or ".xlsx" or ".xlsm" or ".xlsb" or ".doc" or ".docx" or ".docm" or
        ".ppt" or ".pptx" or ".pptm" or ".odt" or ".ods" or ".odp");

    private DiffReviewDocument(IReadOnlyList<DiffReviewBlock> blocks) => Blocks = blocks;
    private int[] _changeLines = [];
    public IReadOnlyList<DiffReviewBlock> Blocks { get; }
    public int ChangeCount => _changeLines.Length;
    public IReadOnlyList<DiffDisplayRow> DisplayRows { get; private set; } = [];

    public static DiffReviewDocument Create(GitDiffResult diff, CancellationToken cancellation = default)
    {
        var blocks = new List<DiffReviewBlock>();
        var changeIndex = 0;
        foreach (var hunk in diff.Hunks)
        {
            var lines = new List<(GitDiffLine Line, bool Newline)>();
            foreach (var line in hunk.Lines)
            {
                if (line.Kind == "meta")
                {
                    if (lines.Count > 0) lines[^1] = (lines[^1].Line, false);
                }
                else lines.Add((line, true));
            }
            var index = 0;
            while (index < lines.Count)
            {
                cancellation.ThrowIfCancellationRequested();
                var changed = lines[index].Line.Kind != "context";
                var before = new List<DiffTextLine>();
                var after = new List<DiffTextLine>();
                while (index < lines.Count && (lines[index].Line.Kind != "context") == changed)
                {
                    var (line, newline) = lines[index++];
                    if (line.OldLine is { } old) before.Add(new(old, line.Text, newline));
                    if (line.NewLine is { } next) after.Add(new(next, line.Text, newline));
                }
                var rows = new List<DiffReviewRow>();
                for (var row = 0; row < Math.Max(before.Count, after.Count); row++)
                {
                    cancellation.ThrowIfCancellationRequested();
                    var left = row < before.Count ? before[row] : null;
                    var right = row < after.Count ? after[row] : null;
                    (IReadOnlyList<DiffWordRange>, IReadOnlyList<DiffWordRange>) words = changed ? WordRanges(left?.Text, right?.Text) : ([], []);
                    rows.Add(new(left, right, words.Item1, words.Item2));
                }
                blocks.Add(new(changed, rows, changed ? changeIndex++ : -1));
            }
        }
        var document = new DiffReviewDocument(blocks);
        document.Rebuild();
        return document;
    }

    private static (IReadOnlyList<DiffWordRange>, IReadOnlyList<DiffWordRange>) WordRanges(string? before, string? after)
    {
        if (before is null || after is null || before == after) return ([], []);
        // Avoid a quadratic character diff on generated/minified lines.
        if (Math.Max(before.Length, after.Length) > 512)
        {
            var start = 0;
            while (start < Math.Min(before.Length, after.Length) && before[start] == after[start]) start++;
            var end = 0;
            while (end < Math.Min(before.Length, after.Length) - start && before[^(end + 1)] == after[^(end + 1)]) end++;
            return ([new(start, before.Length - start - end)], [new(start, after.Length - start - end)]);
        }
        var diff = new Differ().CreateCharacterDiffs(before, after, false);
        return (diff.DiffBlocks.Select(block => new DiffWordRange(block.DeleteStartA, block.DeleteCountA)).ToArray(),
            diff.DiffBlocks.Select(block => new DiffWordRange(block.InsertStartB, block.InsertCountB)).ToArray());
    }

    public void Rebuild()
    {
        var display = new List<DiffDisplayRow>();
        var changes = new List<int>();
        foreach (var block in Blocks)
        {
            if (block.Changed) changes.Add(display.Count + 1);
            if (block.CanFold) display.Add(new(block, null));
            display.AddRange(block.Rows.Select(row => new DiffDisplayRow(block, row)));
        }
        DisplayRows = display;
        _changeLines = changes.ToArray();
    }

    public string Text(bool before) => DisplayRows.Count == 0 ? string.Empty : string.Join('\n', DisplayRows.Select(row => row.Text(before))) + "\n";

    public int ChangeLine(int index) => _changeLines[index];

    // Folding retains source in the editor document. Strip the synthetic fold headers and
    // alignment padding from selected text, including selections across a folded range.
    public string Copy(bool before, int start, int length)
    {
        var output = new StringBuilder();
        var offset = 0;
        var end = start + length;
        foreach (var display in DisplayRows)
        {
            var text = display.Text(before);
            var lineEnd = offset + text.Length;
            if (offset >= end) break;
            if (lineEnd + 1 > start)
            {
                if (!display.IsFold && display.Line(before) is { } source)
                {
                    var from = Math.Clamp(start - offset, 0, text.Length);
                    var to = Math.Clamp(end - offset, from, text.Length);
                    output.Append(text.AsSpan(from, to - from));
                    if (end > lineEnd && source.HasNewline) output.Append('\n');
                }
            }
            offset = lineEnd + 1;
        }
        return output.ToString();
    }
}
