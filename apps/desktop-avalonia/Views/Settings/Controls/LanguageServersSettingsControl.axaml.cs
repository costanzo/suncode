using System.Collections.Specialized;
using Avalonia.Controls;
using Avalonia.Interactivity;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.Settings.Controls;

public sealed partial class LanguageServersSettingsControl : UserControl
{
    private const double CompactWidth = 560;
    private DesktopViewModel? _subscribedViewModel;
    private double _currentWidth;

    public event Action? AddRequested;
    public event Action<LanguageServerItem>? EditRequested;
    public event Action<LanguageServerItem>? DeleteRequested;

    public LanguageServersSettingsControl()
    {
        InitializeComponent();
        SizeChanged += (_, args) =>
        {
            _currentWidth = args.NewSize.Width;
            UpdateResponsiveLayout();
        };
        DataContextChanged += (_, _) => Rebind();
        AttachedToVisualTree += (_, _) => Rebind();
        DetachedFromVisualTree += (_, _) => Unbind();
    }

    private DesktopViewModel? ViewModel => DataContext as DesktopViewModel;

    private void Rebind()
    {
        Unbind();
        _subscribedViewModel = ViewModel;
        if (_subscribedViewModel is not null)
            _subscribedViewModel.LanguageServers.CollectionChanged += LanguageServersChanged;
        UpdateResponsiveLayout();
    }

    private void Unbind()
    {
        if (_subscribedViewModel is not null)
            _subscribedViewModel.LanguageServers.CollectionChanged -= LanguageServersChanged;
        _subscribedViewModel = null;
    }

    private void LanguageServersChanged(object? sender, NotifyCollectionChangedEventArgs e) => UpdateResponsiveLayout();

    private void UpdateResponsiveLayout()
    {
        var hasServers = ViewModel?.HasLanguageServers == true;
        var compact = _currentWidth < CompactWidth;
        RegularList.IsVisible = hasServers && !compact;
        CompactList.IsVisible = hasServers && compact;
    }

    private void AddClicked(object? sender, RoutedEventArgs e) => AddRequested?.Invoke();

    private void EditClicked(object? sender, RoutedEventArgs e)
    {
        if (sender is Button { Tag: LanguageServerItem item }) EditRequested?.Invoke(item);
    }

    private void DeleteClicked(object? sender, RoutedEventArgs e)
    {
        if (sender is Button { Tag: LanguageServerItem item }) DeleteRequested?.Invoke(item);
    }

    private async void RetryClicked(object? sender, RoutedEventArgs e)
    {
        if (sender is Button { Tag: LanguageServerItem item } && ViewModel is { } viewModel)
            await viewModel.RetryLanguageServerAsync(item);
    }

    private async void ToggleClicked(object? sender, RoutedEventArgs e)
    {
        if (sender is ToggleSwitch { Tag: LanguageServerItem item } toggle && ViewModel is { } viewModel)
        {
            toggle.IsEnabled = false;
            var saved = await viewModel.SetLanguageServerEnabledAsync(item, toggle.IsChecked == true);
            if (!saved) toggle.IsChecked = item.Enabled;
            toggle.IsEnabled = item.CanToggle;
        }
    }
}
