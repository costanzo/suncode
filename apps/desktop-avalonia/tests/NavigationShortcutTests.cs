using Avalonia.Input;
using SunCode.Desktop.Views.Shell;

namespace SunCode.Desktop.Tests;

public sealed class NavigationShortcutTests
{
    [Theory]
    [InlineData(KeyModifiers.Meta)]
    [InlineData(KeyModifiers.Control)]
    public void CommandOrControlOneTogglesProjectNavigation(KeyModifiers modifiers)
    {
        Assert.True(MainWindow.IsToggleNavigationShortcut(Key.D1, modifiers, isProjectOpen: true));
    }

    [Fact]
    public void OneWithoutCommandModifierDoesNotToggleProjectNavigation()
    {
        Assert.False(MainWindow.IsToggleNavigationShortcut(Key.D1, KeyModifiers.None, isProjectOpen: true));
    }

    [Fact]
    public void ShortcutDoesNotToggleNavigationOutsideAProject()
    {
        Assert.False(MainWindow.IsToggleNavigationShortcut(Key.D1, KeyModifiers.Meta, isProjectOpen: false));
    }

    [Theory]
    [InlineData(KeyModifiers.Meta)]
    [InlineData(KeyModifiers.Control)]
    public void CommandOrControlNineTogglesGitViewer(KeyModifiers modifiers)
    {
        Assert.True(MainWindow.IsToggleGitViewerShortcut(Key.D9, modifiers, isProjectOpen: true));
    }

    [Fact]
    public void NineWithoutCommandModifierDoesNotToggleGitViewer()
    {
        Assert.False(MainWindow.IsToggleGitViewerShortcut(Key.D9, KeyModifiers.None, isProjectOpen: true));
    }

    [Fact]
    public void ShortcutDoesNotToggleGitViewerOutsideAProject()
    {
        Assert.False(MainWindow.IsToggleGitViewerShortcut(Key.D9, KeyModifiers.Meta, isProjectOpen: false));
    }
}
