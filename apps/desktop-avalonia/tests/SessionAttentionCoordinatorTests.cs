using SunCode.Desktop.Infrastructure;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Tests;

public sealed class SessionAttentionCoordinatorTests
{
    [Fact]
    public void Builds_bounded_sensitive_content_free_child_approval_notification()
    {
        var attention = new AttentionEvent(
            "approval_requested", "approval-1", "project-1", "Project",
            "child-1", "Review implementation", "child", "parent-1",
            "turn-1", "2026-09-23T00:00:00.000Z");

        var notification = SessionAttentionCoordinator.BuildNotification(attention);

        Assert.Equal("SunCode · Project", notification.Title);
        Assert.Equal("Review implementation needs your approval", notification.Body);
        Assert.Equal("parent-1", notification.Activation.ParentSessionId);
        Assert.Equal("child-1", notification.Activation.ChildSessionId);
        Assert.DoesNotContain("/", notification.Body);
    }
}
