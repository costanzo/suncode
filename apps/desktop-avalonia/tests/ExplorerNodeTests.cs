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

    [Theory]
    [InlineData("README.md", "\uE04D", "blue")]
    [InlineData("tsconfig.json", "\uE097", "blue")]
    [InlineData("Dockerfile", "\uE025", "blue")]
    [InlineData(".gitignore", "\uE034", "ink")]
    [InlineData("unknown.bin", "\uE023", "default")]
    public void FileIconUsesFilenameSpecificSetiGlyphs(string name, string glyph, string color)
    {
        var node = new ExplorerNode(name, name, "file");

        Assert.True(node.UseSetiFileIcon);
        Assert.Equal(glyph, node.FileIconGlyph);
        Assert.Equal(color, node.FileIconColor);
    }

    [Theory]
    [InlineData("component.d.ts", "\uE099", "blue")]
    [InlineData("component.test.tsx", "\uE07D", "blue")]
    [InlineData("main.rs", "\uE082", "muted")]
    [InlineData("logo.svg", "\uE091", "purple")]
    public void FileIconFallsBackFromCompoundExtensionToExtension(string name, string glyph, string color)
    {
        var node = new ExplorerNode(name, name, "file");

        Assert.Equal(glyph, node.FileIconGlyph);
        Assert.Equal(color, node.FileIconColor);
    }

    [Fact]
    public void NonFileNodesKeepInterfaceIconPaths()
    {
        var directory = new ExplorerNode("src", "src", "directory");
        var file = new ExplorerNode("README.md", "README.md", "file");

        Assert.False(directory.UseSetiFileIcon);
        Assert.Equal("/Assets/icons/folder.svg", directory.IconPath);
        Assert.Equal("/Assets/icons/file-markdown.svg", file.IconPath);
    }

}
