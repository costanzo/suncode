using System.Diagnostics;
using SunCode.Desktop.Infrastructure.Updates;

namespace SunCode.Desktop.Tests.Updates;

public sealed class UpdateActivationTests
{
    [Fact]
    public async Task StagingAndPreparationNeverChangeTheActiveVersionsAndReextractAuthenticatedBytes()
    {
        using var f = new UpdateFixture();
        using var transport = new UpdateTransport(f.Distribution,f.Store,f.Environment,f.Handler,f.Time);
        var platform = new StubPlatformVerifier();
        var plan = UpdatePlanner.Select(f.Verified(),f.Installed,f.Environment,true)!;
        var staged = await new UpdateStager(transport,f.Store,platform).StageAsync(plan,null,null,default);
        Assert.Equal("0.1.0",f.Store.Read().Installation!.Sdk!.Version);
        File.WriteAllText(Path.Combine(staged.Directory,"sdk",UpdateStager.SdkLibraryName),"tampered after extraction");
        var activation = new UpdateActivation(f.Distribution,f.Store,f.Environment,platform,f.Time);
        var journal=await activation.PrepareAsync(default);
        Assert.Equal("0.1.0",f.Store.Read().Installation!.Sdk!.Version);
        Assert.Equal("0.1.1",journal.Candidate.Sdk!.Version);
        Assert.Equal("new-sdk",File.ReadAllText(Path.Combine(journal.Candidate.Sdk.Directory,UpdateStager.SdkLibraryName)));
        Assert.True(UpdatePlanner.Healthy(f.Installed.Sdk));
        Assert.Equal(2,platform.Checks);
    }
    [Fact]
    public async Task PreflightGatesStartupUntilAtomicCommitAndInterruptedRecoveryKeepsPreviousPointer()
    {
        using var f = new UpdateFixture();
        using var transport = new UpdateTransport(f.Distribution,f.Store,f.Environment,f.Handler,f.Time);
        var platform=new StubPlatformVerifier();
        await new UpdateStager(transport,f.Store,platform).StageAsync(UpdatePlanner.Select(f.Verified(),f.Installed,f.Environment,true)!,null,null,default);
        var activation = new UpdateActivation(f.Distribution,f.Store,f.Environment,platform,f.Time);
        var journal=await activation.PrepareAsync(default);
        activation.RecoverInterrupted();
        Assert.Null(f.Store.Read().Activation);
        Assert.Equal("0.1.0",f.Store.Read().Installation!.Sdk!.Version);
        Assert.NotNull(f.Store.Read().Staged);
        Assert.NotEmpty(f.Store.Read().Catalogs);
        using var process=Process.GetCurrentProcess();
        journal=journal with { Phase="preflight",HelperPid=process.Id,HelperStarted=process.StartTime.ToUniversalTime().Ticks };
        f.Store.Change(s=>s.Activation=journal);
        var identityChecked=false;
        var gate=activation.PreflightAsync(journal.Id,installation=> { identityChecked=true;Assert.Equal("0.1.1",installation.Sdk!.Version);return Task.CompletedTask; },default);
        Assert.True(identityChecked);
        Assert.False(gate.IsCompleted);
        Assert.Equal("0.1.0",f.Store.Read().Installation!.Sdk!.Version);
        f.Store.Change(s=> { s.Installation=journal.Candidate;s.Activation=journal with {Phase="committed"}; });
        Assert.False(gate.IsCompleted);
        UpdateStateStore.AtomicWrite(Path.Combine(f.Store.DirectoryPath,"signals",journal.Id,"commit-ready"),[1]);
        var result=await gate.WaitAsync(TimeSpan.FromSeconds(5));
        Assert.Equal("0.1.1",result.Sdk!.Version);
        activation.RecoverInterrupted();
        Assert.Equal("0.1.1",f.Store.Read().Installation!.Sdk!.Version);
        Assert.Null(f.Store.Read().Activation);
    }
    [Fact]
    public async Task FailedIdentityCannotCommitOrStartAgentState()
    {
        using var f = new UpdateFixture();
        using var transport=new UpdateTransport(f.Distribution,f.Store,f.Environment,f.Handler,f.Time);
        var platform=new StubPlatformVerifier();
        await new UpdateStager(transport,f.Store,platform).StageAsync(UpdatePlanner.Select(f.Verified(),f.Installed,f.Environment,true)!,null,null,default);
        var activation=new UpdateActivation(f.Distribution,f.Store,f.Environment,platform,f.Time);
        var journal=await activation.PrepareAsync(default);
        using var process=Process.GetCurrentProcess();
        journal=journal with {Phase="preflight",HelperPid=process.Id,HelperStarted=process.StartTime.ToUniversalTime().Ticks};
        f.Store.Change(s=>s.Activation=journal);
        await Assert.ThrowsAsync<UpdateException>(()=>activation.PreflightAsync(journal.Id,_=>throw new UpdateException("identity_invalid","wrong release"),default));
        Assert.Equal("0.1.0",f.Store.Read().Installation!.Sdk!.Version);
        Assert.True(File.Exists(Path.Combine(f.Store.DirectoryPath,"signals",journal.Id,"preflight-failed")));
    }
}
