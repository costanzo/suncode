using System.Collections.Specialized;
using SunCode.Desktop.Infrastructure;

namespace SunCode.Desktop.Tests;

public sealed class BulkObservableCollectionTests
{
    [Fact]
    public void ReplaceAllRaisesOneResetAndPreservesOrder()
    {
        var collection = new BulkObservableCollection<int> { 9 };
        var changes = new List<NotifyCollectionChangedEventArgs>();
        collection.CollectionChanged += (_, change) => changes.Add(change);

        collection.ReplaceAll([3, 1, 2]);

        Assert.Equal([3, 1, 2], collection);
        var change = Assert.Single(changes);
        Assert.Equal(NotifyCollectionChangedAction.Reset, change.Action);
    }
}
