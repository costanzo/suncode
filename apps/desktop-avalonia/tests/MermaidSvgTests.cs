using System.Xml.Linq;
using SunCode.Desktop.Controls;

namespace SunCode.Desktop.Tests;

public sealed class MermaidSvgTests
{
    [Fact]
    public void ThemeExpressionsBecomeStaticWithoutRewritingLabels()
    {
        var result = XDocument.Parse(MermaidSvg.Materialize("""
            <svg xmlns="http://www.w3.org/2000/svg" style="--fg:#000000;--bg:#FFFFFF">
              <style>svg { --fg:#123456; --line:color-mix(in srgb, var(--fg) 50%, var(--bg)); --fs-s:calc(var(--fs-m) * 0.875); }</style>
              <path stroke="var(--line)" fill="var(--missing, var(--bg))" />
              <text font-size="var(--fs-s)" fill="var(--fg)" aria-label="var(--user-label)">var(--user-label)</text>
            </svg>
            """));
        var path = result.Descendants().Single(element => element.Name.LocalName == "path");
        Assert.Equal("#808080", path.Attribute("stroke")!.Value);
        Assert.Equal("#FFFFFF", path.Attribute("fill")!.Value);
        var text = result.Descendants().Single(element => element.Name.LocalName == "text");
        Assert.Equal("12px", text.Attribute("font-size")!.Value);
        Assert.Equal("#000000", text.Attribute("fill")!.Value);
        Assert.Equal("var(--user-label)", text.Attribute("aria-label")!.Value);
        Assert.Equal("var(--user-label)", text.Value);
    }

    [Fact]
    public void RecursiveVariablesFailInsteadOfHangingTheRenderer() => Assert.Throws<FormatException>(() =>
        MermaidSvg.Materialize("""<svg style="--a:var(--b);--b:var(--a)"><path fill="var(--a)" /></svg>"""));

    [Fact]
    public void NestedMixesAndTransparencyKeepTheirColorAndAlpha()
    {
        var result = XDocument.Parse(MermaidSvg.Materialize("""
            <svg><path fill="color-mix(in srgb, color-mix(in srgb, #FFFFFF 50%, #000000) 50%, transparent)" /></svg>
            """));
        Assert.Equal("rgba(128,128,128,0.5)", result.Root!.Element("path")!.Attribute("fill")!.Value);
    }
}
