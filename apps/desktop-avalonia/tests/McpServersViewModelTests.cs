using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Tests;

public sealed class McpServersViewModelTests
{
    [Fact]
    public async Task MutationsFailWithoutSdkAndLeaveStateUntouched()
    {
        var host = new FakeViewModelHost();
        var viewModel = new McpServersViewModel(host);

        Assert.False(await viewModel.RetryServerAsync(null!));
        await viewModel.LoadServersAsync();
        await viewModel.StartProjectAsync();

        Assert.Empty(viewModel.Servers);
        Assert.True(viewModel.HasNoServers);
        Assert.False(viewModel.IsLoading);
        Assert.Equal(string.Empty, viewModel.StatusText);
        Assert.Empty(host.Errors);
    }

    [Fact]
    public void ProjectContextFollowsHostProject()
    {
        var host = new FakeViewModelHost();
        var viewModel = new McpServersViewModel(host);
        Assert.False(viewModel.HasProjectContext);
        Assert.Equal("Open a project to start MCP connections.", viewModel.ProjectContext);

        host.SelectedProject = new ProjectItem("project", "Project", "/tmp/project");

        Assert.True(viewModel.HasProjectContext);
        Assert.Equal("Project connection: Project", viewModel.ProjectContext);
    }

    [Fact]
    public void SelectingProjectOnWindowViewModelNotifiesMcpContext()
    {
        using var window = new DesktopViewModel();
        var changed = new List<string?>();
        window.Mcp.PropertyChanged += (_, args) => changed.Add(args.PropertyName);

        window.SetSelectedProjectForTests(new ProjectItem("project", "Project", "/tmp/project"));

        Assert.Contains(nameof(McpServersViewModel.ProjectContext), changed);
        Assert.Contains(nameof(McpServersViewModel.HasProjectContext), changed);
        Assert.Equal("Project connection: Project", window.Mcp.ProjectContext);
    }
}
