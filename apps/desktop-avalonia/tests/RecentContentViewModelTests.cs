using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Tests;

public sealed class RecentContentViewModelTests
{
    [Fact]
    public void AddIfMissingKeepsItemsUniqueWithoutChangingCurrent()
    {
        var recent = new RecentContentViewModel();
        var file = RecentContentItem.FromFile(new ExplorerNode("App.axaml", "App.axaml", "file"));
        recent.Remember(file);

        recent.AddIfMissing(RecentContentItem.FromFile(new ExplorerNode("App.axaml", "App.axaml", "file")));
        recent.AddIfMissing(RecentContentItem.FromFile(new ExplorerNode("Program.cs", "Program.cs", "file")));

        Assert.Equal(["file:project:App.axaml", "file:project:Program.cs"], recent.Items.Select(item => item.ContentId));
        Assert.True(recent.Items[0].IsCurrent);
        Assert.False(recent.Items[1].IsCurrent);
    }

    [Fact]
    public void ReplaceAllAndClearRefreshSwitcherOptions()
    {
        var recent = new RecentContentViewModel();
        var files = new[]
        {
            RecentContentItem.FromFile(new ExplorerNode("a.cs", "a.cs", "file")),
            RecentContentItem.FromFile(new ExplorerNode("b.cs", "b.cs", "file")),
        };

        recent.ReplaceAll(files);
        Assert.Equal(2, recent.Options.Count);
        Assert.Equal("2 / 20", recent.CountText);

        recent.Clear();
        Assert.Empty(recent.Items);
        Assert.False(recent.HasOptions);
    }

    [Fact]
    public void ToUiStatePreservesOrderAndIdentity()
    {
        var recent = new RecentContentViewModel();
        recent.Remember(RecentContentItem.FromFile(new ExplorerNode("lib.rs", "src/lib.rs", "file", "dep-1")));
        recent.Remember(RecentContentItem.FromFile(new ExplorerNode("App.axaml", "App.axaml", "file")));

        var state = recent.ToUiState();

        Assert.Collection(
            state,
            first =>
            {
                Assert.Equal("file", first.Kind);
                Assert.Equal("App.axaml", first.Path);
                Assert.Null(first.DependencyId);
            },
            second =>
            {
                Assert.Equal("src/lib.rs", second.Path);
                Assert.Equal("dep-1", second.DependencyId);
            });
    }
}
