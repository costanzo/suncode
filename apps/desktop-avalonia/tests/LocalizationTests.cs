using Avalonia;
using SunCode.Desktop.Infrastructure;
using System.Xml.Linq;

namespace SunCode.Desktop.Tests;

public sealed class LocalizationTests
{
    [Fact]
    public void SupportedLocalesUseStableCodesAndNativeNames()
    {
        Assert.Equal(
            ["en-US", "zh-CN"],
            LocalizationService.SupportedLocales.Select(item => item.Code).ToArray());
        Assert.Equal("English", LocalizationService.SupportedLocales[0].DisplayName);
        Assert.Equal("简体中文", LocalizationService.SupportedLocales[1].DisplayName);
    }

    [Fact]
    public void InvalidLocaleFallsBackToEnglish()
    {
        var service = new LocalizationService(new Application());

        Assert.Equal(LocalizationService.DefaultLocale, service.Normalize("fr-FR"));
        Assert.Equal(LocalizationService.DefaultLocale, service.Normalize(null));
        Assert.Equal(LocalizationService.SimplifiedChineseLocale, service.Normalize("ZH-cn"));
    }

    [Fact]
    public void BundledDictionariesHaveMatchingKeys()
    {
        var projectRoot = new DirectoryInfo(AppContext.BaseDirectory);
        while (projectRoot is not null && !Directory.Exists(Path.Combine(projectRoot.FullName, "apps", "desktop-avalonia", "Resources", "Localization")))
        {
            projectRoot = projectRoot.Parent;
        }

        Assert.NotNull(projectRoot);
        var root = Path.Combine(projectRoot!.FullName, "apps", "desktop-avalonia", "Resources", "Localization");
        var english = ReadKeys(Path.Combine(root, "Strings.en-US.axaml"));
        var chinese = ReadKeys(Path.Combine(root, "Strings.zh-CN.axaml"));

        Assert.Equal(english.OrderBy(key => key), chinese.OrderBy(key => key));
    }

    private static IReadOnlyList<string> ReadKeys(string path) =>
        XDocument.Load(path)
            .Descendants()
            .Attributes(XName.Get("Key", "http://schemas.microsoft.com/winfx/2006/xaml"))
            .Select(attribute => attribute.Value)
            .ToArray();
}
