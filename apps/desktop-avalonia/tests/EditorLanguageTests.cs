using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Tests;

public sealed class EditorLanguageTests
{
    [Theory]
    [InlineData("MainWindow.axaml", "AXAML", ".xml")]
    [InlineData("Control.xaml", "XAML", ".xml")]
    [InlineData("SunCode.Desktop.csproj", "MSBuild", ".xml")]
    [InlineData("Directory.props", "MSBuild", ".xml")]
    [InlineData("lib.rs", "Rust", ".rs")]
    [InlineData("component.tsx", "TypeScript JSX", ".tsx")]
    [InlineData("settings.yaml", "YAML", ".yaml")]
    [InlineData("LICENSE", "Plain text", "")]
    public void Maps_file_names_to_labels_and_textmate_grammar_extensions(
        string fileName,
        string displayName,
        string grammarExtension)
    {
        var language = EditorLanguage.FromFileName(fileName);

        Assert.Equal(displayName, language.DisplayName);
        Assert.Equal(grammarExtension, language.GrammarExtension);
    }
}
