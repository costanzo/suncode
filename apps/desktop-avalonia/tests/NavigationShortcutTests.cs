using Avalonia.Controls;
using Avalonia.Input;
using SunCode.Desktop.Controls;
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
    [InlineData(TrafficLightKind.Close, TrafficLightState.Normal, "1-close-1-normal.svg")]
    [InlineData(TrafficLightKind.Minimize, TrafficLightState.Hover, "2-minimize-2-hover.svg")]
    [InlineData(TrafficLightKind.Maximize, TrafficLightState.Press, "3-maximize-3-press.svg")]
    public void ActiveTrafficLightsKeepTheirInteractionAssets(TrafficLightKind kind, TrafficLightState state, string expected)
    {
        Assert.Equal(expected, TrafficLightButton.GetAsset(kind, state, isWindowActive: true));
    }

    [Theory]
    [InlineData(TrafficLightKind.Close)]
    [InlineData(TrafficLightKind.Minimize)]
    [InlineData(TrafficLightKind.Maximize)]
    public void InactiveTrafficLightsUseTheMutedAssetForEveryControl(TrafficLightKind kind)
    {
        foreach (var state in Enum.GetValues<TrafficLightState>())
            Assert.Equal("0-all-three-nofocus.svg", TrafficLightButton.GetAsset(kind, state, isWindowActive: false));
    }
}
