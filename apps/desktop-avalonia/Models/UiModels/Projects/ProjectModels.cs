using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Text.Encodings.Web;
using System.Text.Json;
using System.Text.Json.Nodes;
using Avalonia.Media.Imaging;
using SunCode.Desktop.Infrastructure;

namespace SunCode.Desktop.Models;

public sealed record ProjectItem(string ProjectId, string DisplayName, string CanonicalRoot);

public sealed record ProjectDependencyItem(string DependencyId, string DisplayName);

public sealed class ExplorerNode : ObservableObject
{
    private bool _isLoading;
    private bool _isLoaded;
    private bool _isExpanded;

    public ExplorerNode(
        string name,
        string path,
        string kind,
        string? dependencyId = null,
        bool isRoot = false,
        bool isDependency = false,
        bool isGroup = false)
    {
        Name = name;
        Path = path;
        Kind = kind;
        DependencyId = dependencyId;
        IsRoot = isRoot;
        IsDependency = isDependency;
        IsGroup = isGroup;
        _isExpanded = isRoot || isGroup;
        if (IsDirectory && !isGroup) Children.Add(Placeholder());
    }

    private ExplorerNode()
    {
        Name = string.Empty;
        Path = string.Empty;
        Kind = "placeholder";
        IsPlaceholder = true;
    }

    public string Name { get; }
    public string Path { get; }
    public string Kind { get; }
    public string? DependencyId { get; }
    public bool IsRoot { get; }
    public bool IsDependency { get; }
    public bool IsGroup { get; }
    public bool IsPlaceholder { get; }
    public bool IsDirectory => Kind == "directory" || IsGroup;
    public bool IsFile => Kind == "file";
    public bool CanRemove => IsRoot && IsDependency;
    public bool IsDependencyRoot => IsRoot && IsDependency;
    public bool HasPathSubtitle => !string.IsNullOrWhiteSpace(Path) && Path != ".";
    public double ExpansionRotation => IsExpanded ? 90 : 0;
    public bool IsLoading { get => _isLoading; set => SetProperty(ref _isLoading, value); }
    public bool IsLoaded { get => _isLoaded; set => SetProperty(ref _isLoaded, value); }
    public bool IsExpanded
    {
        get => _isExpanded;
        set
        {
            if (!SetProperty(ref _isExpanded, value)) return;
            OnPropertyChanged(nameof(ExpansionRotation));
        }
    }
    public ObservableCollection<ExplorerNode> Children { get; } = [];

    private static ExplorerNode Placeholder() => new();
}
