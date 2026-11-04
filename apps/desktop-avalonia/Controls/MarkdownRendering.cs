using Avalonia.Controls;
using Avalonia.Controls.Documents;
using LiveMarkdown.Avalonia;
using Markdig;
using Markdig.Extensions.Mathematics;
using RaTeX;

namespace SunCode.Desktop.Controls;

/// <summary>Registers presentation-only extensions before the shared parsing pipeline is built.</summary>
internal static class MarkdownRendering
{
    private static readonly Lazy<bool> Initialization = new(Configure);

    internal static bool MathAvailable => Initialization.Value;

    internal static void Initialize() => _ = Initialization.Value;

    private static bool Configure()
    {
        MarkdownRenderer.ConfigurePipeline += pipeline => pipeline.UseExtendedMathematics().UseMermaid();
        var mathAvailable = CanLoadMath();
        if (mathAvailable)
            MarkdownNode.Edit(nodes => nodes.Register<MathInlineNode>().Register<ScrollableMathBlockNode>());
        else
            MarkdownNode.Edit(nodes => nodes.Register<SourceMathInlineNode>().Register<SourceMathBlockNode>());
        MarkdownNode.Register<DesktopMermaidBlockNode>();
        AsyncImageLoader.DefaultDecoders = [SvgImageDecoder.Shared, DefaultBitmapDecoder.Shared];
        return mathAvailable;
    }

    private static bool CanLoadMath()
    {
        try
        {
            return Ratex.TryParseAndLayout("x", null, out _, out _);
        }
        catch (Exception exception) when (exception is DllNotFoundException or EntryPointNotFoundException or BadImageFormatException)
        {
            // RaTeX 1.0.0 has no Windows ARM64 asset. Keep these messages readable
            // there, and when an installed native library is missing/incompatible.
            return false;
        }
    }
}

internal sealed class ScrollableMathBlockNode : BlockNode<MathBlock>
{
    private readonly MathBlockNode _node = new();
    public override Control Control { get; }

    public ScrollableMathBlockNode()
    {
        Control = new ScrollViewer
        {
            HorizontalScrollBarVisibility = Avalonia.Controls.Primitives.ScrollBarVisibility.Auto,
            VerticalScrollBarVisibility = Avalonia.Controls.Primitives.ScrollBarVisibility.Disabled,
            Content = _node.Control
        };
    }

    protected override bool UpdateCore(DocumentNode documentNode, MathBlock math,
        in ObservableStringBuilderChangedEventArgs change, CancellationToken cancellationToken) =>
        _node.Update(documentNode, math, change, cancellationToken) != false;
}

internal sealed class SourceMathInlineNode : InlineNode<MathInline>
{
    private readonly Run _source = new();
    public override Inline Inline => _source;

    public SourceMathInlineNode() { }

    protected override bool UpdateCore(DocumentNode documentNode, MathInline math,
        in ObservableStringBuilderChangedEventArgs change, CancellationToken cancellationToken)
    {
        _source.Text = math.Content.ToString();
        return true;
    }
}

internal sealed class SourceMathBlockNode : BlockNode<MathBlock>
{
    private readonly MarkdownTextBlock _source = new() { Classes = { "MathBlock" } };
    public override Control Control => _source;

    public SourceMathBlockNode() { }

    protected override bool UpdateCore(DocumentNode documentNode, MathBlock math,
        in ObservableStringBuilderChangedEventArgs change, CancellationToken cancellationToken)
    {
        _source.Text = math.Lines.ToString();
        return true;
    }
}
