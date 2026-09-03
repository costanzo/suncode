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

    [Fact]
    public void ProviderCatalogFallsBackForUnknownProvider()
    {
        var result = SCProviderCatalog.GetOrDefault("custom");

        Assert.Equal("custom", result.Title);
        Assert.Equal("Paste API key", result.ApiKeyPlaceholder);
        Assert.Equal(string.Empty, result.DefaultEndpoint);
    }

    [Fact]
    public void ProviderCatalogExposesTheSeededDefaultEndpoint()
    {
        var result = SCProviderCatalog.GetOrDefault("openai");

        Assert.Equal("https://api.openai.com/v1", result.DefaultEndpoint);
    }

}
