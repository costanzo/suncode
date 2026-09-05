using Avalonia.Controls;
using Avalonia.Interactivity;
using SunCode.Desktop.Controls;

namespace SunCode.Desktop.Views.Settings.Controls;

public sealed partial class DefaultsSettingsControl : UserControl
{
    public event EventHandler<SelectionChangedEventArgs>? DefaultModelChanged;
    public event EventHandler<RoutedEventArgs>? SaveToolCallLimitRequested;

    public SCFlatComboBox ModelSelector => DefaultModelSelector;
    public NumericUpDown ToolCallLimit => ToolCallLimitInput;
    public Button SaveToolCallLimitButtonControl => SaveToolCallLimitButton;
    public TextBlock ToolCallLimitScopeText => ToolCallLimitScope;
    public TextBlock ToolCallLimitStatusText => ToolCallLimitStatus;

    public DefaultsSettingsControl()
    {
        InitializeComponent();
    }

    private void OnDefaultModelChanged(object? sender, SelectionChangedEventArgs e) =>
        DefaultModelChanged?.Invoke(this, e);

    private void OnSaveToolCallLimit(object? sender, RoutedEventArgs e) =>
        SaveToolCallLimitRequested?.Invoke(this, e);
}
