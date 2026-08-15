using Avalonia;
using Avalonia.Controls;

namespace SunCode.Desktop.Controls;

public sealed class MarkdownText : ContentControl
{
    public static readonly StyledProperty<string?> MarkdownProperty =
        AvaloniaProperty.Register<MarkdownText, string?>(nameof(Markdown));

    private readonly global::Markdown.Avalonia.Markdown _engine = new();

    public string? Markdown
    {
        get => GetValue(MarkdownProperty);
        set => SetValue(MarkdownProperty, value);
    }

    public MarkdownText()
    {
        HorizontalContentAlignment = Avalonia.Layout.HorizontalAlignment.Stretch;
    }

    protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
    {
        base.OnPropertyChanged(change);
        if (change.Property == MarkdownProperty) Render(change.NewValue as string);
    }

    private void Render(string? value) => Content = _engine.Transform(value ?? string.Empty);
}
