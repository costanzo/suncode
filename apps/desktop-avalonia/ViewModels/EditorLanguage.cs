namespace SunCode.Desktop.ViewModels;

internal sealed record EditorLanguage(string DisplayName, string GrammarExtension)
{
    public static EditorLanguage FromFileName(string fileName)
    {
        var extension = Path.GetExtension(fileName).ToLowerInvariant();
        return extension switch
        {
            ".axaml" => new("AXAML", ".xml"),
            ".xaml" => new("XAML", ".xml"),
            ".csproj" or ".props" or ".targets" => new("MSBuild", ".xml"),
            ".cs" => new("C#", ".cs"),
            ".rs" => new("Rust", ".rs"),
            ".js" => new("JavaScript", ".js"),
            ".jsx" => new("JavaScript JSX", ".jsx"),
            ".ts" => new("TypeScript", ".ts"),
            ".tsx" => new("TypeScript JSX", ".tsx"),
            ".json" => new("JSON", ".json"),
            ".md" or ".markdown" => new("Markdown", ".md"),
            ".py" => new("Python", ".py"),
            ".go" => new("Go", ".go"),
            ".java" => new("Java", ".java"),
            ".html" or ".htm" => new("HTML", ".html"),
            ".css" => new("CSS", ".css"),
            ".scss" => new("SCSS", ".scss"),
            ".sh" => new("Shell", ".sh"),
            ".toml" => new("TOML", ".toml"),
            ".yaml" or ".yml" => new("YAML", ".yaml"),
            ".xml" => new("XML", ".xml"),
            _ => new("Plain text", string.Empty)
        };
    }
}
