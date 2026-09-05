using Avalonia.Controls;
using Avalonia.Interactivity;
using SunCode.Desktop.Controls;

namespace SunCode.Desktop.Views.Settings.Controls;

public sealed partial class LoggingSettingsControl : UserControl
{
    public event EventHandler<SelectionChangedEventArgs>? LogLevelChanged;
    public event EventHandler<RoutedEventArgs>? SaveLoggingRequested;
    public event EventHandler<RoutedEventArgs>? SaveImageDirectoryRequested;
    public SCFlatComboBox LogLevelSelectorControl => LogLevelSelector;
    public SCFileSelector LogDirectoryInputControl => LogDirectoryInput;
    public SCFileSelector ImageDirectoryInputControl => ImageDirectoryInput;
    public SCNumericInput LogMaxMegabytesInputControl => LogMaxMegabytesInput;
    public SCNumericInput LogRetentionInputControl => LogRetentionInput;
    public TextBlock LoggingStatusText => LoggingStatus;
    public TextBlock ImageDirectoryStatusText => ImageDirectoryStatus;
    public LoggingSettingsControl() => InitializeComponent();
    private void OnLogLevelChanged(object? sender, SelectionChangedEventArgs e) => LogLevelChanged?.Invoke(this, e);
    private void OnSaveLogging(object? sender, RoutedEventArgs e) => SaveLoggingRequested?.Invoke(this, e);
    private void OnSaveImageDirectory(object? sender, RoutedEventArgs e) => SaveImageDirectoryRequested?.Invoke(this, e);
}
