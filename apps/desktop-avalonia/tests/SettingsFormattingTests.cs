using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Tests;

public sealed class SettingsFormattingTests
{
    [Theory]
    [InlineData("allowed", "Allowed")]
    [InlineData("primary_display", "Primary display")]
    [InlineData("agent", "Agent")]
    [InlineData("needs_review_now", "Needs review now")]
    [InlineData("", "—")]
    [InlineData(null, "—")]
    public void ComputerValueLocalizesKnownValuesAndHumanizesOthers(string? value, string expected) =>
        Assert.Equal(expected, SettingsFormatting.ComputerValue(value));

    [Theory]
    [InlineData("not_started", "Not started")]
    [InlineData("user_controlled", "User controlled")]
    [InlineData("warming_up", "Warming up")]
    [InlineData(" ", "—")]
    public void BrowserStateLocalizesKnownValuesAndHumanizesOthers(string value, string expected) =>
        Assert.Equal(expected, SettingsFormatting.BrowserState(value));

    [Theory]
    [InlineData("ready", StatusTone.Success)]
    [InlineData("background", StatusTone.Success)]
    [InlineData("starting", StatusTone.Warning)]
    [InlineData("user_controlled", StatusTone.Warning)]
    [InlineData("failed", StatusTone.Danger)]
    [InlineData("not_started", StatusTone.Muted)]
    [InlineData(null, StatusTone.Muted)]
    public void BrowserStateToneMatchesStatusColors(string? state, StatusTone expected) =>
        Assert.Equal(expected, SettingsFormatting.BrowserStateTone(state));

    [Theory]
    [InlineData("allowed", StatusTone.Success)]
    [InlineData("denied", StatusTone.Danger)]
    [InlineData("unsupported", StatusTone.Warning)]
    [InlineData("unknown", StatusTone.Muted)]
    public void PermissionToneMatchesStatusColors(string permission, StatusTone expected) =>
        Assert.Equal(expected, SettingsFormatting.PermissionTone(permission));

    [Theory]
    [InlineData(0UL, "0 B")]
    [InlineData(1023UL, "1023 B")]
    [InlineData(1024UL, "1.0 KB")]
    [InlineData(1536UL, "1.5 KB")]
    [InlineData(5UL * 1024 * 1024, "5.0 MB")]
    [InlineData(3UL * 1024 * 1024 * 1024, "3.0 GB")]
    public void ByteSizeUsesBinaryUnits(ulong bytes, string expected)
    {
        var culture = System.Globalization.CultureInfo.CurrentCulture;
        System.Globalization.CultureInfo.CurrentCulture = System.Globalization.CultureInfo.InvariantCulture;
        try
        {
            Assert.Equal(expected, SettingsFormatting.ByteSize(bytes));
        }
        finally
        {
            System.Globalization.CultureInfo.CurrentCulture = culture;
        }
    }

    [Fact]
    public void NormalizersIgnoreFormattingOnlyDifferences()
    {
        var separator = System.IO.Path.DirectorySeparatorChar;
        Assert.Equal("/tmp/logs", SettingsFormatting.NormalizeDirectory($"  /tmp/logs{separator} "));
        Assert.Equal(string.Empty, SettingsFormatting.NormalizeDirectory(null));
        Assert.Equal("localhost\n*.internal", SettingsFormatting.NormalizeProxyBypass(" localhost \r\n\r\n*.internal\n"));
    }
}
