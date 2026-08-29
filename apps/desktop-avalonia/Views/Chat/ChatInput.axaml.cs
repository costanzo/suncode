using System.Collections.ObjectModel;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.Input;
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

    public ChatInput()
    {
        InitializeComponent();
        AttachmentStrip.ItemsSource = Attachments;
        ComposerInput.AddHandler(KeyDownEvent, ComposerKeyDown, RoutingStrategies.Tunnel);
    }

    public ObservableCollection<ComposerAttachment> Attachments { get; } = [];

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;

    private async void AddAttachment(object? sender, RoutedEventArgs e)
    {
        if (Attachments.Count >= MaxAttachments) return;

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

        foreach (var file in files.Take(MaxAttachments - Attachments.Count))
        {
            try
            {
                await using var stream = await file.OpenReadAsync();
                var bitmap = new Bitmap(stream);
                Attachments.Add(new ComposerAttachment(file.Name, bitmap));
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
        RemoveAttachment(attachment);
    }

    private void RemoveAttachment(ComposerAttachment attachment)
    {
        if (Attachments.Remove(attachment)) attachment.Dispose();
    }

    private async void PreviewAttachment(object? sender, RoutedEventArgs e)
    {
        if ((sender as Control)?.DataContext is not ComposerAttachment attachment) return;
        if (TopLevel.GetTopLevel(this) is not Window owner) return;

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
                    Source = attachment.Preview,
                    Stretch = Avalonia.Media.Stretch.Uniform
                }
            }
        };
        await preview.ShowDialog(owner);
    }

    private async void SubmitTurn(object? sender, RoutedEventArgs e)
    {
        if (!ViewModel.CanSubmit) return;
        ClearAttachments();
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
        ClearAttachments();
        await ViewModel.SubmitTurnAsync();
    }

    private void ClearAttachments()
    {
        foreach (var attachment in Attachments) attachment.Dispose();
        Attachments.Clear();
    }

    protected override void OnDetachedFromVisualTree(Avalonia.VisualTreeAttachmentEventArgs e)
    {
        ClearAttachments();
        base.OnDetachedFromVisualTree(e);
    }
}
