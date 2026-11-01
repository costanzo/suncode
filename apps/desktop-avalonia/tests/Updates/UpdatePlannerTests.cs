using SunCode.Desktop.Infrastructure.Updates;

namespace SunCode.Desktop.Tests.Updates;

public sealed class UpdatePlannerTests
{
    [Fact]
    public void DownloadsOnlyChangedSdkAndRepairsMissingOrDamagedSameVersion()
    {
        using var f = new UpdateFixture();
        var plan = UpdatePlanner.Select(f.Verified(), f.Installed, f.Environment, true)!;
        Assert.False(plan.Desktop); Assert.True(plan.Sdk); Assert.False(plan.Repair);
        Assert.Equal(f.Catalog.Components.Sdk.Package.SizeBytes, plan.SizeBytes);
        Assert.True(UpdatePlanner.Select(f.Verified(), f.Installed with { Sdk = null }, f.Environment, false)!.Repair);
        var updated = f.Installed with { Sdk = f.Installed.Sdk! with { Version = "0.1.1", ArchiveSha256 = f.Catalog.Components.Sdk.Package.Sha256 } };
        Assert.Null(UpdatePlanner.Select(f.Verified(), updated, f.Environment, true));
        Assert.True(UpdatePlanner.Select(f.Verified(), updated, f.Environment, false)!.Repair);
    }
    [Fact]
    public void DesktopOnlyAndRequiredAbiPairRemainIndependentPlans()
    {
        using var f = new UpdateFixture();
        f.Catalog = f.Catalog with { Components = f.Catalog.Components with {
            Desktop = f.Catalog.Components.Desktop with { Version = "0.1.1", Package = f.Catalog.Components.Desktop.Package with { Sha256 = new string('c',64) } },
            Sdk = f.Catalog.Components.Sdk with { Version = "0.1.0", Package = f.Catalog.Components.Sdk.Package with { Sha256 = f.Installed.Sdk!.ArchiveSha256 } }
        }, CompatiblePairs = [new() { Desktop = "0.1.1", Sdk = "0.1.0" }] };
        var desktopOnly = UpdatePlanner.Select(f.Verified(), f.Installed, f.Environment, true)!;
        Assert.True(desktopOnly.Desktop); Assert.False(desktopOnly.Sdk);
        f.Catalog = f.Catalog with { Components = f.Catalog.Components with {
            Desktop = f.Catalog.Components.Desktop with { Version = "0.2.0", RequiredSdkAbi = 28 },
            Sdk = f.Catalog.Components.Sdk with { Version = "0.2.0", Abi = 28 }
        }, CompatiblePairs = [new() { Desktop = "0.2.0", Sdk = "0.2.0" }] };
        var paired = UpdatePlanner.Select(f.Verified(), f.Installed, f.Environment, true)!;
        Assert.True(paired.Desktop && paired.Sdk);
        Assert.Equal(f.Catalog.Components.Desktop.Package.SizeBytes + f.Catalog.Components.Sdk.Package.SizeBytes, paired.SizeBytes);
    }
    [Fact]
    public void DoesNotInferCompatibilityDowngradeOrAcceptReusedReleaseIdentity()
    {
        using var f = new UpdateFixture();
        var future = f.Installed with { Desktop = f.Installed.Desktop with { Version = "9.0.0" } };
        Assert.Equal("no_compatible_update", Assert.Throws<UpdateException>(() => UpdatePlanner.Select(f.Verified(), future, f.Environment, true)).Code);
        var changedIdentity = f.Installed with { Desktop = f.Installed.Desktop with { ArchiveSha256 = new string('e',64) } };
        Assert.Equal("release_conflict", Assert.Throws<UpdateException>(() => UpdatePlanner.Select(f.Verified(), changedIdentity, f.Environment, true)).Code);
        f.Catalog = f.Catalog with { Components = f.Catalog.Components with { Sdk = f.Catalog.Components.Sdk with { Package = f.Catalog.Components.Sdk.Package with { MinimumOsVersion = "11.0" } } } };
        Assert.Throws<UpdateException>(() => UpdatePlanner.Select(f.Verified(), f.Installed, f.Environment, true));
        Assert.False(UpdatePlanner.Supports(f.Catalog.Components.Sdk.Package with { MinimumOsVersion = null, MinimumGlibcVersion = "2.35" }, new("linux-x64","6.0",null)));
    }
    [Fact]
    public void NumericVersionsAndInstalledFileIntegrityAreChecked()
    {
        using var f = new UpdateFixture();
        Assert.True(ReleaseVersion.Parse("0.10.0").CompareTo(ReleaseVersion.Parse("0.2.0")) > 0);
        Assert.True(UpdatePlanner.Healthy(f.Installed.Sdk));
        File.WriteAllText(Path.Combine(f.Installed.Sdk!.Directory, UpdateStager.SdkLibraryName), "tampered");
        Assert.False(UpdatePlanner.Healthy(f.Installed.Sdk));
    }
}
