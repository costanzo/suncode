using SunCode.Desktop.Infrastructure;

namespace SunCode.Desktop.Tests;

public sealed class NotificationLedgerTests
{
    [Fact]
    public void Foreground_requires_an_active_visible_non_minimized_window()
    {
        Assert.True(NotificationForegroundEvaluator.IsForeground([(true, true, false)]));
        Assert.False(NotificationForegroundEvaluator.IsForeground([(false, true, false), (true, true, true)]));
    }

    [Fact]
    public void Delivered_and_foreground_suppressed_ids_are_persisted_and_deduplicated()
    {
        using var directory = new TemporaryDirectory();
        var ledger = new NotificationLedger(directory.Path);
        ledger.Record(new("primary_turn_completed", "turn-1"), NotificationDisposition.Delivered);
        ledger.Record(new("approval_requested", "approval-1"), NotificationDisposition.SuppressedForeground);

        var reloaded = new NotificationLedger(directory.Path);
        Assert.True(reloaded.Contains(new("primary_turn_completed", "turn-1")));
        Assert.True(reloaded.Contains(new("approval_requested", "approval-1")));
        Assert.Equal(2, reloaded.Snapshot().Count);
    }

    private sealed class TemporaryDirectory : IDisposable
    {
        public string Path { get; } = System.IO.Path.Combine(System.IO.Path.GetTempPath(), $"suncode-ledger-{Guid.NewGuid():N}");
        public TemporaryDirectory() => Directory.CreateDirectory(Path);
        public void Dispose() => Directory.Delete(Path, true);
    }
}
