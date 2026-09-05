using Avalonia.Controls;
using SunCode.Desktop.Controls;

namespace SunCode.Desktop.Views.Settings.Controls;

public sealed partial class AppearanceSettingsControl : UserControl
{
    public event EventHandler<SelectionChangedEventArgs>? ThemeChanged;

    public SCFlatComboBox ThemeSelectorControl => ThemeSelector;

    public AppearanceSettingsControl()
    {
        InitializeComponent();
    }

    private void OnThemeChanged(object? sender, SelectionChangedEventArgs e) =>
        ThemeChanged?.Invoke(this, e);
}
