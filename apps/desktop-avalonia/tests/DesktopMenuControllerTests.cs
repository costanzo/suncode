using Avalonia.Controls;
using Avalonia.Input;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Tests;

public sealed class DesktopMenuControllerTests
{
    private static NativeMenuItem Item(NativeMenu menu, string label) =>
        menu.Items.OfType<NativeMenuItem>().Single(item => item.Header == label);

    private static DesktopMenuController Create(Func<DesktopViewModel?>? model = null) =>
        new(() => null, model ?? (() => null), () => null, () => Task.CompletedTask);

    [Fact]
    public void ProjectMenuUsesTheCompleteIdeHierarchy()
    {
        var controller = new DesktopMenuController(
            () => null,
            () => null,
            () => null,
            () => Task.CompletedTask);

        var headers = controller.Menu.Items
            .OfType<NativeMenuItem>()
            .Select(item => item.Header ?? string.Empty)
            .ToArray();

        Assert.Equal(
            ["File", "Edit", "Navigate", "Session", "Agent", "View", "Window", "Help"],
            headers);
    }

    [Fact]
    public void WindowsMenuGroupsMirrorTheNativeCommandHierarchy()
    {
        var controller = Create();
        Assert.Equal(
            controller.Menu.Items.OfType<NativeMenuItem>().Select(item => item.Header?.ToString()),
            controller.BarGroups.Select(item => item.Header?.ToString()));

        var nativeFile = Item(controller.Menu, "File").Menu!;
        var barFile = controller.BarGroups.Single(item => item.Header?.ToString() == "File");
        Assert.Equal(nativeFile.Items.Count, barFile.Items.Count);
        Assert.Equal("Open Project…", Assert.IsType<MenuItem>(barFile.Items[0]).Header);
        Assert.Equal(new KeyGesture(Key.O, OperatingSystem.IsMacOS() ? KeyModifiers.Meta : KeyModifiers.Control),
            Assert.IsType<MenuItem>(barFile.Items[0]).InputGesture);
        Assert.False(Assert.IsType<MenuItem>(barFile.Items[1]).IsEnabled);
        Assert.Equal("No Recent Projects", Assert.IsType<MenuItem>(Assert.IsType<MenuItem>(barFile.Items[1]).Items[0]).Header);
    }

    [Fact]
    public void DynamicMenusIncludeAnExplicitEmptyState()
    {
        var controller = new DesktopMenuController(
            () => null,
            () => null,
            () => null,
            () => Task.CompletedTask);

        var window = controller.Menu.Items
            .OfType<NativeMenuItem>()
            .Single(item => item.Header == "Window");
        var openWindows = window.Menu!.Items
            .OfType<NativeMenuItem>()
            .Single(item => item.Header == "Open Windows");

        Assert.Contains(openWindows.Menu!.Items.OfType<NativeMenuItem>(), item =>
            item.Header == "No Open Windows" && !item.IsEnabled);
    }

    [Fact]
    public void NoWindowOrProjectDisablesSessionAndAgentActions()
    {
        var controller = Create();
        var session = Item(controller.Menu, "Session").Menu!;
        var agent = Item(controller.Menu, "Agent").Menu!;

        Assert.All(session.Items.OfType<NativeMenuItem>().Where(item => item is not NativeMenuItemSeparator), item => Assert.False(item.IsEnabled));
        Assert.All(agent.Items.OfType<NativeMenuItem>().Where(item => item is not NativeMenuItemSeparator), item => Assert.False(item.IsEnabled));
    }

    [Fact]
    public void RefreshRebuildsDynamicEntriesForTheCurrentProjectContext()
    {
        using var first = new DesktopViewModel();
        using var second = new DesktopViewModel();
        first.Projects.Add(new ProjectItem("first", "First project", "/first"));
        first.Sessions.Add(new SessionItem("first-session", "First session", "", false));
        second.Projects.Add(new ProjectItem("second", "Second project", "/second"));
        second.Sessions.Add(new SessionItem("second-session", "Second session", "", false));
        DesktopViewModel current = first;
        var controller = Create(() => current);
        var navigate = Item(controller.Menu, "Navigate").Menu!;
        var projects = Item(navigate, "Projects").Menu!;
        var sessions = Item(navigate, "Sessions").Menu!;

        Assert.Equal("First project", Assert.Single(projects.Items.OfType<NativeMenuItem>()).Header);
        Assert.Equal("First session", Assert.Single(sessions.Items.OfType<NativeMenuItem>()).Header);
        current = second;
        controller.Refresh();
        Assert.Equal("Second project", Assert.Single(projects.Items.OfType<NativeMenuItem>()).Header);
        Assert.Equal("Second session", Assert.Single(sessions.Items.OfType<NativeMenuItem>()).Header);

        second.Sessions.Clear();
        controller.Refresh();
        var empty = Assert.Single(sessions.Items.OfType<NativeMenuItem>());
        Assert.Equal("No Sessions", empty.Header);
        Assert.False(empty.IsEnabled);
    }

    [Fact]
    public void ViewChecksFollowStoredDestinationsEvenAtCompactWidths()
    {
        using var model = new DesktopViewModel();
        model.Layout.NavigationVisible = true;
        model.Layout.ExplorerVisible = false;
        model.Layout.ReviewVisible = true;
        var controller = Create(() => model);
        var view = Item(controller.Menu, "View").Menu!;
        Assert.True(Item(view, "Sessions").IsChecked);
        Assert.False(Item(view, "Explorer").IsChecked);
        Assert.True(Item(view, "Review").IsChecked);

        model.Layout.ToggleExplorer();
        model.Layout.ToggleChildSessions();
        model.Layout.UpdateLayoutWidth(620);
        controller.Refresh();

        Assert.False(Item(view, "Sessions").IsChecked);
        Assert.True(Item(view, "Explorer").IsChecked);
        Assert.False(Item(view, "Review").IsChecked);
        Assert.True(Item(view, "Child Sessions").IsChecked);
        Assert.False(model.Layout.EffectiveExplorerSidebarVisible);
    }

    [Fact]
    public void CheckpointMenuOnlyListsRestorableChangesAndUpdatesAfterRemoval()
    {
        using var model = new DesktopViewModel();
        model.Checkpoints.Add(new CheckpointItem("available", "turn-1", "available", ["file.cs"]));
        model.Checkpoints.Add(new CheckpointItem("expired", "turn-2", "expired", ["old.cs"]));
        var controller = Create(() => model);
        var undo = Item(Item(controller.Menu, "Session").Menu!, "Undo Agent Changes…").Menu!;

        Assert.Equal("turn-1 — 1 file", Assert.Single(undo.Items.OfType<NativeMenuItem>()).Header);
        model.Checkpoints.Clear();
        controller.Refresh();
        var empty = Assert.Single(undo.Items.OfType<NativeMenuItem>());
        Assert.Equal("No Restorable Changes", empty.Header);
        Assert.False(empty.IsEnabled);
    }

    [Theory]
    [InlineData("File", "Open Project…", Key.O)]
    [InlineData("File", "Close Window", Key.W)]
    [InlineData("Session", "New Session…", Key.N)]
    [InlineData("View", "Project Navigation", Key.D1)]
    [InlineData("View", "Source Control", Key.D9)]
    [InlineData("Window", "Minimize", Key.M)]
    public void MenuGesturesUseThePlatformCommandModifier(string group, string label, Key key)
    {
        var controller = Create();
        var gesture = Item(Item(controller.Menu, group).Menu!, label).Gesture;

        Assert.NotNull(gesture);
        Assert.Equal(key, gesture.Key);
        Assert.Equal(OperatingSystem.IsMacOS() ? KeyModifiers.Meta : KeyModifiers.Control, gesture.KeyModifiers);
    }

    [Fact]
    public void ReadOnlyTextSupportsSelectionButNeverEditing()
    {
        var text = new TextBox { Text = "Read-only source", IsReadOnly = true };
        Assert.True(DesktopEditCommands.CanExecuteTarget(text, DesktopEditAction.SelectAll));
        foreach (var action in new[] { DesktopEditAction.Undo, DesktopEditAction.Redo, DesktopEditAction.Cut, DesktopEditAction.Paste })
            Assert.False(DesktopEditCommands.CanExecuteTarget(text, action));
        Assert.False(DesktopEditCommands.CanExecuteTarget(new Border(), DesktopEditAction.Copy));
    }
}
