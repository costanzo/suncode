using Avalonia.Controls;
using SunCode.Desktop.Models;

namespace SunCode.Desktop.Infrastructure;

// Application-owned Dock menu. Opening a native menu stays synchronous; actions
// route through existing project and window services after selection.
internal sealed class DesktopDockMenuController
{
    private const int RecentProjectLimit = 10;
    private readonly Func<IReadOnlyList<ProjectItem>> _projects;
    private readonly Func<IReadOnlyList<ProjectItem>> _openProjects;
    private readonly Func<ProjectItem, Task> _openProject;
    private readonly Func<bool> _canExecute;
    private readonly NativeMenuItem _picker;
    private readonly NativeMenuItem _hub;
    private readonly NativeMenuItem _windows = new() { Menu = new NativeMenu() };
    private readonly NativeMenuItem _recent = new() { Menu = new NativeMenu() };

    internal NativeMenu Menu { get; } = new();

    internal DesktopDockMenuController(
        Func<IReadOnlyList<ProjectItem>> projects,
        Func<IReadOnlyList<ProjectItem>> openProjects,
        Func<ProjectItem, Task> openProject,
        Func<Task> openProjectPicker,
        Action showProjectHub,
        Func<bool>? canExecute = null)
    {
        _projects = projects;
        _openProjects = openProjects;
        _openProject = openProject;
        _canExecute = canExecute ?? (() => true);
        _picker = Command(() => RunAsync(openProjectPicker));
        _hub = Command(() => { showProjectHub(); return Task.CompletedTask; });
        Menu.Items.Add(_picker);
        Menu.Items.Add(_windows);
        Menu.Items.Add(_recent);
        Menu.Items.Add(new NativeMenuItemSeparator());
        Menu.Items.Add(_hub);
        Menu.NeedsUpdate += (_, _) => Refresh();
        _windows.Menu!.NeedsUpdate += (_, _) => PopulateWindows();
        _recent.Menu!.NeedsUpdate += (_, _) => PopulateRecent();
        Refresh();
    }

    internal void Refresh()
    {
        _picker.Header = L("OpenProject", "Open Project…");
        _hub.Header = L("ProjectHub", "Show Project Hub");
        _windows.Header = L("OpenWindows", "Open Windows");
        _recent.Header = L("RecentProjects", "Recent Projects");
        ((AsyncRelayCommand)_picker.Command!).RaiseCanExecuteChanged();
        ((AsyncRelayCommand)_hub.Command!).RaiseCanExecuteChanged();
        _picker.IsEnabled = _hub.IsEnabled = _windows.IsEnabled = _recent.IsEnabled = _canExecute();
        PopulateWindows();
        PopulateRecent();
    }

    private void PopulateWindows() => Populate(_windows.Menu!, _openProjects(), "NoWindows", "No Open Windows", openOnly: true);

    private void PopulateRecent() => Populate(_recent.Menu!, _projects().Take(RecentProjectLimit).ToArray(),
        "NoRecentProjects", "No Recent Projects", openOnly: false);

    private void Populate(NativeMenu menu, IReadOnlyList<ProjectItem> projects, string emptyKey, string emptyLabel, bool openOnly)
    {
        menu.Items.Clear();
        foreach (var project in projects.DistinctBy(item => item.ProjectId))
        {
            var item = Command(() => RunAsync(() => _openProject(project)),
                () => !openOnly || _openProjects().Any(current => current.ProjectId == project.ProjectId));
            var name = string.IsNullOrWhiteSpace(project.DisplayName) ? project.CanonicalRoot : project.DisplayName;
            item.Header = projects.Any(other => other.ProjectId != project.ProjectId &&
                string.Equals(other.DisplayName, project.DisplayName, StringComparison.OrdinalIgnoreCase))
                ? $"{name} — {project.CanonicalRoot}" : name;
            menu.Items.Add(item);
        }
        if (menu.Items.Count == 0)
            menu.Items.Add(new NativeMenuItem { Header = L(emptyKey, emptyLabel), IsEnabled = false });
    }

    private NativeMenuItem Command(Func<Task> execute, Func<bool>? available = null)
    {
        bool Enabled() => _canExecute() && (available?.Invoke() ?? true);
        return new NativeMenuItem
        {
            IsEnabled = Enabled(),
            Command = new AsyncRelayCommand(async () => { if (Enabled()) await execute(); }, Enabled)
        };
    }

    private static string L(string key, string fallback) => LocalizationService.GetString("LocMenu" + key, fallback);

    private static async Task RunAsync(Func<Task> operation)
    {
        try { await operation(); }
        catch (Exception exception) { DiagnosticLog.Error("dock-menu.command", exception, "operation=execute"); }
    }
}
