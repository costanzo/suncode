using Avalonia;
using Avalonia.Controls;
using Avalonia.Markup.Xaml.Styling;

namespace SunCode.Desktop.Infrastructure;

/// <summary>
/// Owns client-side presentation resources. Persisted locale values are deliberately
/// small BCP-47 identifiers so adding another language does not change the settings API.
/// </summary>
public sealed class LocalizationService
{
    public const string DefaultLocale = "en-US";
    public const string SimplifiedChineseLocale = "zh-CN";

    private static readonly IReadOnlyList<LocaleOption> Catalog =
    [
        new(DefaultLocale, "English"),
        new(SimplifiedChineseLocale, "简体中文")
    ];

    private readonly Application _application;
    private IResourceProvider? _activeDictionary;

    public LocalizationService(Application application)
    {
        _application = application;
    }

    public string CurrentLocale { get; private set; } = DefaultLocale;

    public static IReadOnlyList<LocaleOption> SupportedLocales => Catalog;

    public static string GetString(string key, string fallback = "") =>
        Application.Current?.FindResource(key) as string ?? fallback;

    public string Normalize(string? locale) =>
        Catalog.FirstOrDefault(item => string.Equals(item.Code, locale, StringComparison.OrdinalIgnoreCase))?.Code
        ?? DefaultLocale;

    public void SetLocale(string? locale)
    {
        var normalized = Normalize(locale);
        if (string.Equals(CurrentLocale, normalized, StringComparison.Ordinal)) return;

        var dictionary = new ResourceInclude(new Uri($"avares://SunCode/Resources/Localization/Strings.{normalized}.axaml"));
        var merged = _application.Resources.MergedDictionaries;
        if (_activeDictionary is not null) merged.Remove(_activeDictionary);
        merged.Insert(0, dictionary);
        _activeDictionary = dictionary;
        CurrentLocale = normalized;
    }
}

public sealed record LocaleOption(string Code, string DisplayName);
