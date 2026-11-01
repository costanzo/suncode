using System.Diagnostics;
using SunCode.Desktop.Infrastructure.Updates;
using SunCode.Desktop.Models;

namespace SunCode.Desktop.Tests.Updates;

public sealed class UpdateBootstrapTests
{
    [Theory]
    [InlineData(true)]
    [InlineData(false)]
    public async Task RealHostPreflightSelectsNativeRuntimeBeforeAnyAgentDatabaseStartup(bool validLibrary)
    {
        using var f = new UpdateFixture();
        var application = Path.GetDirectoryName(typeof(AppInfo).Assembly.Location)!;
        var source = Path.Combine(application, UpdateStager.SdkLibraryName);
        Assert.True(File.Exists(source));
        var sdk = Path.Combine(f.Root, "isolated-native-runtime");
        Directory.CreateDirectory(sdk);
        if (validLibrary) File.Copy(source, Path.Combine(sdk, UpdateStager.SdkLibraryName));
        else File.WriteAllText(Path.Combine(sdk, UpdateStager.SdkLibraryName), "not a native runtime");
        var candidate = new UpdateInstallation(UpdateEnvironment.Current().Target,
            new("0.1.0",27,new string('a',64),application,UpdateArchive.Manifest(application)),
            new("0.1.0",27,new string('b',64),sdk,UpdateArchive.Manifest(sdk)));
        var store = new UpdateStateStore(Path.Combine(f.Root,"desktop-updates"));
        var staged = new StagedUpdate(f.Sign(),false,true,true,Path.Combine(f.Root,"staging"),candidate);
        using var current=Process.GetCurrentProcess();
        var id=Guid.NewGuid().ToString("N");
        var journal=new ActivationJournal(id,"preflight",staged,candidate,current.Id,current.StartTime.ToUniversalTime().Ticks,current.Id,current.StartTime.ToUniversalTime().Ticks);
        store.Change(s=> {s.Installation=f.Installed;s.Activation=journal;});
        var start = new ProcessStartInfo("dotnet") { UseShellExecute=false,RedirectStandardOutput=true,RedirectStandardError=true,WorkingDirectory=f.Root };
        start.ArgumentList.Add(typeof(AppInfo).Assembly.Location);
        start.ArgumentList.Add("--suncode-update-preflight"); start.ArgumentList.Add(id);
        start.Environment["SUNCODE_DATA_DIRECTORY"]=f.Root;
        using var child=Process.Start(start)!;
        var stdout=child.StandardOutput.ReadToEndAsync();var stderr=child.StandardError.ReadToEndAsync();
        try
        {
            var signal=Path.Combine(store.DirectoryPath,"signals",id,validLibrary?"preflight-ready":"preflight-failed");
            var deadline=DateTime.UtcNow.AddSeconds(15);
            while (!File.Exists(signal)&&!child.HasExited&&DateTime.UtcNow<deadline) await Task.Delay(25);
            Assert.True(File.Exists(signal),$"Preflight signal missing; exited={child.HasExited}");
            Assert.Equal("0.1.0",store.Read().Installation!.Sdk!.Version);
            Assert.Equal(f.Installed.Sdk!.Directory,store.Read().Installation!.Sdk!.Directory);
            Assert.Empty(Directory.GetFiles(f.Root,"*.sqlite*",SearchOption.AllDirectories));
            if (validLibrary)
            {
                Assert.False(child.HasExited); // It cannot start Avalonia/Agent until the helper durably commits.
                Assert.False(File.Exists(Path.Combine(store.DirectoryPath,"signals",id,"commit-ready")));
            }
            else
            {
                await child.WaitForExitAsync().WaitAsync(TimeSpan.FromSeconds(5));
                Assert.Equal(1,child.ExitCode);
            }
        }
        finally
        {
            if (!child.HasExited) child.Kill(entireProcessTree:true);
            await child.WaitForExitAsync();await stdout;await stderr;
        }
    }
}
