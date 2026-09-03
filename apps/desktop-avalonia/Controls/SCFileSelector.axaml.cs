using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Platform.Storage;

namespace SunCode.Desktop.Controls;

public enum SCFileSelectorSelectionMode
{
    File,
    Folder
}

public sealed partial class SCFileSelector : UserControl
{
    public static readonly StyledProperty<string?> TextProperty =
        AvaloniaProperty.Register<SCFileSelector, string?>(nameof(Text), defaultBindingMode: Avalonia.Data.BindingMode.TwoWay);

    public static readonly StyledProperty<string?> PlaceholderTextProperty =
        AvaloniaProperty.Register<SCFileSelector, string?>(nameof(PlaceholderText));

    public static readonly StyledProperty<string> DialogTitleProperty =
        AvaloniaProperty.Register<SCFileSelector, string>(nameof(DialogTitle), "Choose path");

    public static readonly StyledProperty<string?> FileTypeNameProperty =
        AvaloniaProperty.Register<SCFileSelector, string?>(nameof(FileTypeName), "Allowed files");

    public static readonly StyledProperty<string?> AllowedExtensionsProperty =
        AvaloniaProperty.Register<SCFileSelector, string?>(nameof(AllowedExtensions));

    public static readonly StyledProperty<SCFileSelectorSelectionMode> SelectionModeProperty =
        AvaloniaProperty.Register<SCFileSelector, SCFileSelectorSelectionMode>(nameof(SelectionMode), SCFileSelectorSelectionMode.Folder);

    private bool _syncingText;

    public SCFileSelector()
    {
        InitializeComponent();
        PathInput.TextChanged += PathInputTextChanged;
        BrowseButton.Click += BrowseRequested;
        SyncView();
    }

    public string? Text
    {
        get => GetValue(TextProperty);
        set => SetValue(TextProperty, value);
    }

    public string? PlaceholderText
    {
        get => GetValue(PlaceholderTextProperty);
        set => SetValue(PlaceholderTextProperty, value);
    }

    public string DialogTitle
    {
        get => GetValue(DialogTitleProperty);
        set => SetValue(DialogTitleProperty, value);
    }

    public string? FileTypeName
    {
        get => GetValue(FileTypeNameProperty);
        set => SetValue(FileTypeNameProperty, value);
    }

    public string? AllowedExtensions
    {
        get => GetValue(AllowedExtensionsProperty);
        set => SetValue(AllowedExtensionsProperty, value);
    }

    public SCFileSelectorSelectionMode SelectionMode
    {
        get => GetValue(SelectionModeProperty);
        set => SetValue(SelectionModeProperty, value);
    }

    protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
    {
        base.OnPropertyChanged(change);
        if (change.Property == TextProperty
            || change.Property == PlaceholderTextProperty
            || change.Property == DialogTitleProperty
            || change.Property == SelectionModeProperty
            || change.Property == InputElement.IsEnabledProperty)
        {
            SyncView();
        }
    }

    internal static IReadOnlyList<string> NormalizePatterns(string? extensions)
    {
        if (string.IsNullOrWhiteSpace(extensions))
        {
            return [];
        }

        return extensions
            .Split([",", ";", " "], StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries)
            .Select(static value => value.StartsWith("*.", StringComparison.Ordinal)
                ? value
                : value.StartsWith(".", StringComparison.Ordinal)
                    ? $"*{value}"
                    : $"*.{value}")
            .Distinct(StringComparer.OrdinalIgnoreCase)
            .ToArray();
    }

    private void SyncView()
    {
        _syncingText = true;
        PathInput.Text = Text ?? string.Empty;
        _syncingText = false;
        PathInput.PlaceholderText = PlaceholderText;
        PathInput.IsEnabled = IsEnabled;
        BrowseButton.IsEnabled = IsEnabled;
        ToolTip.SetTip(BrowseButton, SelectionMode == SCFileSelectorSelectionMode.File ? "Choose file" : "Choose folder");
    }

    private void PathInputTextChanged(object? sender, TextChangedEventArgs e)
    {
        if (_syncingText) return;
        Text = PathInput.Text;
    }

    private async void BrowseRequested(object? sender, RoutedEventArgs e)
    {
        var topLevel = TopLevel.GetTopLevel(this);
        if (topLevel?.StorageProvider is null) return;

        if (SelectionMode == SCFileSelectorSelectionMode.Folder)
        {
            var folders = await topLevel.StorageProvider.OpenFolderPickerAsync(new FolderPickerOpenOptions
            {
                Title = DialogTitle,
                AllowMultiple = false
            });
            var folder = folders.FirstOrDefault();
            var path = TryGetLocalPath(folder);
            if (!string.IsNullOrWhiteSpace(path)) Text = path;
            return;
        }

        var patterns = NormalizePatterns(AllowedExtensions);
        var files = await topLevel.StorageProvider.OpenFilePickerAsync(new FilePickerOpenOptions
        {
            Title = DialogTitle,
            AllowMultiple = false,
            FileTypeFilter = patterns.Count == 0
                ? null
                : [new FilePickerFileType(FileTypeName ?? "Allowed files") { Patterns = patterns }]
        });
        var file = files.FirstOrDefault();
        var filePath = TryGetLocalPath(file);
        if (!string.IsNullOrWhiteSpace(filePath)) Text = filePath;
    }

    private static string? TryGetLocalPath(IStorageItem? item)
    {
        if (item is null) return null;
        var localPath = item.TryGetLocalPath();
        if (!string.IsNullOrWhiteSpace(localPath))
        {
            return localPath;
        }

        return item.Path is { IsFile: true } uri
            ? uri.LocalPath
            : null;
    }
}
