using Avalonia;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Styling;
using LiveMarkdown.Avalonia;

namespace SunCode.Desktop.Controls;

public sealed class MarkdownText : ContentControl
{
    public static readonly StyledProperty<string?> MarkdownProperty =
        AvaloniaProperty.Register<MarkdownText, string?>(nameof(Markdown));
    public static readonly StyledProperty<bool> IsSecondaryProperty =
        AvaloniaProperty.Register<MarkdownText, bool>(nameof(IsSecondary));

    private readonly ObservableStringBuilder _markdownBuilder = new();
    private readonly MarkdownRenderer _renderer = new();
    private readonly Dictionary<CodeBlock, int> _copySuccessVersions = [];
    private string _renderedMarkdown = string.Empty;

    public string? Markdown
    {
        get => GetValue(MarkdownProperty);
        set => SetValue(MarkdownProperty, value);
    }

    public bool IsSecondary
    {
        get => GetValue(IsSecondaryProperty);
        set => SetValue(IsSecondaryProperty, value);
    }

    public MarkdownText()
    {
        MarkdownSyntaxThemes.Register();
        HorizontalContentAlignment = Avalonia.Layout.HorizontalAlignment.Stretch;
        _renderer.MarkdownBuilder = _markdownBuilder;
        ActualThemeVariantChanged += HandleActualThemeVariantChanged;
        UpdateCodeBlockTheme();
        _renderer.AddHandler(
            CodeBlock.CopyingToClipboardEvent,
            HandleCopyingToClipboard,
            RoutingStrategies.Bubble,
            handledEventsToo: true);
        _renderer.AddHandler(
            MarkdownTextBlock.LinkClickEvent,
            HandleLinkClick,
            RoutingStrategies.Bubble,
            handledEventsToo: true);
        Content = _renderer;
    }
    
    private void HandleActualThemeVariantChanged(object? sender, EventArgs e) => UpdateCodeBlockTheme();

    private void UpdateCodeBlockTheme()
    {
        _renderer.CodeBlockCustomColorTheme = ActualThemeVariant == ThemeVariant.Dark 
            ? MarkdownSyntaxThemes.DarkName 
            : MarkdownSyntaxThemes.LightName;
    }

    protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
    {
        base.OnPropertyChanged(change);
        if (change.Property == MarkdownProperty) Render(change.NewValue as string);
        if (change.Property == IsSecondaryProperty) UpdatePresentation();
    }

    private void UpdatePresentation()
    {
        if (IsSecondary)
        {
            if (!_renderer.Classes.Contains("secondary")) _renderer.Classes.Add("secondary");
        }
        else
        {
            _renderer.Classes.Remove("secondary");
        }
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

    private async void HandleLinkClick(object? sender, LinkClickedEventArgs e)
    {
        try
        {
            if (e.HRef is { IsAbsoluteUri: true, Scheme: "http" or "https" } url)
            {
                var launcher = TopLevel.GetTopLevel(_renderer)?.Launcher;
                if (launcher is not null)
                {
                    await launcher.LaunchUriAsync(url);
                }
            }
        }
        catch
        {
            // Ignore errors when opening links, as we don't want to crash the app for this
        }
    }
    

    private async void HandleCopyingToClipboard(object? sender, RoutedEventArgs e)
    {
        if (e.Source is not CodeBlock codeBlock)
            return;

        if (string.IsNullOrEmpty(codeBlock.Inlines.Text) || TopLevel.GetTopLevel(codeBlock)?.Clipboard is null)
            return;

        var version = _copySuccessVersions.TryGetValue(codeBlock, out var previousVersion)
            ? previousVersion + 1
            : 1;
        _copySuccessVersions[codeBlock] = version;
        if (!codeBlock.Classes.Contains("copy-success"))
            codeBlock.Classes.Add("copy-success");

        await Task.Delay(TimeSpan.FromSeconds(3));
        if (_copySuccessVersions.TryGetValue(codeBlock, out var currentVersion) && currentVersion == version)
        {
            codeBlock.Classes.Remove("copy-success");
        }
    }
}
