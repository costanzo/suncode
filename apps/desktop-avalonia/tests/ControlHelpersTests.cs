using SunCode.Desktop.Controls;

namespace SunCode.Desktop.Tests;

public sealed class ControlHelpersTests
{
    [Fact]
    public void FileSelectorNormalizesExtensionPatterns()
    {
        var result = SCFileSelector.NormalizePatterns("png, .jpg;*.webp png");

        Assert.Equal(["*.png", "*.jpg", "*.webp"], result);
    }

}
