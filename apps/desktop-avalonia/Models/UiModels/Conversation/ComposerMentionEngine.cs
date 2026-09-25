using System.IO;
using System.Text.Json;

namespace SunCode.Desktop.Models;

public sealed record ExplorerDragPayload(
    string Name,
    string Path,
    string? DependencyId,
    string Kind)
{
    public static ExplorerDragPayload FromNode(ExplorerNode node) =>
        new(node.Name, node.Path, node.DependencyId, node.Kind);
    
    public bool IsFile => Kind == "file";
    
    public string Serialize() => JsonSerializer.Serialize(this);

    public static ExplorerDragPayload? Deserialize(string? text)
    {
        if (string.IsNullOrWhiteSpace(text)) return null;
        try
        {
            return JsonSerializer.Deserialize<ExplorerDragPayload>(text);
        }
        catch (JsonException)
        {
            return null;
        }
    }
}

public static class ComposerMentionEngine
{
    public static string FormatExplorerReference(ExplorerNode node)
    {
        if (node.DependencyId is { Length: > 0 } dependencyId)
        {
            var relative = NormalizeRelative(node.Path);
            return relative.Length == 0
                ? $"@dependency:{dependencyId}"
                : $"@dependency:{dependencyId}/{relative}";
        }
        
        var path = NormalizeRelative(node.Path);
        return path.Length == 0 ? "@" : "@" + path;
    }

    public static string FormatLocalReference(string? projectCanonicalRoot, string absolutePath)
    {
        if (!string.IsNullOrWhiteSpace(projectCanonicalRoot))
        {
            var root = Path.GetFullPath(projectCanonicalRoot);
            var full = Path.GetFullPath(absolutePath);
            if (full.StartsWith(root + Path.DirectorySeparatorChar, StringComparison.Ordinal) 
                && TryRelativeTo(root, full, out var relative))
            {
                return "@" + relative;
            }
        }
        return "@" + absolutePath;
    }

    private static string NormalizeRelative(string path) =>
        string.IsNullOrEmpty(path) || path == "." ? string.Empty : path;

    private static bool TryRelativeTo(string root, string full, out string relative)
    {
        relative = string.Empty;
        var rootUri = new System.Uri(root + Path.DirectorySeparatorChar);
        var fullUri = new System.Uri(full);
        if (rootUri.Scheme != fullUri.Scheme) return false; // different schemes, e.g. file vs. http
        var difference = rootUri.MakeRelativeUri(fullUri).ToString();
        relative = difference.Replace('/', Path.DirectorySeparatorChar);
        return relative.Length > 0;
    }
}