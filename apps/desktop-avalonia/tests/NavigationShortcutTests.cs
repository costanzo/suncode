using Avalonia.Input;
using SunCode.Desktop.Views.Projects;

namespace SunCode.Desktop.Tests;

public sealed class NavigationShortcutTests
{
    [Theory]
    [InlineData(KeyModifiers.Meta)]
    [InlineData(KeyModifiers.Control)]
    public void CommandOrControlOneTogglesProjectNavigation(KeyModifiers modifiers)
    {
        Assert.True(WorkspaceWindow.IsToggleNavigationShortcut(Key.D1, modifiers));
    }

    [Fact]
    public void OneWithoutCommandModifierDoesNotToggleProjectNavigation()
    {
        Assert.False(WorkspaceWindow.IsToggleNavigationShortcut(Key.D1, KeyModifiers.None));
    }

    [Theory]
    [InlineData(KeyModifiers.Meta)]
    [InlineData(KeyModifiers.Control)]
    public void CommandOrControlNineTogglesGitViewer(KeyModifiers modifiers)
    {
        Assert.True(WorkspaceWindow.IsToggleGitViewerShortcut(Key.D9, modifiers));
    }

    [Fact]
    public void NineWithoutCommandModifierDoesNotToggleGitViewer()
    {
        Assert.False(WorkspaceWindow.IsToggleGitViewerShortcut(Key.D9, KeyModifiers.None));
    }
}
