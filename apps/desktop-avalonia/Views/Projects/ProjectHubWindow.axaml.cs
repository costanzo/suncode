using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Platform.Storage;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Views.Projects;

public sealed partial class ProjectHubWindow : Window
{
    private bool _initialized;

    public ProjectHubWindow()
    {
        InitializeComponent();
        WindowDecorations = Avalonia.Controls.WindowDecorations.Full;
        Icon = new WindowIcon(Avalonia.Platform.AssetLoader.Open(new Uri("avares://SunCode/Assets/logo/suncode-logo-128.png")));
        AddHandler(KeyDownEvent, WindowKeyDown, RoutingStrategies.Tunnel);
        Opened += OnOpened;
    }

    private DesktopViewModel ViewModel => (DesktopViewModel)DataContext!;

    private async void OnOpened(object? sender, EventArgs e)
    {
        if (_initialized) return;
        _initialized = true;
        await ViewModel.InitializeAsync();
        ViewModel.UpdateLayoutWidth(Bounds.Width);
    }

    internal async Task OpenProjectPickerAsync()
    {
        var folders = await StorageProvider.OpenFolderPickerAsync(new FolderPickerOpenOptions
        {
            Title = "Open a local project",
            AllowMultiple = false
        });
        var folder = folders.FirstOrDefault();
        var path = folder?.TryGetLocalPath();
        if (string.IsNullOrWhiteSpace(path) && folder?.Path is { IsFile: true } uri)
        {
            path = uri.LocalPath;
        }
        if (string.IsNullOrWhiteSpace(path)) return;
        if (Application.Current is App app) await app.OpenProjectPathAsync(path);
    }

    internal async Task OpenProjectAsync(ProjectItem project)
    {
        if (Application.Current is App app) await app.OpenProjectWindowAsync(project);
    }

    internal void ShowSettings()
    {
        if (Application.Current is App app) app.ShowSettings(this);
    }

    private void WindowKeyDown(object? sender, KeyEventArgs e)
    {
        var commandModifier = e.KeyModifiers.HasFlag(KeyModifiers.Meta) || e.KeyModifiers.HasFlag(KeyModifiers.Control);
        if (!commandModifier || e.Key != Key.OemComma) return;
        e.Handled = true;
        ShowSettings();
    }
}
