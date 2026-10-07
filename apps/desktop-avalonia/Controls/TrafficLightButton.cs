using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using SvgControl = Avalonia.Svg.Skia.Svg;

namespace SunCode.Desktop.Controls;

public enum TrafficLightKind
{
    Close,
    Minimize,
    Maximize
}

public enum TrafficLightState
{
    Normal,
    Hover,
    Press
}

// macOS-style window control for the Workspace title bar. It mirrors the
// design-system TrafficLights specimen: hover and press swap the light's
// asset, a press only shows while the pointer is still over the light, and an
// inactive window shows the shared muted asset for every light regardless of
// pointer state. The button chrome itself stays transparent in every state.
public sealed class TrafficLightButton : Button
{
    private const string AssetRoot = "/Assets/traffic-lights/";

    public static readonly StyledProperty<TrafficLightKind> KindProperty =
        AvaloniaProperty.Register<TrafficLightButton, TrafficLightKind>(nameof(Kind));

    public static readonly StyledProperty<bool> IsWindowActiveProperty =
        AvaloniaProperty.Register<TrafficLightButton, bool>(nameof(IsWindowActive), true);

    private SvgControl? _light;

    public TrafficLightKind Kind
    {
        get => GetValue(KindProperty);
        set => SetValue(KindProperty, value);
    }

    public bool IsWindowActive
    {
        get => GetValue(IsWindowActiveProperty);
        set => SetValue(IsWindowActiveProperty, value);
    }

    internal TrafficLightState State =>
        IsPointerOver ? IsPressed ? TrafficLightState.Press : TrafficLightState.Hover : TrafficLightState.Normal;

    internal string AssetPath => AssetRoot + GetAsset(Kind, State, IsWindowActive);

    internal static string GetAsset(TrafficLightKind kind, TrafficLightState state, bool isWindowActive = true)
    {
        if (!isWindowActive) return "0-all-three-nofocus.svg";

        return (kind, state) switch
        {
            (TrafficLightKind.Close, TrafficLightState.Hover) => "2-close-2-hover.svg",
            (TrafficLightKind.Close, TrafficLightState.Press) => "2-close-3-press.svg",
            (TrafficLightKind.Close, _) => "1-close-1-normal.svg",
            (TrafficLightKind.Minimize, TrafficLightState.Hover) => "2-minimize-2-hover.svg",
            (TrafficLightKind.Minimize, TrafficLightState.Press) => "2-minimize-3-press.svg",
            (TrafficLightKind.Minimize, _) => "2-minimize-1-normal.svg",
            (TrafficLightKind.Maximize, TrafficLightState.Hover) => "3-maximize-2-hover.svg",
            (TrafficLightKind.Maximize, TrafficLightState.Press) => "3-maximize-3-press.svg",
            _ => "3-maximize-1-normal.svg"
        };
    }

    protected override void OnApplyTemplate(TemplateAppliedEventArgs e)
    {
        base.OnApplyTemplate(e);
        _light = e.NameScope.Find<SvgControl>("PART_Light");
        UpdateLight();
    }

    protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
    {
        base.OnPropertyChanged(change);
        if (change.Property == KindProperty ||
            change.Property == IsWindowActiveProperty ||
            change.Property == IsPointerOverProperty ||
            change.Property == IsPressedProperty)
        {
            PseudoClasses.Set(":window-inactive", !IsWindowActive);
            UpdateLight();
        }
    }

    private void UpdateLight()
    {
        if (_light is not null && _light.Path != AssetPath) _light.Path = AssetPath;
    }
}
