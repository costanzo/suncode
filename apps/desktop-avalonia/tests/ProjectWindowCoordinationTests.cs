namespace SunCode.Desktop.Tests;

public sealed class ProjectWindowCoordinationTests
{
    [Theory]
    [InlineData(true, false, (int)ProjectWindowDisposition.ActivateExisting)]
    [InlineData(true, true, (int)ProjectWindowDisposition.ActivateExisting)]
    [InlineData(false, true, (int)ProjectWindowDisposition.AwaitOpening)]
    [InlineData(false, false, (int)ProjectWindowDisposition.OpenNew)]
    public void ProjectSelectionChoosesTheExpectedWindowAction(
        bool isOpen,
        bool isOpening,
        int expected)
    {
        Assert.Equal((ProjectWindowDisposition)expected, App.ResolveProjectWindowDisposition(isOpen, isOpening));
    }
}
