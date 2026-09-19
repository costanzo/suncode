using Avalonia.Controls;
using Avalonia.Controls.Shapes;
using Avalonia.Interactivity;

namespace SunCode.Desktop.Views.Settings.Controls;

public sealed partial class ComputerUseSettingsControl : UserControl
{
    public event EventHandler<RoutedEventArgs>? EnabledChanged;
    public event EventHandler<RoutedEventArgs>? EmergencyStopRequested;

    public ComputerUseSettingsControl() => InitializeComponent();

    public ToggleSwitch EnabledToggleControl => EnabledToggle;
    public Ellipse ModelStatusDotControl => ModelStatusDot;
    public TextBlock ModelSupportTextControl => ModelSupportText;
    public TextBlock ModelNameTextControl => ModelNameText;
    public Ellipse BackendStatusDotControl => BackendStatusDot;
    public TextBlock BackendStateTextControl => BackendStateText;
    public TextBlock TargetDisplayTextControl => TargetDisplayText;
    public Ellipse CapturePermissionDotControl => CapturePermissionDot;
    public TextBlock CapturePermissionTextControl => CapturePermissionText;
    public Ellipse InputPermissionDotControl => InputPermissionDot;
    public TextBlock InputPermissionTextControl => InputPermissionText;
    public TextBlock ControlOwnerTextControl => ControlOwnerText;
    public TextBlock StatusTextControl => StatusText;
    public Button EmergencyStopButtonControl => EmergencyStopButton;

    private void OnEnabledChanged(object? sender, RoutedEventArgs e) => EnabledChanged?.Invoke(this, e);
    private void OnEmergencyStop(object? sender, RoutedEventArgs e) => EmergencyStopRequested?.Invoke(this, e);
}
