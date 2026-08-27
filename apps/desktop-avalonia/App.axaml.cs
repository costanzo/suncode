using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Markup.Xaml;
using Avalonia.Styling;
using SunCode.Desktop.Models;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Views.About;
using SunCode.Desktop.Views.Settings;
using SunCode.Desktop.Views.Shell;

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
        DiagnosticLog.Info("app.initialize", "xaml_load begin");
        AvaloniaXamlLoader.Load(this);
        ConfigureNativeApplicationMenu();
        DiagnosticLog.Info("app.initialize", "xaml_load end");
    }

    public override void OnFrameworkInitializationCompleted()
    {
        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
        {
            DiagnosticLog.Info("app.lifecycle", "framework_initialization begin");
            _viewModel = new DesktopViewModel();
            MacOSDockIcon.Apply();
            _viewModel.ThemeChanged += ApplyTheme;
            _hubWindow = new MainWindow(isHubWindow: true) { DataContext = _viewModel };
            desktop.MainWindow = _hubWindow;
            desktop.Exit += (_, _) =>
            {
                DiagnosticLog.Info("app.lifecycle", "exit begin");
                _settingsWindow?.Close();
                _aboutWindow?.Close();
                foreach (var window in _projectWindows.Values.ToArray()) window.Close();
                _viewModel.Dispose();
                DiagnosticLog.Info("app.lifecycle", "exit end");
            };
        }

        base.OnFrameworkInitializationCompleted();
    }

    private void ApplyTheme(string mode)
    {
        var variant = mode == "light" ? ThemeVariant.Light : ThemeVariant.Dark;
        RequestedThemeVariant = variant;
        if (_hubWindow is not null) _hubWindow.RequestedThemeVariant = variant;
        if (_settingsWindow is not null) _settingsWindow.RequestedThemeVariant = variant;
        if (_aboutWindow is not null) _aboutWindow.RequestedThemeVariant = variant;
        foreach (var window in _projectWindows.Values)
        {
            window.RequestedThemeVariant = variant;
        }
    }

    internal async Task OpenProjectPathAsync(string path)
    {
        if (_viewModel is null) return;
        var project = await _viewModel.RegisterProjectAsync(path);
        if (project is not null)
        {
            await OpenProjectWindowAsync(project);
        }
        else if (_hubWindow is not null)
        {
            _hubWindow.Show();
            _hubWindow.Activate();
        }
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
            DiagnosticLog.Info("project.window", $"open begin project={project.ProjectId}");
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
            DiagnosticLog.Info("project.window", $"open end project={project.ProjectId}");
        }
        catch (Exception exception)
        {
            DiagnosticLog.Error("project.window", exception, $"project={project.ProjectId}");
            viewModel.Dispose();
            throw;
        }
        finally
        {
            _openingProjects.Remove(project.ProjectId);
        }
    }

    internal void ShowSettings(Window owner)
    {
        var viewModel = owner.DataContext as DesktopViewModel ?? _viewModel;
        if (viewModel is null) return;
        if (_settingsWindow is not null)
        {
            _settingsWindow.Activate();
            return;
        }

        _settingsWindow = new SettingsWindow { DataContext = viewModel };
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
