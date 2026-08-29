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

    [Fact]
    public void PathSubtitleAndDependencyRootExposePresentationState()
    {
        var root = new ExplorerNode("suncode", ".", "directory", isRoot: true);
        var dependency = new ExplorerNode("shared-ui", "lib", "directory", "dep-1", isRoot: true, isDependency: true);

        Assert.False(root.HasPathSubtitle);
        Assert.True(root.IsRoot);
        Assert.True(dependency.HasPathSubtitle);
        Assert.True(dependency.IsDependencyRoot);
    }
}
