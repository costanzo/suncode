using Avalonia.Controls;
using Avalonia.Interactivity;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.Settings.Controls;

public sealed partial class McpServersSettingsControl : UserControl
{
    public event Action? AddRequested;
    public event Action<McpServerItem>? EditRequested;
    public event Action<McpServerItem>? DeleteRequested;

    public McpServersSettingsControl() => InitializeComponent();

    private DesktopViewModel? ViewModel => DataContext as DesktopViewModel;

    private void AddClicked(object? sender, RoutedEventArgs e) => AddRequested?.Invoke();

    private void EditClicked(object? sender, RoutedEventArgs e)
    {
        if (sender is Button { Tag: McpServerItem item }) EditRequested?.Invoke(item);
    }

    private void DeleteClicked(object? sender, RoutedEventArgs e)
    {
        if (sender is Button { Tag: McpServerItem item }) DeleteRequested?.Invoke(item);
    }

    private async void RetryClicked(object? sender, RoutedEventArgs e)
    {
        if (sender is Button { Tag: McpServerItem item } && ViewModel is { } viewModel)
            await viewModel.RetryMcpServerAsync(item);
    }

    private async void ToggleClicked(object? sender, RoutedEventArgs e)
    {
        if (sender is ToggleSwitch { Tag: McpServerItem item } toggle && ViewModel is { } viewModel)
        {
            toggle.IsEnabled = false;
            var saved = await viewModel.SetMcpServerEnabledAsync(item, toggle.IsChecked == true);
            if (!saved) toggle.IsChecked = item.Enabled;
            toggle.IsEnabled = true;
        }
    }
}
