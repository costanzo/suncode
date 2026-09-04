using System.Collections.ObjectModel;
using System.Collections.Specialized;
using System.ComponentModel;
using System.IO;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Input.Platform;
using Avalonia.Interactivity;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using Avalonia.Platform.Storage;
using Avalonia.Threading;
using SunCode.Desktop.Controls;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.ProjectWorkspace.Chat;

public sealed partial class ChatInput : UserControl
{
    private const int MaxAttachments = 3;
    private const int ThumbnailEdge = 96;
    private const int MaxImageBytes = 20 * 1024 * 1024;
    private const long MaxImagePixels = 50_000_000;
    private DesktopViewModel? _subscribedViewModel;
    private string _expandedDraft = string.Empty;
    private bool _selectorRefreshQueued;
    private bool _selectorRefreshInProgress;

    /// <summary>
    /// Raised when the expanded composer should be shown. The modal itself is
    /// hosted by ProjectWorkspace so its backdrop can cover the full window.
    /// </summary>
    public event EventHandler? ExpandedComposerRequested;

    internal string ExpandedComposerDraft => _expandedDraft;

    public ChatInput()
    {
        InitializeComponent();
        ComposerInput.AddHandler(KeyDownEvent, ComposerKeyDown, RoutingStrategies.Tunnel);
        ComposerInput.AddHandler(TextBox.PastingFromClipboardEvent, ComposerPaste);
        DataContextChanged += (_, _) => RebindViewModelSubscriptions();
        AttachedToVisualTree += (_, _) => RebindViewModelSubscriptions();
    }

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;

    private async void AddAttachment(object? sender, RoutedEventArgs e)
    {
        if (!ViewModel.CanAttachImages || ViewModel.ComposerAttachments.Count >= MaxAttachments) return;

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
                var bytes = await ReadImageBytesAsync(stream);
                var thumbnail = await Task.Run(() => CreateThumbnailBytes(bytes));
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

    private void ExpandComposer(object? sender, RoutedEventArgs e)
    {
        _expandedDraft = ComposerInput.Text ?? string.Empty;
        ExpandedComposerRequested?.Invoke(this, EventArgs.Empty);
    }

    internal void SetComposerText(string text)
    {
        _expandedDraft = text;
        ComposerInput.Text = text;
        ViewModel.ComposerText = text;
    }

    internal void ClearComposerText()
    {
        _expandedDraft = string.Empty;
        ComposerInput.Text = string.Empty;
    }

    private void ModelSelectionChanged(object? sender, SelectionChangedEventArgs e)
    {
        if (ModelSelector.SelectedItem?.Value is not ModelItem selected) return;

        // Resolve back to the ViewModel-owned instance. The selector rebuilds
        // presentation items whenever model data changes, so its item payload
        // must not become a second source of model state.
        var model = ViewModel.Models.FirstOrDefault(item => item.Id == selected.Id) ?? selected;
        ViewModel.SelectedModel = model;
    }

    private void ReasoningSelectionChanged(object? sender, SelectionChangedEventArgs e) =>
        ViewModel.SelectedReasoningEffort = ReasoningSelector.SelectedItem?.Value as string;

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
            || !ViewModel.CanAttachImages
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
                if (bytes.Length > MaxImageBytes) throw new InvalidDataException("Image exceeds the 20 MB limit.");
                ValidatePixelCount(bitmap);
                var thumbnail = await Task.Run(() => CreateThumbnailBytes(bytes));
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

    private static async Task<byte[]> ReadImageBytesAsync(Stream stream)
    {
        if (stream.CanSeek && stream.Length > MaxImageBytes)
            throw new InvalidDataException("Image exceeds the 20 MB limit.");

        using var memory = new MemoryStream(stream.CanSeek ? (int)Math.Min(stream.Length, MaxImageBytes) : 0);
        var buffer = new byte[64 * 1024];
        while (true)
        {
            var read = await stream.ReadAsync(buffer);
            if (read == 0) break;
            if (memory.Length + read > MaxImageBytes)
                throw new InvalidDataException("Image exceeds the 20 MB limit.");
            await memory.WriteAsync(buffer.AsMemory(0, read));
        }
        if (memory.Length == 0) throw new InvalidDataException("Image is empty.");
        return memory.ToArray();
    }

    private static byte[] CreateThumbnailBytes(byte[] bytes)
    {
        using var original = new Bitmap(new MemoryStream(bytes, writable: false));
        ValidatePixelCount(original);
        var scale = Math.Min(1d, ThumbnailEdge / (double)Math.Max(original.PixelSize.Width, original.PixelSize.Height));
        var width = Math.Max(1, (int)Math.Round(original.PixelSize.Width * scale));
        var height = Math.Max(1, (int)Math.Round(original.PixelSize.Height * scale));
        using var thumbnail = original.CreateScaledBitmap(new PixelSize(width, height), BitmapInterpolationMode.HighQuality);
        using var stream = new MemoryStream();
        thumbnail.Save(stream, PngBitmapEncoderOptions.Default);
        return stream.ToArray();
    }

    private static void ValidatePixelCount(Bitmap bitmap)
    {
        var pixels = (long)bitmap.PixelSize.Width * bitmap.PixelSize.Height;
        if (pixels <= 0 || pixels > MaxImagePixels)
            throw new InvalidDataException("Image dimensions are too large.");
    }

    private static string ExtensionFromName(string name)
    {
        var extension = Path.GetExtension(name).TrimStart('.').ToLowerInvariant();
        return string.IsNullOrWhiteSpace(extension) ? "png" : extension;
    }

    private void RebindViewModelSubscriptions()
    {
        if (_subscribedViewModel is not null)
        {
            _subscribedViewModel.Models.CollectionChanged -= ModelDataChanged;
            _subscribedViewModel.Providers.CollectionChanged -= ModelDataChanged;
            _subscribedViewModel.PropertyChanged -= ViewModelPropertyChanged;
        }

        _subscribedViewModel = DataContext as DesktopViewModel;
        if (_subscribedViewModel is null)
        {
            return;
        }

        _subscribedViewModel.Models.CollectionChanged += ModelDataChanged;
        _subscribedViewModel.Providers.CollectionChanged += ModelDataChanged;
        _subscribedViewModel.PropertyChanged += ViewModelPropertyChanged;
        RefreshSelectors();
    }

    private void ModelDataChanged(object? sender, NotifyCollectionChangedEventArgs e) =>
        QueueRefreshSelectors();

    private void ViewModelPropertyChanged(object? sender, PropertyChangedEventArgs e)
    {
        if (e.PropertyName is nameof(DesktopViewModel.SelectedModel) or nameof(DesktopViewModel.SelectedReasoningEffort))
        {
            QueueRefreshSelectors();
        }
    }

    private void QueueRefreshSelectors()
    {
        if (_selectorRefreshQueued) return;
        _selectorRefreshQueued = true;
        Dispatcher.UIThread.Post(() =>
        {
            _selectorRefreshQueued = false;
            RefreshSelectors();
        }, DispatcherPriority.Background);
    }

    private void RefreshSelectors()
    {
        if (_selectorRefreshInProgress) return;
        if (DataContext is not DesktopViewModel viewModel)
        {
            return;
        }

        _selectorRefreshInProgress = true;
        try
        {
        var groups = viewModel.Providers
            .Select(provider => new SCComboBoxGroup(
                provider.DisplayName,
                viewModel.ModelsForProvider(provider.Id)
                    .Select(model => new SCComboBoxItem(model.Display, model))
                    .ToArray()))
            .ToArray();
        ModelSelector.GroupSource = groups;
        ModelSelector.SelectedItem = groups
            .SelectMany(group => group.Items)
            .FirstOrDefault(item => item.Value is ModelItem model && model.Id == viewModel.SelectedModel?.Id);

        var effortItems = viewModel.ReasoningEffortOptions
            .Select(option => new SCComboBoxItem(option.ToUpperInvariant(), option))
            .ToArray();
        ReasoningSelector.ItemsSource = effortItems;
        ReasoningSelector.SelectedItem = effortItems.FirstOrDefault(item => Equals(item.Value, viewModel.SelectedReasoningEffort));
        }
        finally
        {
            _selectorRefreshInProgress = false;
        }
    }
}
