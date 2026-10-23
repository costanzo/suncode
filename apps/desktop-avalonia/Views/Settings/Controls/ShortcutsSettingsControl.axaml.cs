using Avalonia.Controls;

namespace SunCode.Desktop.Views.Settings.Controls;

public sealed partial class ShortcutsSettingsControl : UserControl
{
    public ShortcutsSettingsControl()
    {
        InitializeComponent();
        var modifier = OperatingSystem.IsMacOS() ? "⌘" : "Ctrl";
        OpenProjectModifier.Text = modifier;
        NewSessionModifier.Text = modifier;
        CloseWindowModifier.Text = modifier;
        MinimizeWindowModifier.Text = modifier;
        OpenSettingsModifier.Text = modifier;
        ToggleNavigationModifier.Text = modifier;
        ToggleGitModifier.Text = modifier;
    }
}
