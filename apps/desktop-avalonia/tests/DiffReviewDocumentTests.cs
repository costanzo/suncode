using SunCode.Desktop.Infrastructure.Git;
using SunCode.Desktop.Models;

namespace SunCode.Desktop.Tests;

public sealed class DiffReviewDocumentTests
{
    [Fact]
    public void UnequalChangesKeepRealLineNumbersAndCopyWithoutPadding()
    {
        var document = DiffReviewDocument.Create(Patch([
            new("deletion", 8, null, "old one"), new("deletion", 9, null, "old two"),
            new("addition", null, 8, "new one"), new("addition", null, 9, "new two"),
            new("addition", null, 10, "new three"), new("context", 10, 11, "tail")
        ]));
        Assert.Equal(1, document.ChangeCount);
        Assert.Null(document.DisplayRows[2].Line(true));
        Assert.Equal(10, document.DisplayRows[3].Line(true)!.Number);
        Assert.Equal(11, document.DisplayRows[3].Line(false)!.Number);
        Assert.Equal("old one\nold two\ntail\n", document.Copy(true, 0, document.Text(true).Length));
        Assert.Equal("new one\nnew two\nnew three\ntail\n", document.Copy(false, 0, document.Text(false).Length));
        Assert.Equal("ld o", document.Copy(true, 1, 4));
    }

    [Fact]
    public void FoldingRetainsSourceAndNavigationUsesContiguousChangeBlocks()
    {
        var lines = Enumerable.Range(1, 70).Select(line => new GitDiffLine("context", line, line, $"unchanged {line}")).ToList();
        lines.AddRange([new("deletion", 71, null, "a"), new("addition", null, 71, "b"),
            new("context", 72, 72, "context"), new("addition", null, 73, "c")]);
        var document = DiffReviewDocument.Create(Patch(lines));
        Assert.Equal(2, document.ChangeCount);
        Assert.True(document.DisplayRows[0].IsFold);
        Assert.Equal(72, document.ChangeLine(0));
        var text = document.Copy(true, 0, document.Text(true).Length);
        Assert.Contains("unchanged 70\n", text);
        Assert.DoesNotContain('\uFFFC', text);
        document.Blocks[0].Expanded = true;
        document.Rebuild();
        Assert.Equal(text, document.Copy(true, 0, document.Text(true).Length));
        Assert.Equal(72, document.ChangeLine(0));
    }

    [Fact]
    public void CharacterRangesAndMissingFinalNewlineArePreserved()
    {
        var document = DiffReviewDocument.Create(Patch([
            new("deletion", 1, null, "  value = 20;"), new("meta", null, null, "\\ No newline at end of file"),
            new("addition", null, 1, "  value = 24;"), new("meta", null, null, "\\ No newline at end of file")
        ]));
        var row = Assert.Single(document.Blocks).Rows[0];
        Assert.Contains(row.BeforeWords, range => range.Start == 11 && range.Length == 1);
        Assert.Equal("  value = 20;", document.Copy(true, 0, document.Text(true).Length));
        Assert.Equal("  value = 24;", document.Copy(false, 0, document.Text(false).Length));
    }

    [Fact]
    public void LongLinesUseBoundedInlineComparison()
    {
        var prefix = new string('a', 20_000);
        var document = DiffReviewDocument.Create(Patch([new("deletion", 1, null, prefix + "old"), new("addition", null, 1, prefix + "new")]));
        Assert.Equal(new DiffWordRange(20_000, 3), Assert.Single(document.Blocks[0].Rows[0].BeforeWords));
    }

    internal static GitDiffResult Patch(IReadOnlyList<GitDiffLine> lines) => new("", false, lines.Count(line => line.Kind == "addition"),
        lines.Count(line => line.Kind == "deletion"), [new("@@", lines)], false);
}
