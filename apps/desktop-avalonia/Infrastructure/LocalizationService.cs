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
        // App.axaml ships with the English dictionary as the bootstrap resource.
        // The first runtime locale change must replace that dictionary too; only
        // removing _activeDictionary would leave the bootstrap English values
        // competing with the selected locale.
        if (string.Equals(CurrentLocale, normalized, StringComparison.Ordinal) && _activeDictionary is not null) return;

        var dictionary = CreateDictionary(normalized);
        var merged = _application.Resources.MergedDictionaries;
        foreach (var existing in merged.OfType<ResourceInclude>().Where(IsLocalizationDictionary).ToArray())
        {
            merged.Remove(existing);
        }
        merged.Insert(0, dictionary);
        _activeDictionary = dictionary;
        CurrentLocale = normalized;
    }

    private static bool IsLocalizationDictionary(ResourceInclude include) =>
        include.Source?.OriginalString.Contains("/Resources/Localization/Strings.", StringComparison.OrdinalIgnoreCase) == true;

    internal static ResourceInclude CreateDictionary(string locale) =>
        new(new Uri("avares://SunCode"))
        {
            Source = new Uri($"avares://SunCode/Resources/Localization/Strings.{locale}.axaml")
        };
}

public sealed record LocaleOption(string Code, string DisplayName);
