using System.IO.Compression;
using SunCode.Desktop.Infrastructure.Updates;

namespace SunCode.Desktop.Tests.Updates;

public sealed class UpdateArchiveTests
{
    [Theory]
    [InlineData("../escape")] [InlineData("/absolute")] [InlineData("a/../../escape")] [InlineData("a\\escape")] [InlineData("C:/escape")] [InlineData("a./file")]
    public async Task TraversalIsRefusedAndPartialExtractionRemoved(string entry)
    {
        using var f = new UpdateFixture();
        var archive=Path.Combine(f.Root,"attack.zip");File.WriteAllBytes(archive,UpdateFixture.Zip((entry,"payload")));
        var destination=Path.Combine(f.Root,"extract");
        await Assert.ThrowsAsync<UpdateException>(()=>UpdateArchive.ExtractAsync(archive,"zip",destination,default));
        Assert.False(Directory.Exists(destination));
    }
    [Fact]
    public async Task DuplicateAndCaseCollidingArchiveEntriesAreRefused()
    {
        using var f = new UpdateFixture();
        var archive=Path.Combine(f.Root,"duplicates.zip");File.WriteAllBytes(archive,UpdateFixture.Zip(("a","one"),("A","two")));
        await Assert.ThrowsAsync<UpdateException>(()=>UpdateArchive.ExtractAsync(archive,"zip",Path.Combine(f.Root,"extract"),default));
    }
    [Fact]
    public async Task SymlinkCannotRedirectLaterExtractionOutsideTheRoot()
    {
        using var f = new UpdateFixture();
        var archive=Path.Combine(f.Root,"symlink.zip");
        using(var stream=File.Create(archive))using(var zip=new ZipArchive(stream,ZipArchiveMode.Create))
        {
            var link=zip.CreateEntry("link");link.ExternalAttributes=unchecked((int)(0xa1ffu<<16));
            using(var writer=new StreamWriter(link.Open()))writer.Write("../outside");
            var file=zip.CreateEntry("link/payload");using var data=new StreamWriter(file.Open());data.Write("payload");
        }
        await Assert.ThrowsAsync<UpdateException>(()=>UpdateArchive.ExtractAsync(archive,"zip",Path.Combine(f.Root,"extract"),default));
        Assert.False(Directory.Exists(Path.Combine(f.Root,"outside")));
    }
    [Fact]
    public async Task CancellationPreservesInstalledFilesAndDoesNotPublishStagedState()
    {
        using var f = new UpdateFixture();
        using var transport=new UpdateTransport(f.Distribution,f.Store,f.Environment,f.Handler,f.Time);
        var plan=UpdatePlanner.Select(f.Verified(),f.Installed,f.Environment,true)!;
        var stager=new UpdateStager(transport,f.Store,new StubPlatformVerifier());
        using var cancellation=new CancellationTokenSource();cancellation.Cancel();
        await Assert.ThrowsAnyAsync<OperationCanceledException>(()=>stager.StageAsync(plan,null,null,cancellation.Token));
        Assert.Null(f.Store.Read().Staged);Assert.True(UpdatePlanner.Healthy(f.Installed.Sdk));
    }
}
