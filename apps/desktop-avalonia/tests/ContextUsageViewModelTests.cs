using System.Text.Json;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Tests;

public sealed class ContextUsageViewModelTests
{
    [Fact]
    public void UsageIsUnknownWithoutModelLimit()
    {
        var usage = new ContextUsageViewModel(() => null);

        usage.Update(Usage(1_200, 300, 50));

        Assert.False(usage.Known);
        Assert.True(usage.IsUnknown);
        Assert.Equal("Unavailable", usage.Text);
        Assert.Equal("1.2k", usage.InputTokenText);
    }

    [Theory]
    [InlineData(50_000UL, false, false, true)]
    [InlineData(80_000UL, true, false, false)]
    [InlineData(95_000UL, false, true, false)]
    public void PercentBandsFollowModelLimit(ulong input, bool warning, bool danger, bool normal)
    {
        var usage = new ContextUsageViewModel(() => Model(autoCompact: 100_000));

        usage.Update(Usage(input, 0, null));

        Assert.True(usage.Known);
        Assert.Equal(warning, usage.IsWarning);
        Assert.Equal(danger, usage.IsDanger);
        Assert.Equal(normal, usage.IsNormal);
        Assert.Equal($"{input / 1_000d:0.#}k / 100k tokens", usage.Text);
    }

    [Fact]
    public void ResetClearsTokensAndCollapses()
    {
        var usage = new ContextUsageViewModel(() => Model(autoCompact: 100_000)) { Expanded = true };
        usage.Update(Usage(10, 20, 30));

        usage.Reset();

        Assert.False(usage.Known);
        Assert.False(usage.Expanded);
        Assert.Equal("--", usage.InputTokenText);
        Assert.Equal("--", usage.CachedTokenText);
    }

    private static AgentUsage Usage(ulong input, ulong output, ulong? cacheRead) =>
        JsonSerializer.Deserialize<AgentUsage>(
            JsonSerializer.Serialize(new { input_tokens = input, output_tokens = output, total_tokens = input + output, cache_read_tokens = cacheRead }),
            new JsonSerializerOptions { PropertyNameCaseInsensitive = true })!;

    private static ModelItem Model(ulong autoCompact) =>
        new("model", "provider", "Provider", "configured", false, false, false, string.Empty, string.Empty, [], 200_000, autoCompact, 8_000);
}
