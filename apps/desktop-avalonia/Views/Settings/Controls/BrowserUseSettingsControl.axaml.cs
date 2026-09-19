using Avalonia.Controls;
using Avalonia.Controls.Shapes;
using Avalonia.Interactivity;

namespace SunCode.Desktop.Views.Settings.Controls;

public sealed partial class BrowserUseSettingsControl : UserControl
{
    public event EventHandler<RoutedEventArgs>? EnabledChanged;
    public event EventHandler<RoutedEventArgs>? VerifyRequested;
    public event EventHandler<RoutedEventArgs>? StartRequested;
    public event EventHandler<RoutedEventArgs>? TakeControlRequested;
    public event EventHandler<RoutedEventArgs>? ReturnControlRequested;
    public event EventHandler<RoutedEventArgs>? RestartRequested;
    public event EventHandler<RoutedEventArgs>? StopRequested;
    public event EventHandler<RoutedEventArgs>? ClearRequested;
    public event EventHandler<RoutedEventArgs>? CopyNodePathRequested;
    public event EventHandler<RoutedEventArgs>? CopyChromiumPathRequested;
    public event EventHandler<RoutedEventArgs>? CopyProfilePathRequested;

    public BrowserUseSettingsControl() => InitializeComponent();

    public ToggleSwitch EnabledToggleControl => EnabledToggle;
    public TextBlock InstallationStateTextControl => InstallationStateText;
    public Ellipse InstallationStatusDotControl => InstallationStatusDot;
    public TextBlock RuntimeStateTextControl => RuntimeStateText;
    public Ellipse RuntimeStatusDotControl => RuntimeStatusDot;
    public TextBlock RuntimeScopeTextControl => RuntimeScopeText;
    public TextBlock TargetTextControl => TargetText;
    public TextBlock NodeVersionTextControl => NodeVersionText;
    public TextBlock NodePathTextControl => NodePathText;
    public TextBlock PlaywrightVersionTextControl => PlaywrightVersionText;
    public TextBlock ChromiumVersionTextControl => ChromiumVersionText;
    public TextBlock ChromiumPathTextControl => ChromiumPathText;
    public TextBlock WorkerProtocolTextControl => WorkerProtocolText;
    public TextBlock IntegrityTextControl => IntegrityText;
    public Border ProjectSectionControl => ProjectSection;
    public Border NoProjectSectionControl => NoProjectSection;
    public TextBlock ProfilePathTextControl => ProfilePathText;
    public TextBlock ProfileUsageTextControl => ProfileUsageText;
    public TextBlock VisibilityTextControl => VisibilityText;
    public TextBlock ControlTitleTextControl => ControlTitleText;
    public TextBlock ControlHintTextControl => ControlHintText;
    public TextBlock ErrorTextControl => ErrorText;
    public TextBlock StatusTextControl => StatusText;
    public Button CopyNodePathButtonControl => CopyNodePathButton;
    public Button CopyChromiumPathButtonControl => CopyChromiumPathButton;
    public Button CopyProfilePathButtonControl => CopyProfilePathButton;
    public Button VerifyButtonControl => VerifyButton;
    public Button StartButtonControl => StartButton;
    public Button TakeControlButtonControl => TakeControlButton;
    public Button ReturnControlButtonControl => ReturnControlButton;
    public Button RestartButtonControl => RestartButton;
    public Button StopButtonControl => StopButton;
    public Button ClearButtonControl => ClearButton;

    private void OnEnabledChanged(object? sender, RoutedEventArgs e) => EnabledChanged?.Invoke(this, e);
    private void OnVerify(object? sender, RoutedEventArgs e) => VerifyRequested?.Invoke(this, e);
    private void OnStart(object? sender, RoutedEventArgs e) => StartRequested?.Invoke(this, e);
    private void OnTakeControl(object? sender, RoutedEventArgs e) => TakeControlRequested?.Invoke(this, e);
    private void OnReturnControl(object? sender, RoutedEventArgs e) => ReturnControlRequested?.Invoke(this, e);
    private void OnRestart(object? sender, RoutedEventArgs e) => RestartRequested?.Invoke(this, e);
    private void OnStop(object? sender, RoutedEventArgs e) => StopRequested?.Invoke(this, e);
    private void OnClear(object? sender, RoutedEventArgs e) => ClearRequested?.Invoke(this, e);
    private void OnCopyNodePath(object? sender, RoutedEventArgs e) => CopyNodePathRequested?.Invoke(this, e);
    private void OnCopyChromiumPath(object? sender, RoutedEventArgs e) => CopyChromiumPathRequested?.Invoke(this, e);
    private void OnCopyProfilePath(object? sender, RoutedEventArgs e) => CopyProfilePathRequested?.Invoke(this, e);
}
