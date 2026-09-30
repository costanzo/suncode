using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Tests;

public sealed class ExplorerViewModelTests
{
    [Fact]
    public void ResetRootsBuildsProjectRootAndLoadedDependencyGroup()
    {
        var host = new FakeViewModelHost { SelectedProject = new ProjectItem("project", "SunCode", "/tmp/suncode") };
        var explorer = new ExplorerViewModel(host);
        explorer.Dependencies.Add(new ProjectDependencyItem("dep-1", "shared-lib"));

        explorer.ResetRoots();

        Assert.Collection(
            explorer.Roots,
            root =>
            {
                Assert.Equal("SunCode", root.Name);
                Assert.True(root.IsRoot);
                Assert.Null(root.DependencyId);
            },
            group =>
            {
                Assert.True(group.IsGroup);
                Assert.True(group.IsLoaded);
                var dependency = Assert.Single(group.Children);
                Assert.Equal("shared-lib", dependency.Name);
                Assert.Equal("dep-1", dependency.DependencyId);
                Assert.True(dependency.IsDependency);
            });
    }

    [Fact]
    public void ResetRootsWithoutProjectLeavesTreeEmpty()
    {
        var explorer = new ExplorerViewModel(new FakeViewModelHost());

        explorer.ResetRoots();

        Assert.Empty(explorer.Roots);
    }

    [Fact]
    public async Task ReloadWithoutSdkClearsDependencies()
    {
        var host = new FakeViewModelHost { SelectedProject = new ProjectItem("project", "SunCode", "/tmp/suncode") };
        var explorer = new ExplorerViewModel(host);
        explorer.Dependencies.Add(new ProjectDependencyItem("dep-1", "shared-lib"));

        await explorer.ReloadAsync();

        Assert.False(explorer.HasDependencies);
        Assert.Empty(Assert.Single(explorer.Roots, root => root.IsGroup).Children);
        Assert.False(explorer.Roots[0].IsLoaded);
        Assert.Empty(host.Errors);
    }
}
