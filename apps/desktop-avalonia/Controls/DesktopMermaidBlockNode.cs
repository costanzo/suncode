using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.Markup.Xaml.MarkupExtensions;
using LiveMarkdown.Avalonia;

namespace SunCode.Desktop.Controls;

/// <summary>Keeps diagram source selectable and copyable, including unsupported diagrams.</summary>
internal sealed class DesktopMermaidBlockNode : BlockNode<MermaidCodeBlock>
{
    private readonly MermaidBlockNode _node = new();
    private readonly CodeBlock _source = new() { AutoSyntaxHighlight = false, Classes = { "MermaidSource" } };
    private readonly Expander _sourceExpander;
    private readonly Control _diagram;
    private bool _wasOpen = true;
    public override Control Control { get; }

    public DesktopMermaidBlockNode()
    {
        _diagram = _node.Control;
        _sourceExpander = new Expander
        {
            Classes = { "MermaidSource" },
            IsExpanded = true,
            HorizontalAlignment = Avalonia.Layout.HorizontalAlignment.Stretch,
            HorizontalContentAlignment = Avalonia.Layout.HorizontalAlignment.Stretch,
            Content = _source
        };
        _sourceExpander[!HeaderedContentControl.HeaderProperty] = new DynamicResourceExtension("LocMarkdownDiagramSource");
        Control = new StackPanel { Classes = { "DesktopMermaidBlock" }, Children = { _diagram, _sourceExpander } };
    }

    protected override bool UpdateCore(DocumentNode documentNode, MermaidCodeBlock mermaid,
        in ObservableStringBuilderChangedEventArgs change, CancellationToken cancellationToken)
    {
        var result = _node.Update(documentNode, mermaid, change, cancellationToken) != false;
        _source.Code = mermaid.Lines.ToString();
        if (_wasOpen != mermaid.IsOpen)
            _sourceExpander.IsExpanded = mermaid.IsOpen;
        _wasOpen = mermaid.IsOpen;
        _diagram.IsVisible = !mermaid.IsOpen;
        return result;
    }
}
