using Avalonia;
using Avalonia.Media.Fonts;

namespace SunCode.Desktop.Infrastructure;

internal static class BundledFonts
{
    private static readonly Uri CollectionKey = new("fonts:SunCode", UriKind.Absolute);
    private static readonly Uri ResourceRoot = new("avares://SunCode/Assets/fonts", UriKind.Absolute);

    internal static AppBuilder UseSunCodeFonts(this AppBuilder builder) =>
        builder.ConfigureFonts(fontManager =>
        {
            fontManager.AddFontCollection(new EmbeddedFontCollection(CollectionKey, ResourceRoot));
        });
}
