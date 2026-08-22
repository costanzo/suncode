using SunCode.Desktop.Models;

namespace SunCode.Desktop.Tests;

public sealed class ProviderTraceTurnItemTests
{
    [Fact]
    public void DurationUsesStartedAndCompletedTimestamps()
    {
        var turn = CreateTurn(
            state: "completed",
            startedAt: "2026-08-22T10:00:00.000Z",
            completedAt: "2026-08-22T10:00:01.250Z");

        Assert.Equal("1.25 s", turn.DurationText);
    }

    [Fact]
    public void DurationFallsBackToCreatedTimestampForOlderRecords()
    {
        var turn = CreateTurn(
            state: "completed",
            createdAt: "2026-08-22T10:00:00.000Z",
            completedAt: "2026-08-22T10:00:00.125Z");

        Assert.Equal("125 ms", turn.DurationText);
    }

    [Fact]
    public void RunningDurationUsesCurrentTime()
    {
        var turn = CreateTurn(
            state: "calling_model",
            startedAt: DateTimeOffset.UtcNow.AddSeconds(-2).ToString("O"));

        var duration = turn.DurationText;

        Assert.EndsWith(" s", duration);
        Assert.NotEqual("—", duration);
    }

    [Fact]
    public void DurationIsUnavailableWhenStartTimestampIsInvalid()
    {
        var turn = CreateTurn(
            state: "completed",
            createdAt: "invalid",
            startedAt: "invalid",
            completedAt: "2026-08-22T10:00:01.000Z");

        Assert.Equal("—", turn.DurationText);
    }

    [Fact]
    public void DurationClampsNegativeElapsedTimeToZero()
    {
        var turn = CreateTurn(
            state: "completed",
            startedAt: "2026-08-22T10:00:01.000Z",
            completedAt: "2026-08-22T10:00:00.000Z");

        Assert.Equal("0 ms", turn.DurationText);
    }

    private static ProviderTraceTurnItem CreateTurn(
        string state,
        string createdAt = "2026-08-22T09:59:59.000Z",
        string startedAt = "",
        string completedAt = "") =>
        new(
            "turn-12345678",
            state,
            "gpt-5.6-sol",
            createdAt,
            startedAt,
            completedAt,
            0,
            0,
            0,
            1,
            []);
}
