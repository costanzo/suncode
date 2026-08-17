using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Markup.Xaml;
using Avalonia.Styling;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Views;

namespace SunCode.Desktop;

public sealed partial class App : Application
{
    private DesktopViewModel? _viewModel;
    private MainWindow? _hubWindow;
    private SettingsWindow? _settingsWindow;
    private AboutWindow? _aboutWindow;
    private readonly Dictionary<string, MainWindow> _projectWindows = [];
    private readonly HashSet<string> _openingProjects = [];

    public override void Initialize()
    {
        AvaloniaXamlLoader.Load(this);
        ConfigureNativeApplicationMenu();
    }

    public override void OnFrameworkInitializationCompleted()
    {
        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
        {
            _viewModel = new DesktopViewModel();
            MacOSDockIcon.Apply();
            _viewModel.ThemeChanged += ApplyTheme;
            _hubWindow = new MainWindow(isHubWindow: true) { DataContext = _viewModel };
            desktop.MainWindow = _hubWindow;
            desktop.Exit += (_, _) =>
            {
                _settingsWindow?.Close();
                _aboutWindow?.Close();
                foreach (var window in _projectWindows.Values.ToArray()) window.Close();
                _viewModel.Dispose();
            };
        }

        base.OnFrameworkInitializationCompleted();
    }

    private void ApplyTheme(string mode) =>
        RequestedThemeVariant = mode == "light" ? ThemeVariant.Light : ThemeVariant.Dark;

    internal async Task OpenProjectPathAsync(string path)
    {
        if (_viewModel is null) return;
        var project = await _viewModel.RegisterProjectAsync(path);
        if (project is not null) await OpenProjectWindowAsync(project);
    }

    internal async Task OpenProjectWindowAsync(ProjectItem project)
    {
        if (_projectWindows.TryGetValue(project.ProjectId, out var existing))
        {
            existing.Show();
            existing.Activate();
            return;
        }
        if (!_openingProjects.Add(project.ProjectId)) return;

        var viewModel = new DesktopViewModel();
        viewModel.ThemeChanged += ApplyTheme;
        try
        {
            await viewModel.InitializeAsync();
            await viewModel.SelectProjectAsync(project);
            if (!viewModel.IsProjectOpen)
            {
                viewModel.Dispose();
                return;
            }

            var window = new MainWindow(isHubWindow: false) { DataContext = viewModel };
            _projectWindows[project.ProjectId] = window;
            window.Closed += (_, _) => ProjectWindowClosed(project.ProjectId, viewModel);
            _hubWindow?.Hide();
            window.Show();
            window.Activate();
        }
        finally
        {
            _openingProjects.Remove(project.ProjectId);
        }
    }

    internal void ShowSettings(Window owner)
    {
        if (_viewModel is null) return;
        if (_settingsWindow is not null)
        {
            _settingsWindow.Activate();
            return;
        }

        _settingsWindow = new SettingsWindow { DataContext = _viewModel };
        SetOtherWindowsEnabled(owner, false);
        _settingsWindow.Closed += (_, _) =>
        {
            SetOtherWindowsEnabled(owner, true);
            _settingsWindow = null;
        };
        _ = _settingsWindow.ShowDialog(owner);
    }

    internal void ShowAbout(Window owner)
    {
        if (_aboutWindow is not null)
        {
            _aboutWindow.Activate();
            return;
        }

        _aboutWindow = new AboutWindow();
        SetOtherWindowsEnabled(owner, false);
        _aboutWindow.Closed += (_, _) =>
        {
            SetOtherWindowsEnabled(owner, true);
            _aboutWindow = null;
        };
        _ = _aboutWindow.ShowDialog(owner);
    }

    private void ConfigureNativeApplicationMenu()
    {
        var menu = new NativeMenu();
        var about = new NativeMenuItem { Header = $"About {AppInfo.ProductName}" };
        about.Click += (_, _) =>
        {
            if (ApplicationLifetime is not IClassicDesktopStyleApplicationLifetime desktop) return;
            var owner = desktop.Windows.FirstOrDefault(window => window.IsActive)
                ?? desktop.Windows.FirstOrDefault(window => window.IsVisible)
                ?? desktop.MainWindow;
            if (owner is not null) ShowAbout(owner);
        };
        menu.Items.Add(about);
        NativeMenu.SetMenu(this, menu);
    }

    internal bool IsProjectOpen(string projectId) => _projectWindows.ContainsKey(projectId);

    private void SetOtherWindowsEnabled(Window owner, bool enabled)
    {
        if (_hubWindow is not null && _hubWindow != owner) _hubWindow.IsEnabled = enabled;
        foreach (var window in _projectWindows.Values)
        {
            if (window != owner) window.IsEnabled = enabled;
        }
    }

    private void ProjectWindowClosed(string projectId, DesktopViewModel viewModel)
    {
        viewModel.ThemeChanged -= ApplyTheme;
        viewModel.Dispose();
        _projectWindows.Remove(projectId);
        if (_projectWindows.Count == 0 && _hubWindow is not null)
        {
            _hubWindow.Show();
            _hubWindow.Activate();
        }
    }
}
