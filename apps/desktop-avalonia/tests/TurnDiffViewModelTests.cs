using System.Reflection;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Tests;

public sealed class TurnDiffViewModelTests
{
    [Fact]
    public async Task LoadWithoutSdkKeepsDrawerIdle()
    {
        var model = new TurnDiffViewModel(new FakeViewModelHost());

        await model.LoadAsync("session", "turn");

        Assert.Equal("idle", model.State);
        Assert.Empty(model.Error);
        Assert.Empty(model.Files);
        Assert.Empty(model.DiffLines);
        Assert.Empty(model.SessionId);
        Assert.Empty(model.TurnId);
    }

    [Fact]
    public void SelectingTextFileRendersHunksAndLineNumbers()
    {
        var model = Seeded();

        model.SelectedFile = model.Files[0];

        Assert.Equal("1 file", model.FileCountText);
        Assert.Equal("sort.py", model.SelectedPath);
        Assert.Equal("+1", model.SelectedAdditionsText);
        Assert.Equal("-1", model.SelectedDeletionsText);
        Assert.True(model.HasDiffLines);
        Assert.False(model.ShowDiffEmpty);
        Assert.True(model.ShowDiffStats);
        Assert.Equal(new DiffLineItem("hunk", "@@ -1,2 +1,2 @@", "", ""), model.DiffLines[0]);
        Assert.Equal(new DiffLineItem("context", "# sort", "1", "1"), model.DiffLines[1]);
        Assert.Equal(new DiffLineItem("deletion", "old", "2", ""), model.DiffLines[2]);
        Assert.Equal(new DiffLineItem("addition", "new", "", "2"), model.DiffLines[3]);

        model.SelectedFile = null;

        Assert.Empty(model.DiffLines);
        Assert.False(model.HasDiffLines);
        Assert.False(model.ShowDiffStats);
    }

    [Fact]
    public void SelectingBinaryFileShowsBinaryMessageWithoutTextLines()
    {
        var model = Seeded(binary: true);

        model.SelectedFile = model.Files[0];

        Assert.Empty(model.DiffLines);
        Assert.False(model.HasDiffLines);
        Assert.False(model.ShowDiffStats);
        Assert.True(model.ShowDiffEmpty);
        Assert.Equal("Binary files cannot be displayed as text.", model.EmptyMessage);
    }

    private static TurnDiffViewModel Seeded(bool binary = false)
    {
        var model = new TurnDiffViewModel(new FakeViewModelHost());
        var file = new TurnFileDiff("sort.py", "modified", 1, 1, binary,
        [
            new TurnDiffHunk(1, 2, 1, 2,
            [
                new TurnDiffLine("context", "# sort", 1, 1),
                new TurnDiffLine("deletion", "old", 2, null),
                new TurnDiffLine("addition", "new", null, 2)
            ])
        ]);
        // Seed SDK output so rendering can be tested without opening a native agent.
        typeof(TurnDiffViewModel).GetField("_diffs", BindingFlags.Instance | BindingFlags.NonPublic)!
            .SetValue(model, new Dictionary<string, TurnFileDiff> { [file.Path] = file });
        typeof(TurnDiffViewModel).GetProperty(nameof(TurnDiffViewModel.State))!
            .SetValue(model, "ready");
        model.Files.Add(new TurnChangeItem(file.Path, file.Status, file.Additions, file.Deletions, file.Binary));
        return model;
    }
}
