using SunCode.Desktop.Controls;
using SunCode.Desktop.Infrastructure;
using static SunCode.Desktop.ViewModels.SettingsFormatting;

namespace SunCode.Desktop.ViewModels;

// Appearance page: theme and interface language. A user selection saves
// immediately; Load and RefreshLocalizedText only sync the selectors.
public sealed class AppearanceSettingsViewModel : ObservableObject
{
    public static IReadOnlyList<SCComboBoxItem> LanguageOptions { get; } =
    [
        new("English", LocalizationService.DefaultLocale),
        new("简体中文", LocalizationService.SimplifiedChineseLocale)
    ];

    private readonly IViewModelHost _host;
    private readonly AppSettingsViewModel _settings;
    private IReadOnlyList<SCComboBoxItem> _themeOptions = [];
    private SCComboBoxItem? _selectedTheme;
    private SCComboBoxItem? _selectedLanguage;

    internal AppearanceSettingsViewModel(IViewModelHost host, AppSettingsViewModel settings)
    {
        _host = host;
        _settings = settings;
    }

    public IReadOnlyList<SCComboBoxItem> ThemeOptions { get => _themeOptions; private set => SetProperty(ref _themeOptions, value); }

    public SCComboBoxItem? SelectedTheme
    {
        get => _selectedTheme;
        set
        {
            if (SetProperty(ref _selectedTheme, value) && value?.Value is string mode)
                _ = _settings.SaveThemeAsync(_host, mode);
        }
    }

    public SCComboBoxItem? SelectedLanguage
    {
        get => _selectedLanguage;
        set
        {
            if (SetProperty(ref _selectedLanguage, value) && value?.Value is string locale)
                _ = _settings.SaveLanguageAsync(_host, locale);
        }
    }

    public static IReadOnlyList<SCComboBoxItem> CreateThemeOptions() =>
    [
        new(L("LocDark", "Dark"), "dark"),
        new(L("LocLight", "Light"), "light")
    ];

    public void Load() => RefreshLocalizedText();

    // Rebuilds localized theme labels and re-selects the saved values.
    public void RefreshLocalizedText()
    {
        ThemeOptions = CreateThemeOptions();
        _selectedTheme = ThemeOptions.FirstOrDefault(item => Equals(item.Value, _settings.ThemeMode));
        _selectedLanguage = LanguageOptions.FirstOrDefault(item => Equals(item.Value, _settings.Language));
        OnPropertyChanged(nameof(SelectedTheme));
        OnPropertyChanged(nameof(SelectedLanguage));
    }
}
