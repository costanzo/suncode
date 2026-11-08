using System.Globalization;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.Input.Platform;
using SunCode.Desktop.ViewModels;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Media;
using Avalonia.VisualTree;
using AvaloniaEdit;
using AvaloniaEdit.Document;
using AvaloniaEdit.Editing;
using AvaloniaEdit.Rendering;
using AvaloniaEdit.TextMate;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using TextMateSharp.Grammars;

namespace SunCode.Desktop.Controls;

public sealed class DiffCodePane : TextEditor
{
    protected override Type StyleKeyOverride => typeof(TextEditor);
    private readonly List<CollapsedLineSection> _collapsed = [];
    private readonly OriginalLineMargin _margin;
    private readonly WordForeground _wordForeground;
    private TextMate.Installation? _syntax;
    private RegistryOptions? _registry;
    private string _path = string.Empty;
    private bool _closed;
    internal DiffReviewDocument? ReviewDocument { get; private set; }
    internal bool Before { get; private set; }
    internal int CurrentChange { get; set; }
    internal event Action<DiffReviewBlock>? ContextToggled;

    public DiffCodePane()
    {
        IsReadOnly = true;
        ShowLineNumbers = false;
        WordWrap = false;
        HorizontalScrollBarVisibility = ScrollBarVisibility.Visible;
        VerticalScrollBarVisibility = ScrollBarVisibility.Visible;
        Options.EnableTextDragDrop = false;
        Options.HighlightCurrentLine = false;
        Options.AllowScrollBelowDocument = false;
        Options.CutCopyWholeLine = false;
        Options.IndentationSize = 2;
        TextArea.Caret.CaretBrush = Brushes.Transparent;
        TextArea.RightClickMovesCaret = false;
        _margin = new OriginalLineMargin(this);
        TextArea.TextView.VisualLinesChanged += (_, _) => _margin.InvalidateVisual();
        TextArea.LeftMargins.Clear();
        TextArea.LeftMargins.Add(_margin);
        TextArea.TextView.ElementGenerators.Add(new ContextGenerator(this));
        TextArea.TextView.BackgroundRenderers.Add(new DifferenceBackground(this));
        _wordForeground = new WordForeground(this);
        TextArea.TextView.LineTransformers.Add(_wordForeground);
        TextArea.CommandBindings.Insert(0, new RoutedCommandBinding(ApplicationCommands.Copy, CopySelected,
            (_, args) => { args.CanExecute = SelectionLength > 0; args.Handled = true; }));
        var copy = new MenuItem();
        copy.Bind(MenuItem.HeaderProperty, this.GetResourceObservable("LocCopy"));
        copy.Click += CopySelected;
        ContextMenu = new ContextMenu { ItemsSource = new[] { copy } };
        ContextMenu.Opening += (_, _) => copy.IsEnabled = SelectionLength > 0;
        ActualThemeVariantChanged += (_, _) => RefreshAppearance();
        ResourcesChanged += (_, _) => RefreshAppearance();
        AttachedToVisualTree += (_, _) => RefreshAppearance();
        DetachedFromVisualTree += (_, _) => DisposeSyntax();
    }

    internal void SetReview(DiffReviewDocument? document, bool before, string path)
    {
        var text = document?.Text(before) ?? string.Empty;
        if (Text != text) TextArea.ClearSelection();
        foreach (var section in _collapsed) section.Uncollapse();
        _collapsed.Clear();
        // Text changes can synchronously request visual lines before native folds are installed.
        ReviewDocument = null;
        Before = before;
        _path = path;
        if (document is null || string.IsNullOrEmpty(EditorLanguage.FromFileName(path).GrammarExtension))
            DisposeSyntax();
        if (Text != text) Text = text;
        ReviewDocument = document;
        if (document is not null)
        {
            for (var index = 0; index < document.DisplayRows.Count; index++)
            {
                var display = document.DisplayRows[index];
                if (display.IsFold && !display.Block.Expanded)
                    _collapsed.Add(TextArea.TextView.CollapseLines(Document.GetLineByNumber(index + 2),
                        Document.GetLineByNumber(index + 1 + display.Block.Rows.Count)));
            }
        }
        RefreshAppearance();
    }

    internal void RedrawReview()
    {
        _margin.InvalidateVisual();
        TextArea.TextView.Redraw();
    }

    internal void DisposeReview()
    {
        _closed = true;
        DisposeSyntax();
        foreach (var section in _collapsed) section.Uncollapse();
        _collapsed.Clear();
        ReviewDocument = null;
        ContextToggled = null;
    }

    private void DisposeSyntax()
    {
        _syntax?.Dispose();
        _syntax = null;
        _registry = null;
    }

    internal double ResourceDouble(string key, double fallback) => this.FindResource(key) is double value ? value : fallback;
    internal IBrush Brush(string key) => this.TryFindResource(key, ActualThemeVariant, out var value) && value is IBrush brush ? brush : Brushes.Transparent;
    internal DiffDisplayRow? Row(int line) => ReviewDocument is { } document && line > 0 && line <= document.DisplayRows.Count
        ? document.DisplayRows[line - 1] : null;

    private void RefreshAppearance()
    {
        if (_closed) return;
        Background = Brush("SurfaceBrush");
        Foreground = Brush("TextBrush");
        TextArea.SelectionBrush = Brush("TextSelectionBrush");
        FontSize = ResourceDouble("DiffReviewerCodeSize", 11);
        FontFamily = this.FindResource("MonoFontFamily") as FontFamily ?? FontFamily.Default;
        _margin.Width = ResourceDouble("DiffReviewerGutterWidth", 42) + 12;
        var height = TextArea.TextView.DefaultLineHeight;
        if (height > 0) Options.LineHeightFactor *= ResourceDouble("DiffReviewerRowHeight", 22) / height;
        var extension = EditorLanguage.FromFileName(_path).GrammarExtension;
        if (ReviewDocument is not null && !string.IsNullOrEmpty(extension) && this.IsAttachedToVisualTree())
        {
            var theme = ActualThemeVariant == Avalonia.Styling.ThemeVariant.Dark ? ThemeName.DarkPlus : ThemeName.LightPlus;
            if (_syntax is null)
            {
                _registry = new RegistryOptions(theme);
                _syntax = this.InstallTextMate(_registry,
                    exceptionHandler: exception => DiagnosticLog.Error("diff.syntax", $"TextMate failure type={exception.GetType().Name}"));
            }
            _syntax.SetTheme(_registry!.LoadTheme(theme));
            _syntax.SetGrammar(_registry.GetScopeByExtension(extension));
        }
        TextArea.TextView.LineTransformers.Remove(_wordForeground);
        TextArea.TextView.LineTransformers.Add(_wordForeground);
        RedrawReview();
    }

    private async void CopySelected(object? sender, RoutedEventArgs args)
    {
        args.Handled = true;
        if (ReviewDocument is null || SelectionLength == 0 || TopLevel.GetTopLevel(this)?.Clipboard is not { } clipboard) return;
        var segments = TextArea.Selection.Segments.ToArray();
        var text = segments.Length <= 1 ? ReviewDocument.Copy(Before, SelectionStart, SelectionLength)
            : string.Join('\n', segments
                .Where(segment => Row(Document.GetLineByOffset(segment.StartOffset).LineNumber)?.Line(Before) is not null)
                .Select(segment => ReviewDocument.Copy(Before, segment.StartOffset, segment.Length)));
        await clipboard.SetTextAsync(text);
    }

    private sealed class ContextGenerator(DiffCodePane pane) : VisualLineElementGenerator
    {
        public override int GetFirstInterestedOffset(int startOffset)
        {
            var line = CurrentContext.Document.GetLineByOffset(startOffset);
            return pane.Row(line.LineNumber)?.IsFold == true && startOffset == line.Offset ? startOffset : -1;
        }

        public override VisualLineElement ConstructElement(int offset)
        {
            var line = CurrentContext.Document.GetLineByOffset(offset);
            var block = pane.Row(line.LineNumber)!.Block;
            var label = block.Expanded ? LocalizationService.GetString("LocDiffCollapse", "Collapse")
                : string.Format(LocalizationService.GetString("LocDiffUnchangedLines", "{0} unmodified lines"), block.Rows.Count);
            var button = new Button
            {
                Content = label, Height = pane.ResourceDouble("DiffReviewerFoldHeight", 32), MinHeight = 0,
                Width = Math.Max(140, pane.TextArea.TextView.Bounds.Width), Padding = new Thickness(0),
                HorizontalContentAlignment = Avalonia.Layout.HorizontalAlignment.Left,
                VerticalContentAlignment = Avalonia.Layout.VerticalAlignment.Center,
                FontSize = 11, Background = pane.Brush("FieldBrush"), Foreground = pane.Brush("TextSecondaryBrush"),
                BorderThickness = new Thickness(0), CornerRadius = new CornerRadius(0)
            };
            Avalonia.Automation.AutomationProperties.SetName(button, label);
            TextBlock.SetBaselineOffset(button, pane.TextArea.TextView.DefaultBaseline);
            button.Click += (_, _) => pane.ContextToggled?.Invoke(block);
            var end = block.Expanded ? line.EndOffset : CurrentContext.Document.GetLineByNumber(line.LineNumber + block.Rows.Count).EndOffset;
            return new InlineObjectElement(end - offset, button);
        }
    }

    private sealed class OriginalLineMargin(DiffCodePane pane) : AbstractMargin
    {
        public override void Render(DrawingContext context)
        {
            if (TextView is not { VisualLinesValid: true }) return;
            foreach (var visual in TextView.VisualLines)
            {
                var row = pane.Row(visual.FirstDocumentLine.LineNumber);
                if (row is null) continue;
                var rectangle = new Rect(0, visual.VisualTop - TextView.ScrollOffset.Y, Bounds.Width, visual.Height);
                PaintRow(pane, context, row, rectangle);
                PaintActiveBlockEdges(pane, context, row, rectangle, isMargin: true);
                if (row.IsFold || row.Line(pane.Before) is not { } line) continue;
                var text = new FormattedText(line.Number.ToString(CultureInfo.InvariantCulture), CultureInfo.InvariantCulture,
                    FlowDirection.LeftToRight, new Typeface(pane.FontFamily), pane.FontSize, pane.Brush("TextSecondaryBrush"));
                context.DrawText(text, new Point(pane.ResourceDouble("DiffReviewerGutterWidth", 42) - 8 - text.Width,
                    rectangle.Top + (rectangle.Height - text.Height) / 2));
            }
        }
    }

    private static void PaintRow(DiffCodePane pane, DrawingContext context, DiffDisplayRow row, Rect rectangle)
    {
        if (row.IsFold) context.FillRectangle(pane.Brush("FieldBrush"), rectangle);
        else if (row.Line(pane.Before) is null)
        {
            using var clip = context.PushClip(rectangle);
            var pen = new Pen(pane.Brush("SurfaceHoverBrush"), 1);
            for (var x = rectangle.Left - rectangle.Height; x < rectangle.Right; x += 6)
                context.DrawLine(pen, new Point(x, rectangle.Bottom), new Point(x + rectangle.Height, rectangle.Top));
        }
        else if (row.Block.Changed)
            context.FillRectangle(pane.Brush(pane.Before ? "DiffReviewerDeletedSurfaceBrush" : "DiffReviewerAddedSurfaceBrush"), rectangle);
    }

    private static void PaintActiveBlockEdges(DiffCodePane pane, DrawingContext context, DiffDisplayRow row, Rect rectangle, bool isMargin)
    {
        if (!row.Block.Changed || row.Block.ChangeIndex != pane.CurrentChange) return;
        var border = new Pen(pane.Brush("BorderStrongBrush"), 1);
        if (ReferenceEquals(row.Row, row.Block.Rows[0]))
            context.DrawLine(border, new Point(rectangle.Left, rectangle.Top + 0.5), new Point(rectangle.Right, rectangle.Top + 0.5));
        if (ReferenceEquals(row.Row, row.Block.Rows[^1]))
            context.DrawLine(border, new Point(rectangle.Left, rectangle.Bottom - 0.5), new Point(rectangle.Right, rectangle.Bottom - 0.5));
        var edge = isMargin ? rectangle.Left + 0.5 : rectangle.Right - 0.5;
        context.DrawLine(border, new Point(edge, rectangle.Top), new Point(edge, rectangle.Bottom));
    }

    private sealed class DifferenceBackground(DiffCodePane pane) : IBackgroundRenderer
    {
        public KnownLayer Layer => KnownLayer.Background;
        public void Draw(TextView textView, DrawingContext context)
        {
            if (!textView.VisualLinesValid) return;
            foreach (var visual in textView.VisualLines)
            {
                var line = visual.FirstDocumentLine;
                var row = pane.Row(line.LineNumber);
                if (row is null) continue;
                var rectangle = new Rect(0, visual.VisualTop - textView.ScrollOffset.Y, textView.Bounds.Width, visual.Height);
                PaintRow(pane, context, row, rectangle);
                if (row.Row is null) continue;
                var ranges = pane.Before ? row.Row.BeforeWords : row.Row.AfterWords;
                foreach (var word in ranges.Where(word => word.Length > 0))
                    foreach (var area in BackgroundGeometryBuilder.GetRectsForSegment(textView, new TextSegment { StartOffset = line.Offset + word.Start, Length = word.Length }))
                        context.FillRectangle(pane.Brush(pane.Before ? "DiffReviewerDeletedWordBrush" : "DiffReviewerAddedWordBrush"), area);
                PaintActiveBlockEdges(pane, context, row, rectangle, isMargin: false);
            }
        }
    }

    private sealed class WordForeground(DiffCodePane pane) : DocumentColorizingTransformer
    {
        protected override void ColorizeLine(DocumentLine line)
        {
            if (pane.Row(line.LineNumber)?.Row is not { } row) return;
            foreach (var range in (pane.Before ? row.BeforeWords : row.AfterWords).Where(range => range.Length > 0))
                ChangeLinePart(line.Offset + range.Start, line.Offset + range.Start + range.Length,
                    element => element.TextRunProperties.SetForegroundBrush(pane.Brush("TextBrush")));
        }
    }
}
