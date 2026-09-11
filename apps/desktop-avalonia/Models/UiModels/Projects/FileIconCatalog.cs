namespace SunCode.Desktop.Models;

internal sealed record FileIconDefinition(string Glyph, string Color);

internal static class FileIconCatalog
{
    private static readonly FileIconDefinition Default = Definition("\uE023", "default");

    private static readonly IReadOnlyDictionary<string, FileIconDefinition> Definitions =
        new Dictionary<string, FileIconDefinition>(StringComparer.OrdinalIgnoreCase)
        {
            ["javascript"] = Definition("\uE051", "yellow"),
            ["typescript"] = Definition("\uE099", "blue"),
            ["react"] = Definition("\uE07D", "blue"),
            ["markdown"] = Definition("\uE060", "blue"),
            ["info"] = Definition("\uE04D", "blue"),
            ["json"] = Definition("\uE055", "yellow"),
            ["tsconfig"] = Definition("\uE097", "blue"),
            ["yaml"] = Definition("\uE0A7", "purple"),
            ["html"] = Definition("\uE048", "orange"),
            ["css"] = Definition("\uE01D", "blue"),
            ["config"] = Definition("\uE019", "muted"),
            ["rust"] = Definition("\uE082", "muted"),
            ["python"] = Definition("\uE07B", "blue"),
            ["csharp"] = Definition("\uE00B", "blue"),
            ["shell"] = Definition("\uE089", "green"),
            ["docker"] = Definition("\uE025", "blue"),
            ["git"] = Definition("\uE034", "ink"),
            ["image"] = Definition("\uE04C", "purple"),
            ["svg"] = Definition("\uE091", "purple"),
            ["audio"] = Definition("\uE005", "purple"),
            ["video"] = Definition("\uE09B", "pink"),
            ["pdf"] = Definition("\uE06D", "red"),
            ["database"] = Definition("\uE022", "pink"),
            ["archive"] = Definition("\uE0A9", "muted"),
            ["license"] = Definition("\uE05A", "yellow"),
            ["todo"] = Definition("\uE096", "todo"),
        };

    private static readonly IReadOnlyDictionary<string, string> ExactFileNames =
        new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase)
        {
            ["readme"] = "info",
            ["readme.md"] = "info",
            ["readme.txt"] = "info",
            ["tsconfig.json"] = "tsconfig",
            ["dockerfile"] = "docker",
            [".gitignore"] = "git",
            [".gitattributes"] = "git",
            [".gitmodules"] = "git",
            ["license"] = "license",
            ["license.md"] = "license",
            ["license.txt"] = "license",
            ["todo.md"] = "todo",
        };

    private static readonly IReadOnlyDictionary<string, string> CompoundExtensions =
        new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase)
        {
            [".d.ts"] = "typescript",
            [".test.js"] = "javascript",
            [".spec.js"] = "javascript",
            [".test.ts"] = "typescript",
            [".spec.ts"] = "typescript",
            [".test.jsx"] = "react",
            [".spec.jsx"] = "react",
            [".test.tsx"] = "react",
            [".spec.tsx"] = "react",
        };

    private static readonly IReadOnlyDictionary<string, string> Extensions =
        new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase)
        {
            [".js"] = "javascript",
            [".mjs"] = "javascript",
            [".cjs"] = "javascript",
            [".ts"] = "typescript",
            [".jsx"] = "react",
            [".tsx"] = "react",
            [".rs"] = "rust",
            [".py"] = "python",
            [".cs"] = "csharp",
            [".c"] = "csharp",
            [".cpp"] = "csharp",
            [".cc"] = "csharp",
            [".h"] = "csharp",
            [".hpp"] = "csharp",
            [".html"] = "html",
            [".htm"] = "html",
            [".axaml"] = "html",
            [".css"] = "css",
            [".scss"] = "css",
            [".sass"] = "css",
            [".md"] = "markdown",
            [".markdown"] = "markdown",
            [".json"] = "json",
            [".jsonc"] = "json",
            [".yaml"] = "yaml",
            [".yml"] = "yaml",
            [".toml"] = "config",
            [".env"] = "config",
            [".ini"] = "config",
            [".xml"] = "config",
            [".xaml"] = "config",
            [".sh"] = "shell",
            [".bash"] = "shell",
            [".zsh"] = "shell",
            [".fish"] = "shell",
            [".sql"] = "database",
            [".png"] = "image",
            [".jpg"] = "image",
            [".jpeg"] = "image",
            [".webp"] = "image",
            [".gif"] = "image",
            [".ico"] = "image",
            [".svg"] = "svg",
            [".mp3"] = "audio",
            [".wav"] = "audio",
            [".ogg"] = "audio",
            [".flac"] = "audio",
            [".mp4"] = "video",
            [".webm"] = "video",
            [".mov"] = "video",
            [".pdf"] = "pdf",
            [".zip"] = "archive",
            [".jar"] = "archive",
            [".tar"] = "archive",
            [".gz"] = "archive",
            [".7z"] = "archive",
        };

    public static FileIconDefinition Resolve(string fileName)
    {
        var normalizedName = Path.GetFileName(fileName);
        if (ExactFileNames.TryGetValue(normalizedName, out var exactType))
            return Definitions[exactType];

        foreach (var (compoundExtension, compoundType) in CompoundExtensions)
        {
            if (normalizedName.EndsWith(compoundExtension, StringComparison.OrdinalIgnoreCase))
                return Definitions[compoundType];
        }

        var extension = Path.GetExtension(normalizedName);
        return Extensions.TryGetValue(extension, out var extensionType)
            ? Definitions[extensionType]
            : Default;
    }

    private static FileIconDefinition Definition(string glyph, string color) => new(glyph, color);
}
