using System.Collections.ObjectModel;
using System.IO;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.Input;
using Avalonia.Input.Platform;
using Avalonia.Interactivity;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using Avalonia.Platform.Storage;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.Chat;

public sealed partial class ChatInput : UserControl
{
    private const int MaxAttachments = 3;
    private const int ThumbnailEdge = 96;

    public ChatInput()
    {
        InitializeComponent();
        ComposerInput.AddHandler(KeyDownEvent, ComposerKeyDown, RoutingStrategies.Tunnel);
        ComposerInput.AddHandler(TextBox.PastingFromClipboardEvent, ComposerPaste);
    }

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;

    private async void AddAttachment(object? sender, RoutedEventArgs e)
    {
        if (ViewModel.ComposerAttachments.Count >= MaxAttachments) return;

        var topLevel = TopLevel.GetTopLevel(this);
        if (topLevel?.StorageProvider is null) return;

        var files = await topLevel.StorageProvider.OpenFilePickerAsync(new FilePickerOpenOptions
        {
            Title = "Add images",
            AllowMultiple = true,
            FileTypeFilter =
            [
                new FilePickerFileType("Images")
                {
                    Patterns = ["*.png", "*.jpg", "*.jpeg", "*.gif", "*.webp", "*.bmp", "*.avif"],
                    MimeTypes = ["image/*"]
                }
            ]
        });

        foreach (var file in files.Take(MaxAttachments - ViewModel.ComposerAttachments.Count))
        {
            try
            {
                var localPath = file.TryGetLocalPath();
                await using var stream = await file.OpenReadAsync();
                using var memory = new MemoryStream();
                await stream.CopyToAsync(memory);
                var bytes = memory.ToArray();
                using var original = new Bitmap(new MemoryStream(bytes, writable: false));
                var thumbnail = CreateThumbnailBytes(original);
                var extension = ExtensionFromName(file.Name);
                await ViewModel.AddSessionImageAsync(
                    file.Name,
                    "file",
                    localPath,
                    extension,
                    bytes,
                    thumbnail);
            }
            catch (Exception exception)
            {
                ViewModel.ReportPresentationError($"Could not load image '{file.Name}': {exception.Message}");
            }
        }
    }

    private void RemoveAttachment(object? sender, RoutedEventArgs e)
    {
        if ((sender as Control)?.DataContext is not ComposerAttachment attachment) return;
        _ = ViewModel.RemoveSessionImageAsync(attachment);
    }

    private async void PreviewAttachment(object? sender, RoutedEventArgs e)
    {
        if ((sender as Control)?.DataContext is not ComposerAttachment attachment) return;
        if (TopLevel.GetTopLevel(this) is not Window owner) return;
        Bitmap? bitmap = null;
        try
        {
            bitmap = File.Exists(attachment.StoragePath)
                ? new Bitmap(attachment.StoragePath)
                : attachment.Preview;

            var preview = new Window
            {
                Title = attachment.Name,
                Width = 720,
                Height = 560,
                MinWidth = 360,
                MinHeight = 280,
                WindowStartupLocation = WindowStartupLocation.CenterOwner,
                Background = this.FindResource("SurfaceRaisedBrush") as IBrush,
                Content = new Border
                {
                    Margin = new Thickness(18),
                    Padding = new Thickness(1),
                    Background = this.FindResource("FieldBrush") as IBrush,
                    BorderBrush = this.FindResource("BorderBrush") as IBrush,
                    BorderThickness = new Thickness(1),
                    Child = new Image
                    {
                        Source = bitmap,
                        Stretch = Avalonia.Media.Stretch.Uniform
                    }
                }
            };
            await preview.ShowDialog(owner);
        }
        finally
        {
            if (bitmap is not null && !ReferenceEquals(bitmap, attachment.Preview))
            {
                bitmap.Dispose();
            }
        }
    }

    private async void SubmitTurn(object? sender, RoutedEventArgs e)
    {
        if (!ViewModel.CanSubmit) return;
        await ViewModel.SubmitTurnAsync();
    }

    private async void CancelTurn(object? sender, RoutedEventArgs e) =>
        await ViewModel.CancelTurnAsync();

    private void OpenModelMenu(object? sender, RoutedEventArgs e)
    {
        var menu = new MenuFlyout
        {
            Placement = PlacementMode.TopEdgeAlignedRight
        };
        foreach (var provider in ViewModel.Providers)
        {
            var providerItem = new MenuItem
            {
                Header = provider.Display
            };
            foreach (var model in ViewModel.ModelsForProvider(provider.Id))
            {
                var modelItem = new MenuItem
                {
                    Header = model.Display,
                    CommandParameter = model,
                    ToggleType = MenuItemToggleType.Radio,
                    GroupName = "models",
                    IsChecked = model == ViewModel.SelectedModel
                };
                modelItem.Click += SelectModel;
                providerItem.Items.Add(modelItem);
            }
            menu.Items.Add(providerItem);
        }
        menu.ShowAt(ModelMenuButton);
    }

    private void SelectModel(object? sender, RoutedEventArgs e)
    {
        if (sender is MenuItem { CommandParameter: ModelItem model })
        {
            ViewModel.SelectedModel = model;
        }
    }

    private async void ComposerKeyDown(object? sender, KeyEventArgs e)
    {
        if (e.Key != Key.Enter || e.KeyModifiers.HasFlag(KeyModifiers.Shift)) return;
        if (!ViewModel.CanSubmit) return;

        e.Handled = true;
        ViewModel.ComposerText = ComposerInput.Text ?? string.Empty;
        await ViewModel.SubmitTurnAsync();
    }

    private async void ComposerPaste(object? sender, RoutedEventArgs e)
    {
        if (sender is not TextBox
            || TopLevel.GetTopLevel(this)?.Clipboard is not { } clipboard
            || ViewModel.ComposerAttachments.Count >= MaxAttachments)
        {
            return;
        }
        try
        {
            var bitmap = await clipboard.TryGetBitmapAsync();
            if (bitmap is null) return;
                using (bitmap)
            {
                using var originalStream = new MemoryStream();
                bitmap.Save(originalStream, PngBitmapEncoderOptions.Default);
                var bytes = originalStream.ToArray();
                var thumbnail = CreateThumbnailBytes(bitmap);
                await ViewModel.AddSessionImageAsync(
                    $"clipboard-{DateTimeOffset.Now:yyyyMMdd-HHmmss}.png",
                    "clipboard",
                    null,
                    "png",
                    bytes,
                    thumbnail);
            }
            e.Handled = true;
        }
        catch (Exception exception)
        {
            ViewModel.ReportPresentationError($"Could not read image from clipboard: {exception.Message}");
        }
    }

    private static byte[] CreateThumbnailBytes(Bitmap original)
    {
        var scale = Math.Min(1d, ThumbnailEdge / (double)Math.Max(original.PixelSize.Width, original.PixelSize.Height));
        var width = Math.Max(1, (int)Math.Round(original.PixelSize.Width * scale));
        var height = Math.Max(1, (int)Math.Round(original.PixelSize.Height * scale));
        using var thumbnail = original.CreateScaledBitmap(new PixelSize(width, height), BitmapInterpolationMode.HighQuality);
        using var stream = new MemoryStream();
        thumbnail.Save(stream, PngBitmapEncoderOptions.Default);
        return stream.ToArray();
    }

    private static string ExtensionFromName(string name)
    {
        var extension = Path.GetExtension(name).TrimStart('.').ToLowerInvariant();
        return string.IsNullOrWhiteSpace(extension) ? "png" : extension;
    }
}
