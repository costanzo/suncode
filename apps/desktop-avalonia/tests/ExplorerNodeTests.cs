using SunCode.Desktop.Models;

namespace SunCode.Desktop.Tests;

public sealed class ExplorerNodeTests
{
    [Fact]
    public void ExpansionRotationTracksTreeExpansion()
    {
        var node = new ExplorerNode("src", "src", "directory");
        var changed = new List<string?>();
        node.PropertyChanged += (_, args) => changed.Add(args.PropertyName);

        Assert.Equal(0, node.ExpansionRotation);

        node.IsExpanded = true;

        Assert.Equal(90, node.ExpansionRotation);
        Assert.Contains(nameof(ExplorerNode.ExpansionRotation), changed);
    }

}
