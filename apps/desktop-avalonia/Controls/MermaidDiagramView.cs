using Avalonia;
using Avalonia.Controls;
using Avalonia.Media;
using LiveMarkdown.Avalonia;
using Mermaider.Models;
using SvgControl = Avalonia.Svg.Skia.Svg;

namespace SunCode.Desktop.Controls;

/// <summary>Uses Mermaider's public SVG API so chart support is not limited by LiveMarkdown's native renderer.</summary>
internal sealed class MermaidDiagramView : ContentControl
{
    private readonly SvgControl _image = new(new Uri("avares://SunCode/")) { EnableCache = false, Stretch = Stretch.None };
    private readonly MarkdownTextBlock _error = new() { Classes = { "MermaidError" }, IsVisible = false };
    private readonly PanAndZoom _viewport;
    private string? _text;
    private Avalonia.Svg.Skia.SvgSource? _svgSource;

    internal string? Text
    {
        get => _text;
        set
        {
            if (_text == value) return;
            _text = value;
            Rebuild();
        }
    }

    internal string? Error { get; private set; }
    internal string? RenderedSvg { get; private set; }

    public MermaidDiagramView()
    {
        _viewport = new PanAndZoom { Classes = { "MermaidBlock" }, Content = _image };
        Content = new StackPanel { Children = { _viewport, _error } };
        ActualThemeVariantChanged += (_, _) => Rebuild();
        AttachedToVisualTree += (_, _) => Rebuild();
        DetachedFromVisualTree += (_, _) =>
        {
            _image.SvgSource = null;
            _svgSource?.Dispose();
            _svgSource = null;
            RenderedSvg = null;
        };
    }

    private string Color(string key)
    {
        Application.Current!.TryGetResource(key, ActualThemeVariant, out var resource);
        if (resource is not ISolidColorBrush brush) throw new InvalidOperationException($"Missing diagram resource: {key}");
        return $"#{brush.Color.R:X2}{brush.Color.G:X2}{brush.Color.B:X2}";
    }

    private void Rebuild()
    {
        if (VisualRoot is null || string.IsNullOrWhiteSpace(_text)) return;
        try
        {
            var svg = Mermaider.MermaidRenderer.RenderSvg(_text, new Mermaider.Models.RenderOptions
            {
                Bg = Color("SurfaceBrush"), Fg = Color("TextBrush"),
                Muted = Color("TextMutedBrush"), Accent = Color("AccentBrush"),
                Surface = Color("FieldBrush"), Border = Color("BorderStrongBrush"), Line = Color("BorderStrongBrush"),
                Default = Color("AccentBrush"), Success = Color("AccentBrush"),
                Failure = Color("DangerBrush"), Warning = Color("WarningBrush"), Info = Color("AccentBrush"),
                DataPalette = [Color("AccentBrush"), Color("TextSecondaryBrush"), Color("TextMutedBrush")],
                Font = "Noto Sans", MonoFont = "JetBrains Mono", FontSize = "14px",
                Gradient = false, Elevation = 0,
                Limits = ResourceLimits.Default with { RenderDeadline = TimeSpan.FromSeconds(2) }
            });
            RenderedSvg = MermaidSvg.Materialize(svg);
            var source = Avalonia.Svg.Skia.SvgSource.LoadFromSvg(RenderedSvg);
            if (source.Picture is null)
            {
                source.Dispose();
                throw new InvalidOperationException("The diagram produced no SVG picture.");
            }
            var previous = _svgSource;
            _image.SvgSource = _svgSource = source;
            _image.Width = source.Picture.CullRect.Width;
            _image.Height = source.Picture.CullRect.Height;
            previous?.Dispose();
            Error = null;
            _image.IsVisible = true;
            _viewport.IsVisible = true;
            _error.IsVisible = false;
        }
        catch (Exception exception)
        {
            RenderedSvg = null;
            Error = exception.Message;
            _image.SvgSource = null;
            _svgSource?.Dispose();
            _svgSource = null;
            _image.IsVisible = false;
            _viewport.IsVisible = false;
            _error.Text = exception.Message;
            _error.IsVisible = true;
        }
    }
}
