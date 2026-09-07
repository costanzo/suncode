using SunCode.Desktop.Models;

namespace SunCode.Desktop.Tests;

public sealed class AppInfoTests
{
    [Fact]
    public void DisplayVersionUsesTheSemverDisplayFormat()
    {
        Assert.Equal("v0.0.1", AppInfo.DisplayVersion);
    }

    [Fact]
    public void FormatVersionUsesTheSemverDisplayFormat()
    {
        Assert.Equal("v0.1.0", AppInfo.FormatVersion("0.1.0+build.42"));
    }
}
