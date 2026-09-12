using Avalonia;
using Avalonia.Automation;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.VisualTree;
using LiveMarkdown.Avalonia;

namespace SunCode.Desktop.Controls;

public sealed class MarkdownText : ContentControl
{
    public static readonly StyledProperty<string?> MarkdownProperty =
        AvaloniaProperty.Register<MarkdownText, string?>(nameof(Markdown));

    private readonly ObservableStringBuilder _markdownBuilder = new();
    private readonly MarkdownRenderer _renderer = new();
    private readonly Dictionary<Button, int> _copySuccessVersions = [];
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
        _renderer.AddHandler(CodeBlock.CopyingToClipboardEvent, HandleCopyingToClipboard);
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

    private async void HandleCopyingToClipboard(object? sender, RoutedEventArgs e)
    {
        if (e.Source is not CodeBlock codeBlock)
            return;

        if (string.IsNullOrEmpty(codeBlock.Inlines.Text) || TopLevel.GetTopLevel(codeBlock)?.Clipboard is null)
            return;

        var copyButton = codeBlock.GetVisualDescendants()
            .OfType<Button>()
            .FirstOrDefault(button => button.Name == "PART_CopyButton");
        if (copyButton is null)
            return;

        var version = _copySuccessVersions.TryGetValue(copyButton, out var previousVersion)
            ? previousVersion + 1
            : 1;
        _copySuccessVersions[copyButton] = version;
        if (!copyButton.Classes.Contains("copy-success"))
            copyButton.Classes.Add("copy-success");

        await Task.Delay(TimeSpan.FromSeconds(3));
        if (_copySuccessVersions.TryGetValue(copyButton, out var currentVersion) && currentVersion == version)
        {
            copyButton.Classes.Remove("copy-success");
            ToolTip.SetTip(copyButton, "Copy code");
            AutomationProperties.SetName(copyButton, "Copy code");
        }
    }
}
