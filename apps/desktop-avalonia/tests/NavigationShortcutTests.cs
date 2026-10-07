using Avalonia.Controls;
using Avalonia.Input;
using SunCode.Desktop.Views.ProjectWorkspace;

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

    [Theory]
    [InlineData(WindowState.Normal, WindowState.Maximized)]
    [InlineData(WindowState.Maximized, WindowState.Normal)]
    public void DoubleClickingTitleBarTogglesMaximizedState(WindowState current, WindowState expected)
    {
        Assert.Equal(expected, WorkspaceWindow.GetTitleBarDoubleTapTargetState(current));
    }

    [Theory]
    [InlineData(true, true)]
    [InlineData(false, false)]
    public void OnlyMacOsUsesMaximizedStateForTitleBarDoubleClick(bool isMacOS, bool expected)
    {
        Assert.Equal(expected, WorkspaceWindow.UsesMaximizedStateForTitleBarDoubleTap(isMacOS));
    }

    [Fact]
    public void ProjectSwitcherButtonDoesNotStartTitleBarDragging()
    {
        Assert.True(WorkspaceWindow.OriginatesFromButton(new Button()));
        Assert.False(WorkspaceWindow.OriginatesFromButton(new Border()));
    }

    [Theory]
    [InlineData("close", "normal", "1-close-1-normal.svg")]
    [InlineData("minimize", "hover", "2-minimize-2-hover.svg")]
    [InlineData("maximize", "press", "3-maximize-3-press.svg")]
    public void ActiveTrafficLightsKeepTheirInteractionAssets(string kind, string state, string expected)
    {
        Assert.Equal(expected, WorkspaceWindow.GetTrafficLightAsset(kind, state, isActive: true));
    }

    [Theory]
    [InlineData("close")]
    [InlineData("minimize")]
    [InlineData("maximize")]
    public void InactiveTrafficLightsUseTheMutedAssetForEveryControl(string kind)
    {
        Assert.Equal("0-all-three-nofocus.svg", WorkspaceWindow.GetTrafficLightAsset(kind, "normal", isActive: false));
        Assert.Equal("0-all-three-nofocus.svg", WorkspaceWindow.GetTrafficLightAsset(kind, "hover", isActive: false));
    }
}
