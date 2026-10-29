using Avalonia;
using Avalonia.Headless;

namespace SunCode.Desktop.Tests;

[CollectionDefinition("Avalonia UI", DisableParallelization = true)]
public sealed class AvaloniaUiCollection : ICollectionFixture<AvaloniaUiFixture>;

public sealed class AvaloniaUiFixture : IDisposable
{
    private readonly HeadlessUnitTestSession _session = HeadlessUnitTestSession.StartNew(typeof(Application));

    public Task Run(Action test) => _session.Dispatch(test, CancellationToken.None);

    public Task Run(Func<Task> test) => _session.Dispatch(async () =>
    {
        await test();
        return 0;
    }, CancellationToken.None);

    public void Dispose() => _session.Dispose();
}
