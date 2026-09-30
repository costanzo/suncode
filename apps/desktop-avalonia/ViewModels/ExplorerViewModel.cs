using System.Collections.ObjectModel;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;

namespace SunCode.Desktop.ViewModels;

// Read-only project and dependency tree for the selected project. Children are
// listed lazily through the SDK; dependency mutations are run by the window so
// they share its status and busy reporting.
public sealed class ExplorerViewModel : ObservableObject
{
    private readonly IViewModelHost _host;

    internal ExplorerViewModel(IViewModelHost host) => _host = host;

    public ObservableCollection<ProjectDependencyItem> Dependencies { get; } = [];
    public ObservableCollection<ExplorerNode> Roots { get; } = [];
    public bool HasDependencies => Dependencies.Count > 0;

    // Re-reads dependencies, rebuilds the root nodes, and lists their first level.
    internal async Task ReloadAsync()
    {
        await LoadDependenciesAsync();
        ResetRoots();
        await LoadRootsAsync();
    }

    public async Task RefreshAsync()
    {
        if (_host.SelectedProject is null) return;
        ResetRoots();
        await LoadRootsAsync();
    }

    public async Task LoadRootsAsync()
    {
        foreach (var root in Roots)
        {
            if (root.IsGroup)
            {
                foreach (var dependency in root.Children)
                    await LoadChildrenAsync(dependency);
            }
            else
            {
                await LoadChildrenAsync(root);
            }
        }
    }

    public async Task LoadChildrenAsync(ExplorerNode node)
    {
        if (!_host.EnsureSdk() || _host.SelectedProject is not { } project || !node.IsDirectory || node.IsGroup || node.IsLoaded || node.IsLoading) return;
        node.IsLoading = true;
        try
        {
            var result = await _host.Sdk!.ListProjectDirectoryAsync(
                project.ProjectId,
                node.DependencyId,
                node.Path);
            node.Children.Clear();
            foreach (var item in result.Entries)
            {
                node.Children.Add(new ExplorerNode(
                    item.Name,
                    item.Path,
                    item.Kind,
                    node.DependencyId));
            }
            node.IsLoaded = true;
        }
        catch (Exception exception)
        {
            _host.ReportError(exception);
        }
        finally
        {
            node.IsLoading = false;
        }
    }

    private async Task LoadDependenciesAsync()
    {
        Dependencies.Clear();
        if (_host.Sdk is not { } sdk || _host.SelectedProject is not { } project)
        {
            OnPropertyChanged(nameof(HasDependencies));
            return;
        }
        var result = await sdk.ListProjectDependenciesAsync(project.ProjectId);
        foreach (var item in result.Dependencies)
        {
            Dependencies.Add(new ProjectDependencyItem(
                item.DependencyId,
                item.DisplayName));
        }
        OnPropertyChanged(nameof(HasDependencies));
    }

    // Exposed for tests that seed Dependencies directly.
    internal void ResetRoots()
    {
        Roots.Clear();
        if (_host.SelectedProject is not { } project) return;
        Roots.Add(new ExplorerNode(
            project.DisplayName,
            ".",
            "directory",
            isRoot: true));
        var dependencyGroup = new ExplorerNode(
            "Dependencies",
            ".",
            "group",
            isRoot: true,
            isGroup: true);
        foreach (var dependency in Dependencies)
        {
            dependencyGroup.Children.Add(new ExplorerNode(
                dependency.DisplayName,
                ".",
                "directory",
                dependency.DependencyId,
                isRoot: true,
                isDependency: true));
        }
        dependencyGroup.IsLoaded = true;
        Roots.Add(dependencyGroup);
    }
}
