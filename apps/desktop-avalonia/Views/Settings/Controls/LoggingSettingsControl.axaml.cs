using Avalonia;
using Avalonia.Controls;
using Avalonia.Interactivity;
using SunCode.Desktop.Controls;

namespace SunCode.Desktop.Views.Settings.Controls;

public sealed partial class LoggingSettingsControl : UserControl
{
    public event EventHandler<SelectionChangedEventArgs>? LogLevelChanged;
    public event EventHandler<TextChangedEventArgs>? LogDirectoryChanged;
    public event EventHandler<AvaloniaPropertyChangedEventArgs>? LogMaxMegabytesChanged;
    public event EventHandler<AvaloniaPropertyChangedEventArgs>? LogRetentionChanged;
    public event EventHandler<RoutedEventArgs>? SaveLoggingRequested;
    public event EventHandler<TextChangedEventArgs>? ImageDirectoryChanged;
    public event EventHandler<RoutedEventArgs>? SaveImageDirectoryRequested;
    public SCFlatComboBox LogLevelSelectorControl => LogLevelSelector;
    public SCFileSelector LogDirectoryInputControl => LogDirectoryInput;
    public SCFileSelector ImageDirectoryInputControl => ImageDirectoryInput;
    public SCNumericInput LogMaxMegabytesInputControl => LogMaxMegabytesInput;
    public SCNumericInput LogRetentionInputControl => LogRetentionInput;
    public Button SaveLoggingButtonControl => SaveLoggingButton;
    public Button SaveImageDirectoryButtonControl => SaveImageDirectoryButton;
    public TextBlock LoggingStatusText => LoggingStatus;
    public TextBlock ImageDirectoryStatusText => ImageDirectoryStatus;
    public LoggingSettingsControl() => InitializeComponent();
    private void OnLogLevelChanged(object? sender, SelectionChangedEventArgs e) => LogLevelChanged?.Invoke(this, e);
    private void OnSaveLogging(object? sender, RoutedEventArgs e) => SaveLoggingRequested?.Invoke(this, e);
    private void OnSaveImageDirectory(object? sender, RoutedEventArgs e) => SaveImageDirectoryRequested?.Invoke(this, e);
    private void OnLogMaxMegabytesChanged(object? sender, AvaloniaPropertyChangedEventArgs e) => LogMaxMegabytesChanged?.Invoke(this, e);
    private void OnLogRetentionChanged(object? sender, AvaloniaPropertyChangedEventArgs e) => LogRetentionChanged?.Invoke(this, e);
    private void OnLogDirectoryChanged(object? sender, TextChangedEventArgs e) => LogDirectoryChanged?.Invoke(this, e);
    private void OnImageDirectoryChanged(object? sender, TextChangedEventArgs e) => ImageDirectoryChanged?.Invoke(this, e);
}
