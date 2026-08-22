using Avalonia;
using Avalonia.Controls;
using LiveMarkdown.Avalonia;

namespace SunCode.Desktop.Controls;

public sealed class MarkdownText : ContentControl
{
    public static readonly StyledProperty<string?> MarkdownProperty =
        AvaloniaProperty.Register<MarkdownText, string?>(nameof(Markdown));

    private readonly ObservableStringBuilder _markdownBuilder = new();
    private readonly MarkdownRenderer _renderer = new();
    private string _renderedMarkdown = string.Empty;

    public string? Markdown
    {
        get => GetValue(MarkdownProperty);
        set => SetValue(MarkdownProperty, value);
    }

    public MarkdownText()
    {
        HorizontalContentAlignment = Avalonia.Layout.HorizontalAlignment.Stretch;
        _renderer.MarkdownBuilder = _markdownBuilder;
        Content = _renderer;
    }

    protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
    {
        base.OnPropertyChanged(change);
        if (change.Property == MarkdownProperty) Render(change.NewValue as string);
    }

    private void Render(string? value)
    {
        var markdown = value ?? string.Empty;
        if (markdown.StartsWith(_renderedMarkdown, StringComparison.Ordinal))
        {
            _markdownBuilder.Append(markdown[_renderedMarkdown.Length..]);
            _renderedMarkdown = markdown;
            return;
        }

        _markdownBuilder.Clear();
        _markdownBuilder.Append(markdown);
        _renderedMarkdown = markdown;
    }
}
