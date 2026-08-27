using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.Input;
using Avalonia.Interactivity;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.Chat;

public sealed partial class ChatInput : UserControl
{
    public ChatInput()
    {
        InitializeComponent();
        ComposerInput.AddHandler(KeyDownEvent, ComposerKeyDown, RoutingStrategies.Tunnel);
    }

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;

    private async void SubmitTurn(object? sender, RoutedEventArgs e) =>
        await ViewModel.SubmitTurnAsync();

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
}
