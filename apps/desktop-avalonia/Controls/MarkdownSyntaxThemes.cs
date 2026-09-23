using LiveMarkdown.Avalonia;
using TextMateSharp.Themes;

namespace SunCode.Desktop.Controls;

public class MarkdownSyntaxThemes
{
    internal const string DarkName = "SunCode.Markdown.Dark";
    internal const string LightName = "SunCode.Markdown.Light";

    private static bool _registered;

    internal static void Register()
    {
        if (_registered) return;
        
        SyntaxHighlighting.RegisterCustomTheme(DarkName, CreateDarkTheme());
        SyntaxHighlighting.RegisterCustomTheme(LightName, CreateLightTheme());
        _registered = true;
    }

    private static IRawTheme CreateDarkTheme() => new RawTheme(
        DarkName,
        "#a7afb9",
        "#7f8994",
        "#9eb9d0",
        "#d6c18d",
        "#a9c7a5",
        "#c7a5c7",
        "#9fc4c8",
        "#c7b4d8",
        "#c4d3a3",
        "#d9ac8c",
        "#a7afb9"
    );
    
    private static IRawTheme CreateLightTheme() => new RawTheme(
        LightName,
        "#44515e",
        "#65717d",
        "#315f86",
        "#7b5b16",
        "#35683c",
        "#764b7d",
        "#21636a",
        "#664b83",
        "#4c681e",
        "#8a4b25",
        "#44515e"
    );
        

    private sealed class RawTheme : IRawTheme
    {
        private readonly string _name;
        private readonly ICollection<IRawThemeSetting> _tokenColors;

        internal RawTheme(
            string name,
            string foreground,
            string comment,
            string keyword,
            string function,
            string text,
            string number,
            string property,
            string variable,
            string type,
            string annotation,
            string punctuation)
        {
            _name = name;
            _tokenColors =
            [
                new RawThemeSetting("Default", string.Empty, foreground),
                new RawThemeSetting("Comment", "comment", comment, "italic"),
                new RawThemeSetting("Keyword", "keyword, storage", keyword),
                new RawThemeSetting("Function", "entity.name.function, support.function, meta.function-call", function),
                new RawThemeSetting("String", "string, constant.character, entity.other.attribute-name", text),
                new RawThemeSetting("Number", "constant.numeric, constant.language.boolean, constant.language.null",
                    number),
                new RawThemeSetting("Property", "variable.other.property, support.type.property-name, entity.name.tag",
                    property),
                new RawThemeSetting("Variable", "variable, meta.parameter, entity.name.label", variable),
                new RawThemeSetting("Type", "entity.name.type, entity.name.class, support.type, support.class", type),
                new RawThemeSetting("Annotation",
                    "meta.annotation, storage.type.annotation, entity.name.function.preprocessor", annotation),
                new RawThemeSetting("Punctuation", "punctuation, keyword.operator", punctuation)
            ];
        }
        
        public string GetName() => _name;

        public string GetInclude() => string.Empty;
        
        public ICollection<IRawThemeSetting> GetTokenColors() => _tokenColors;
        
        public ICollection<IRawThemeSetting> GetSettings() => [];
        
        public ICollection<KeyValuePair<string, object>> GetGuiColors() => [];
    }
    
    private sealed class RawThemeSetting(
        string name,
        object scope,
        string foreground,
        string fontStyle = "") : IRawThemeSetting
    {
        public string GetName() => name;
        
        public object GetScope() => scope;
        
        public IThemeSetting GetSetting() => new ThemeSetting(foreground, fontStyle);
    }

    private sealed class ThemeSetting(string foreground, string fontStyle) : IThemeSetting
    {
        public object GetFontStyle() => fontStyle;
        
        public string GetBackground() => string.Empty;
        
        public string GetForeground() => foreground;
        
    }
}