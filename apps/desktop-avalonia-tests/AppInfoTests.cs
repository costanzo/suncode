using SunCode.Desktop.Models;

namespace SunCode.Desktop.Tests;

public sealed class AppInfoTests
{
    [Fact]
    public void DisplayVersionUsesTheSemverDisplayFormat()
    {
        Assert.Equal("v0.0.1", AppInfo.DisplayVersion);
    }
}
