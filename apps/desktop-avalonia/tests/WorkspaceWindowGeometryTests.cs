using Avalonia;
using Avalonia.Controls;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Views.ProjectWorkspace;

namespace SunCode.Desktop.Tests;

public sealed class WorkspaceWindowGeometryTests
{
    [Theory]
    [InlineData(1, 1440, 900, 240, 70)]
    [InlineData(1.25, 1440, 832, 60, 0)]
    [InlineData(1.5, 1280, 1040.0 / 1.5, 0, 0)]
    [InlineData(2, 960, 620, 0, 0)]
    public void NewWindowFitsAndCentersInTheScaledWorkingArea(
        double scaling, double width, double height, int x, int y)
    {
        var geometry = WorkspaceWindowGeometry.Resolve(new UiProjectState(), new PixelRect(0, 0, 1920, 1040), scaling);

        Assert.Equal(new Size(width, height), geometry.Size);
        Assert.Equal(new PixelPoint(x, y), geometry.Position);
        Assert.Equal(WindowState.Normal, geometry.State);
    }

    [Theory]
    [InlineData("normal", WindowState.Normal)]
    [InlineData("maximized", WindowState.Maximized)]
    [InlineData("fullscreen", WindowState.FullScreen)]
    [InlineData("minimized", WindowState.Normal)]
    public void ReopenedWindowPreservesSavedNormalBoundsAndPresentationState(string savedState, WindowState expected)
    {
        var saved = new UiProjectState
        {
            WindowWidth = 1000, WindowHeight = 700, WindowX = 160, WindowY = 80, WindowState = savedState
        };

        var geometry = WorkspaceWindowGeometry.Resolve(saved, new PixelRect(0, 0, 2560, 1400), 1.5);

        Assert.Equal(new Size(1000, 700), geometry.Size);
        Assert.Equal(new PixelPoint(160, 80), geometry.Position);
        Assert.Equal(expected, geometry.State);
    }

    [Fact]
    public void SavedPositionUsesPhysicalWindowSizeWhenClampingOnASecondaryMonitor()
    {
        var saved = new UiProjectState
        {
            WindowWidth = 1000, WindowHeight = 700, WindowX = -500, WindowY = 700
        };

        var geometry = WorkspaceWindowGeometry.Resolve(saved, new PixelRect(-2560, -200, 2560, 1400), 1.5);

        Assert.Equal(new PixelPoint(-1500, 150), geometry.Position);
    }

    [Fact]
    public void DisconnectedDisplayBoundsMoveIntoTheAvailableWorkingArea()
    {
        var saved = new UiProjectState
        {
            WindowWidth = 2000, WindowHeight = 1200, WindowX = 4000, WindowY = -1000
        };

        var geometry = WorkspaceWindowGeometry.Resolve(saved, new PixelRect(0, 0, 1920, 1080), 1.5);

        Assert.Equal(new Size(1280, 720), geometry.Size);
        Assert.Equal(new PixelPoint(0, 0), geometry.Position);
    }
}
