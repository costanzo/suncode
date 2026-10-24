using Avalonia.Controls;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;

namespace SunCode.Desktop.Tests;

public sealed class DesktopDockMenuControllerTests
{
    private static NativeMenuItem Item(NativeMenu menu, string label) =>
        menu.Items.OfType<NativeMenuItem>().Single(item => item.Header == label);

    private static DesktopDockMenuController Create(
        Func<IReadOnlyList<ProjectItem>>? recent = null,
        Func<IReadOnlyList<ProjectItem>>? open = null,
        Func<ProjectItem, Task>? activate = null,
        Func<bool>? enabled = null) =>
        new(recent ?? (() => []), open ?? (() => []), activate ?? (_ => Task.CompletedTask),
            () => Task.CompletedTask, () => { }, enabled);

    [Fact]
    public void OpenWindowsUsesTheRegistryEvenWhenTheRecentCatalogIsStale()
    {
        var open = new ProjectItem("open", "Open project", "/open");
        var controller = Create(
            () => [new ProjectItem("recent", "Recent project", "/recent")],
            () => [open]);

        Assert.Equal(["Open Project…", "Open Windows", "Recent Projects", "Show Project Hub"],
            controller.Menu.Items.OfType<NativeMenuItem>().Where(item => item is not NativeMenuItemSeparator)
                .Select(item => item.Header ?? string.Empty).ToArray());
        Assert.Equal("Open project", Assert.Single(Item(controller.Menu, "Open Windows").Menu!.Items.OfType<NativeMenuItem>()).Header);
        Assert.Equal("Recent project", Assert.Single(Item(controller.Menu, "Recent Projects").Menu!.Items.OfType<NativeMenuItem>()).Header);
    }

    [Fact]
    public void RefreshRemovesClosedProjectsAndProvidesDisabledEmptyRows()
    {
        var projects = new List<ProjectItem> { new("project", "Project", "/project") };
        var controller = Create(() => projects, () => projects);
        projects.Clear();
        controller.Refresh();

        var noOpen = Assert.Single(Item(controller.Menu, "Open Windows").Menu!.Items.OfType<NativeMenuItem>());
        var noRecent = Assert.Single(Item(controller.Menu, "Recent Projects").Menu!.Items.OfType<NativeMenuItem>());
        Assert.Equal("No Open Windows", noOpen.Header);
        Assert.Equal("No Recent Projects", noRecent.Header);
        Assert.False(noOpen.IsEnabled);
        Assert.False(noRecent.IsEnabled);
    }

    [Fact]
    public void RecentProjectsPreservesCatalogOrderAndLimitsItsLength()
    {
        var projects = Enumerable.Range(0, 14).Select(index => new ProjectItem($"{index}", $"Project {index}", $"/{index}")).ToArray();
        var controller = Create(() => projects);
        Assert.Equal(projects.Take(10).Select(item => item.DisplayName),
            Item(controller.Menu, "Recent Projects").Menu!.Items.OfType<NativeMenuItem>().Select(item => item.Header ?? string.Empty));
    }

    [Fact]
    public void ProjectsWithTheSameNameIncludeTheirPaths()
    {
        var controller = Create(() => [new("first", "Project", "/first"), new("second", "Project", "/second")]);
        Assert.Equal(["Project — /first", "Project — /second"],
            Item(controller.Menu, "Recent Projects").Menu!.Items.OfType<NativeMenuItem>().Select(item => item.Header ?? string.Empty).ToArray());
    }

    [Fact]
    public async Task OpenProjectSelectionRechecksTheLiveRegistryBeforeInvoking()
    {
        var project = new ProjectItem("project", "Project", "/project");
        var open = new List<ProjectItem> { project };
        ProjectItem? activated = null;
        var controller = Create(open: () => open, activate: item => { activated = item; return Task.CompletedTask; });
        var command = Assert.IsType<AsyncRelayCommand>(Item(Item(controller.Menu, "Open Windows").Menu!, "Project").Command);
        await command.ExecuteAsync();
        Assert.Same(project, activated);

        activated = null;
        open.Clear();
        await command.ExecuteAsync();
        Assert.Null(activated);
    }

    [Fact]
    public async Task ModalStateDisablesAndRechecksCommandsAtInvocation()
    {
        bool enabled = true;
        bool activated = false;
        var controller = Create(() => [new("project", "Project", "/project")],
            activate: _ => { activated = true; return Task.CompletedTask; }, enabled: () => enabled);
        var command = Assert.IsType<AsyncRelayCommand>(Item(Item(controller.Menu, "Recent Projects").Menu!, "Project").Command);
        enabled = false;
        await command.ExecuteAsync();
        Assert.False(activated);
        controller.Refresh();
        Assert.All(controller.Menu.Items.OfType<NativeMenuItem>().Where(item => item is not NativeMenuItemSeparator), item => Assert.False(item.IsEnabled));
        enabled = true;
        controller.Refresh();
        Assert.All(controller.Menu.Items.OfType<NativeMenuItem>().Where(item => item is not NativeMenuItemSeparator), item => Assert.True(item.IsEnabled));
    }
}
