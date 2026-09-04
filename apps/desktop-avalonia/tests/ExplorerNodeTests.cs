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
        var nested = new ExplorerNode("ceps-app", "/projects/ceps-app", "directory");
        var markdown = new ExplorerNode("README.md", "README.md", "file");
        var source = new ExplorerNode("Program.cs", "Program.cs", "file");
        var config = new ExplorerNode("settings.json", "settings.json", "file");

        Assert.False(root.HasPathSubtitle);
        Assert.True(root.IsRoot);
        Assert.True(dependency.HasPathSubtitle);
        Assert.True(dependency.IsDependencyRoot);
        Assert.True(dependency.ShowPathSubtitle);
        Assert.Equal("lib", dependency.PathSubtitle);
        Assert.False(nested.ShowPathSubtitle);
        Assert.Equal(string.Empty, nested.PathSubtitle);
        Assert.Equal("/Assets/icons/folder.svg", nested.IconPath);
        Assert.Equal("/Assets/icons/sidebar-project.svg", root.IconPath);
        Assert.Equal("/Assets/icons/file-markdown.svg", markdown.IconPath);
        Assert.Equal("/Assets/icons/file-code.svg", source.IconPath);
        Assert.Equal("/Assets/icons/file-config.svg", config.IconPath);
    }
}
