using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Documents;
using Avalonia.Interactivity;
using Avalonia.Input.Platform;
using Avalonia.LogicalTree;
using Avalonia.Markup.Xaml.Styling;
using Avalonia.Media;
using Avalonia.Styling;
using Avalonia.Themes.Fluent;
using Avalonia.VisualTree;
using LiveMarkdown.Avalonia;
using Markdig;
using Markdig.Extensions.Mathematics;
using Markdig.Syntax;
using CodeBlock = LiveMarkdown.Avalonia.CodeBlock;
using RaTeX.Avalonia;
using SunCode.Desktop.Controls;

namespace SunCode.Desktop.Tests;

[Collection("Avalonia UI")]
public sealed class MarkdownRenderingTests(AvaloniaUiFixture ui)
{
    [Theory]
    [InlineData("Inline $x^2$.")]
    [InlineData(@"Inline \(x^2\).")]
    [InlineData("$$\nx^2\n$$")]
    [InlineData("\\[\nx^2\n\\]")]
    [InlineData("$$\n\\begin{pmatrix}1&2\\\\3&4\\end{pmatrix}\n$$")]
    public Task MathDelimitersProduceNativeFormulas(string source) => ui.Run(async () =>
    {
        using var surface = new Surface();
        await surface.Set(source);
        Assert.True(MarkdownRendering.MathAvailable);
        var formula = Assert.Single(surface.Descendants<FormulaControl>());
        Assert.True(formula.IsVisible, string.Join("; ", surface.Descendants<MarkdownTextBlock>().Select(block => ToolTip.GetTip(block)).OfType<string>()));
        Assert.NotNull(formula.DisplayList);
        Assert.True(formula.DesiredSize.Width > 0);
        Assert.Equal(source, surface.Text.Markdown);
    });

    [Theory]
    [InlineData("flowchart LR\n A[Start] --> B[Finish]")]
    [InlineData("sequenceDiagram\n Alice->>Bob: Hello")]
    [InlineData("classDiagram\n Animal <|-- Duck")]
    [InlineData("stateDiagram-v2\n [*] --> Ready\n Ready --> [*]")]
    [InlineData("erDiagram\n CUSTOMER ||--o{ ORDER : places")]
    public Task CommonDiagramsKeepCopyableSource(string diagram) => ui.Run(async () =>
    {
        using var surface = new Surface();
        await surface.Set($"```mermaid\n{diagram}\n```");
        var presenter = Assert.Single(surface.Descendants<MermaidPresenter>());
        Assert.Equal(diagram, presenter.Text);
        Assert.True(presenter.Bounds.Width > 0);
        Assert.True(presenter.Bounds.Height > 0);
        Assert.Single(presenter.GetLogicalChildren().OfType<MermaidRenderer>());
        var disclosure = Assert.Single(surface.Descendants<Expander>());
        Assert.False(disclosure.IsExpanded);
        disclosure.IsExpanded = true;
        surface.Window.UpdateLayout();
        var source = Assert.Single(surface.Descendants<CodeBlock>());
        Assert.Equal(diagram, source.Code?.ReplaceLineEndings("\n"));
        Assert.True(source.IsEffectivelyVisible);
        Assert.NotNull(source.GetVisualDescendants().OfType<Button>().SingleOrDefault(button => button.Name == "PART_CopyButton"));
    });

    [Fact]
    public Task StreamingFenceShowsSourceThenDiagramAndRecoversAfterReplacement() => ui.Run(async () =>
    {
        using var surface = new Surface();
        const string open = "```mermaid\nflowchart LR\n A --> B";
        await surface.Set(open);
        Assert.True(Assert.Single(surface.Descendants<Expander>()).IsExpanded);
        Assert.DoesNotContain(surface.Descendants<MermaidPresenter>(), presenter => presenter.IsEffectivelyVisible);
        await surface.Set(open + "\n```");
        Assert.False(Assert.Single(surface.Descendants<Expander>()).IsExpanded);
        Assert.True(Assert.Single(surface.Descendants<MermaidPresenter>()).IsEffectivelyVisible);
        await surface.Set("```mermaid\nunsupportedDiagram\n A --> B\n```");
        var disclosure = Assert.Single(surface.Descendants<Expander>());
        disclosure.IsExpanded = true;
        surface.Window.UpdateLayout();
        Assert.Contains("unsupportedDiagram", Assert.Single(surface.Descendants<CodeBlock>()).Code);
        Assert.Empty(Assert.Single(surface.Descendants<MermaidPresenter>()).GetLogicalChildren().OfType<MermaidRenderer>());
        await surface.Set("```mermaid\nunsupportedDiagram\n A --> B\n```\n\nTail");
        Assert.True(Assert.Single(surface.Descendants<Expander>()).IsExpanded);
        await surface.Set("Replacement $x^2$.");
        Assert.Empty(surface.Descendants<MermaidPresenter>());
        Assert.Single(surface.Descendants<FormulaControl>());
        await surface.Set(null);
        Assert.Empty(surface.Descendants<FormulaControl>());
        Assert.Empty(surface.Descendants<MermaidPresenter>());
    });

    [Fact]
    public Task DiagramSourceCopiesThroughTheExistingCodeTemplate() => ui.Run(async () =>
    {
        using var surface = new Surface();
        await surface.Set("```mermaid\nflowchart LR\n A --> B\n```");
        Assert.Single(surface.Descendants<Expander>()).IsExpanded = true;
        surface.Window.UpdateLayout();
        var code = Assert.Single(surface.Descendants<CodeBlock>());
        var button = Assert.Single(code.GetVisualDescendants().OfType<Button>(), item => item.Name == "PART_CopyButton");
        button.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
        var copied = await surface.Window.Clipboard!.TryGetTextAsync();
        Assert.Equal("flowchart LR\n A --> B", copied?.ReplaceLineEndings("\n"));
    });

    [Fact]
    public Task OpenMathBlockWaitsForItsClosingDelimiter() => ui.Run(async () =>
    {
        using var surface = new Surface(width: 320);
        const string open = "$$\nx_1+x_2+x_3+x_4+x_5+x_6+x_7+x_8+x_9+x_{10}+x_{11}+x_{12}";
        await surface.Set(open);
        Assert.False(Assert.Single(surface.Descendants<FormulaControl>()).IsVisible);
        Assert.Contains("x_1", Assert.Single(surface.Descendants<MarkdownTextBlock>(), block => block.Classes.Contains("MathBlock")).Text);
        await surface.Set(open + "\n$$");
        Assert.True(Assert.Single(surface.Descendants<FormulaControl>()).IsVisible);
        Assert.Contains(surface.Descendants<ScrollViewer>(), scroll => scroll.Extent.Width > scroll.Viewport.Width);
        Assert.True(surface.Text.Bounds.Width <= 320);
    });

    [Fact]
    public Task MissingGlyphsKeepReadableFormulaSource() => ui.Run(async () =>
    {
        using var surface = new Surface();
        await surface.Set(@"Inline $\text{中文}+x$.");
        var formula = Assert.Single(surface.Descendants<FormulaControl>());
        if (formula.IsVisible)
            Assert.NotNull(formula.DisplayList);
        else
        {
            var fallback = Assert.Single(surface.Descendants<MarkdownTextBlock>(), block => block.Classes.Contains("Error"));
            Assert.Contains("中文", fallback.Text);
            Assert.NotNull(ToolTip.GetTip(fallback));
        }
    });

    [Fact]
    public Task InvalidFormulaRetainsSourceAndNextUpdateRecovers() => ui.Run(async () =>
    {
        using var surface = new Surface();
        await surface.Set("$$\n\\frac{1}{\n$$");
        var error = Assert.Single(surface.Descendants<MarkdownTextBlock>(), block => block.Classes.Contains("Error"));
        Assert.Contains(@"\frac{1}{", error.Text);
        Assert.NotNull(ToolTip.GetTip(error));
        await surface.Set("$$\n\\frac{1}{2}\n$$");
        Assert.DoesNotContain(surface.Descendants<MarkdownTextBlock>(), block => block.Classes.Contains("Error"));
        Assert.True(Assert.Single(surface.Descendants<FormulaControl>()).IsVisible);
    });

    [Theory]
    [InlineData(false, false, 14)]
    [InlineData(true, false, 14)]
    [InlineData(false, true, 12)]
    [InlineData(true, true, 12)]
    public Task ThemeAndSecondarySizingFollowTheReadingSurface(bool dark, bool secondary, double expectedSize) => ui.Run(async () =>
    {
        using var surface = new Surface(dark, width: 320);
        surface.Text.IsSecondary = secondary;
        await surface.Set("Inline $x^2$ and `code`.\n\n```mermaid\nflowchart LR\n A --> B\n```");
        var formula = Assert.Single(surface.Descendants<FormulaControl>());
        Assert.Equal(expectedSize, formula.FontSize);
        var renderer = (MarkdownRenderer)surface.Text.Content!;
        Assert.Equal(dark ? MarkdownSyntaxThemes.DarkName : MarkdownSyntaxThemes.LightName, renderer.CodeBlockCustomColorTheme);
        var pan = Assert.Single(surface.Descendants<PanAndZoom>());
        Assert.Equal(480, pan.MaxHeight);
        Assert.True(pan.Bounds.Width <= 320);
        surface.Window.RequestedThemeVariant = dark ? ThemeVariant.Light : ThemeVariant.Dark;
        surface.Window.UpdateLayout();
        Assert.Equal(dark ? MarkdownSyntaxThemes.LightName : MarkdownSyntaxThemes.DarkName, renderer.CodeBlockCustomColorTheme);
    });

    [Fact]
    public Task NormalMarkdownTemplateSelectionAndReattachmentStillWork() => ui.Run(async () =>
    {
        using var surface = new Surface();
        await surface.Set("# Heading\n\nBody with **bold** and [link](https://example.com).\n\n```csharp\nvar x = 1;\n```");
        var code = Assert.Single(surface.Descendants<CodeBlock>());
        Assert.Equal("var x = 1;", code.Code);
        Assert.Equal("PART_CodeTextBlock", code.CodeTextBlock?.Name);
        var renderer = (MarkdownRenderer)surface.Text.Content!;
        renderer.SelectAll();
        Assert.True(renderer.CanCopy);
        Assert.Contains("Heading", renderer.SelectedText);
        Assert.Contains("var x = 1;", renderer.SelectedText);
        surface.Window.Content = null;
        surface.Window.Content = surface.Text;
        await surface.Set("Reattached $y^2$.");
        Assert.Single(surface.Descendants<FormulaControl>());
        Assert.False(renderer.CanCopy);
    });

    [Fact]
    public Task SourceMathNodesRemainReadableWithoutNativeRendering() => ui.Run(() =>
    {
        MarkdownRendering.Initialize();
        var renderer = new MarkdownRenderer();
        var documentNode = new DocumentNode(renderer);
        var document = Markdown.Parse("Inline $x^2$.\n\n$$\nx+y\n$$", MarkdownUpdateProducer.DefaultPipeline);
        var mathInline = document.Descendants<MathInline>().Single();
        var inline = new SourceMathInlineNode();
        var change = new ObservableStringBuilderChangedEventArgs(0, 100, 100, 1);
        inline.Update(documentNode, mathInline, change, CancellationToken.None);
        Assert.Equal("x^2", ((Run)inline.Inline).Text);
        var block = new SourceMathBlockNode();
        block.Update(documentNode, document.Descendants<MathBlock>().Single(), change, CancellationToken.None);
        Assert.Equal("x+y", ((MarkdownTextBlock)block.Control).Text);
    });

    private sealed class Surface : IDisposable
    {
        private readonly Application _application = Application.Current!;
        private readonly FluentTheme _theme = new();
        private readonly StyleInclude _styles = new(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/MarkdownStyles.axaml") };
        private readonly ResourceInclude _resources = new(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/ThemeResources.axaml") };
        public MarkdownText Text { get; } = new();
        public Window Window { get; }

        public Surface(bool dark = false, double width = 700)
        {
            _application.Styles.Add(_theme);
            _application.Resources.MergedDictionaries.Add(_resources);
            _application.Styles.Add(_styles);
            Window = new Window { Width = width, Height = 700, Content = Text, RequestedThemeVariant = dark ? ThemeVariant.Dark : ThemeVariant.Light };
            Window.Show();
        }

        public IEnumerable<T> Descendants<T>() where T : Visual => Text.GetVisualDescendants().OfType<T>();

        public async Task Set(string? source)
        {
            Text.Markdown = source;
            var renderer = (MarkdownRenderer)Text.Content!;
            var deadline = DateTime.UtcNow.AddSeconds(5);
            while (renderer.RenderedTextProjection?.SourceVersion != renderer.MarkdownBuilder!.Version)
            {
                Assert.True(DateTime.UtcNow < deadline, "Markdown update did not finish.");
                await Task.Delay(10);
                Window.UpdateLayout();
            }
            Window.UpdateLayout();
        }

        public void Dispose()
        {
            Window.Close();
            _application.Styles.Remove(_styles);
            _application.Styles.Remove(_theme);
            _application.Resources.MergedDictionaries.Remove(_resources);
        }
    }
}
