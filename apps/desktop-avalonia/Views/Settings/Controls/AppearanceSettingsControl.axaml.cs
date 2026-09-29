using Avalonia.Controls;
using SunCode.Desktop.Controls;

namespace SunCode.Desktop.Views.Settings.Controls;

public sealed partial class AppearanceSettingsControl : UserControl
{
    public event EventHandler<SelectionChangedEventArgs>? ThemeChanged;
    public event EventHandler<SelectionChangedEventArgs>? LanguageChanged;

    public SCFlatComboBox ThemeSelectorControl => ThemeSelector;
    public SCFlatComboBox LanguageSelectorControl => LanguageSelector;

    public AppearanceSettingsControl()
    {
        InitializeComponent();
    }

    private void OnThemeChanged(object? sender, SelectionChangedEventArgs e) =>
        ThemeChanged?.Invoke(this, e);

    private void OnLanguageChanged(object? sender, SelectionChangedEventArgs e) =>
        LanguageChanged?.Invoke(this, e);
}
