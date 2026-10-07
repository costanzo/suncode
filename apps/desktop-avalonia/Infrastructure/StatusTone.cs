using Avalonia;
using Avalonia.Controls;

namespace SunCode.Desktop.Infrastructure;

// Semantic color of a settings status dot or status line. View models expose
// a tone; the view maps it to theme brushes through `tone-*` style classes so
// the color follows theme switches without code-behind.
public enum StatusTone
{
    Neutral,
    Muted,
    Success,
    Warning,
    Danger
}

public static class StatusToneProperties
{
    public static readonly AttachedProperty<StatusTone> ToneProperty =
        AvaloniaProperty.RegisterAttached<StyledElement, StatusTone>("Tone", typeof(StatusToneProperties));

    static StatusToneProperties()
    {
        ToneProperty.Changed.AddClassHandler<StyledElement>((element, _) => Apply(element));
    }

    public static StatusTone GetTone(StyledElement element) => element.GetValue(ToneProperty);

    public static void SetTone(StyledElement element, StatusTone value) => element.SetValue(ToneProperty, value);

    private static void Apply(StyledElement element)
    {
        var tone = GetTone(element);
        element.Classes.Set("tone-muted", tone == StatusTone.Muted);
        element.Classes.Set("tone-success", tone == StatusTone.Success);
        element.Classes.Set("tone-warning", tone == StatusTone.Warning);
        element.Classes.Set("tone-danger", tone == StatusTone.Danger);
    }
}
