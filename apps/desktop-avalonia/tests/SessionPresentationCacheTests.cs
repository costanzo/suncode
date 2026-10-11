using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Tests;

public sealed class SessionPresentationCacheTests
{
    [Fact]
    public void RetainedSessionIdsRequiresRecentViewAndActivityWithinLastDay()
    {
        var now = DateTimeOffset.UtcNow;
        var sessions = Enumerable.Range(0, 14)
            .Select(index => new SessionItem(
                $"session-{index}",
                $"Session {index}",
                (index is 0 or 1 or 2 or 3 or 4 or 5 or 6 or 7 or 8 or 10
                    ? now.AddHours(-index)
                    : now.AddDays(-2).AddMinutes(-index)).ToString("O"),
                false))
            .ToArray();
        var recentlyViewed = Enumerable.Range(0, 10).Select(index => $"session-{index}");

        var retained = SessionPresentationCache.RetainedSessionIds(sessions, recentlyViewed, now);

        Assert.Equal(9, retained.Count);
        Assert.Contains("session-0", retained);
        Assert.Contains("session-8", retained);
        Assert.DoesNotContain("session-9", retained);
        Assert.DoesNotContain("session-10", retained);
    }

    [Fact]
    public void RetainedSessionIdsUsesOnlyFirstTenRecentlyViewedSessions()
    {
        var now = DateTimeOffset.UtcNow;
        var sessions = Enumerable.Range(0, 12)
            .Select(index => new SessionItem(
                $"session-{index}",
                $"Session {index}",
                now.AddMinutes(-index).ToString("O"),
                false))
            .ToArray();

        var recentlyViewed = Enumerable.Range(0, 12).Select(index => $"session-{index}");
        var retained = SessionPresentationCache.RetainedSessionIds(sessions, recentlyViewed, now);

        Assert.Equal(10, retained.Count);
        Assert.DoesNotContain("session-10", retained);
        Assert.DoesNotContain("session-11", retained);
    }

    [Fact]
    public void RetainedSessionIdsIgnoresRecentViewsThatDoNotExistInTheProject()
    {
        var now = DateTimeOffset.UtcNow;
        var sessions = new[]
        {
            new SessionItem("session-0", "Session 0", now.ToString("O"), false),
            new SessionItem("session-1", "Session 1", now.ToString("O"), false)
        };

        var retained = SessionPresentationCache.RetainedSessionIds(
            sessions,
            ["missing-session", "session-1", "session-0"],
            now);

        Assert.Equal(["session-0", "session-1"], retained.OrderBy(id => id, StringComparer.Ordinal));
    }

    [Fact]
    public void RetainedSessionIdsExcludesFutureActivityTimestamps()
    {
        var now = DateTimeOffset.UtcNow;
        var sessions = new[]
        {
            new SessionItem("session-0", "Session 0", now.AddMinutes(1).ToString("O"), false)
        };

        var retained = SessionPresentationCache.RetainedSessionIds(sessions, ["session-0"], now);

        Assert.Empty(retained);
    }
}
