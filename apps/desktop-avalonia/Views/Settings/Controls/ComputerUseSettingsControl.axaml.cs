using Avalonia.Controls;
using Avalonia.Controls.Shapes;
using Avalonia.Interactivity;

namespace SunCode.Desktop.Views.Settings.Controls;

public sealed partial class ComputerUseSettingsControl : UserControl
{
    public event EventHandler<RoutedEventArgs>? EnabledChanged;
    public event EventHandler<RoutedEventArgs>? CapturePermissionRequested;
    public event EventHandler<RoutedEventArgs>? InputPermissionRequested;
    public event EventHandler<RoutedEventArgs>? TakeControlRequested;
    public event EventHandler<RoutedEventArgs>? ReturnControlRequested;
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
    public Button CapturePermissionButtonControl => CapturePermissionButton;
    public Ellipse InputPermissionDotControl => InputPermissionDot;
    public TextBlock InputPermissionTextControl => InputPermissionText;
    public Button InputPermissionButtonControl => InputPermissionButton;
    public TextBlock ControlOwnerTextControl => ControlOwnerText;
    public Button TakeControlButtonControl => TakeControlButton;
    public Button ReturnControlButtonControl => ReturnControlButton;
    public TextBlock StatusTextControl => StatusText;
    public Button EmergencyStopButtonControl => EmergencyStopButton;

    private void OnEnabledChanged(object? sender, RoutedEventArgs e) => EnabledChanged?.Invoke(this, e);
    private void OnRequestCapturePermission(object? sender, RoutedEventArgs e) => CapturePermissionRequested?.Invoke(this, e);
    private void OnRequestInputPermission(object? sender, RoutedEventArgs e) => InputPermissionRequested?.Invoke(this, e);
    private void OnTakeControl(object? sender, RoutedEventArgs e) => TakeControlRequested?.Invoke(this, e);
    private void OnReturnControl(object? sender, RoutedEventArgs e) => ReturnControlRequested?.Invoke(this, e);
    private void OnEmergencyStop(object? sender, RoutedEventArgs e) => EmergencyStopRequested?.Invoke(this, e);
}
