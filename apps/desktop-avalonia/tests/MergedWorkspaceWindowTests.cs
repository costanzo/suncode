using SunCode.Desktop.Views.ProjectWorkspace;

namespace SunCode.Desktop.Tests;

public sealed class MergedWorkspaceWindowTests
{
    [Theory]
    [InlineData(0, 0, false)]
    [InlineData(20, 10, false)]
    [InlineData(28, 0, true)]
    [InlineData(0, -28, true)]
    [InlineData(21, 21, true)]
    public void TearOffRequiresThresholdDistance(double deltaX, double deltaY, bool expected)
    {
        Assert.Equal(expected, MergedWorkspaceWindow.ShouldTearOff(deltaX, deltaY));
    }
}
