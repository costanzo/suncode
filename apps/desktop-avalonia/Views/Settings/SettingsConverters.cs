using System.Globalization;
using Avalonia;
using Avalonia.Data.Converters;

namespace SunCode.Desktop.Views.Settings;

public static class SettingsConverters
{
    // True when the bound value equals ConverterParameter, e.g. the selected
    // Settings page against the page a panel represents.
    public static readonly IValueConverter EqualsParameter = new EqualsParameterConverter();

    // Chevron angle for an expandable navigation group.
    public static readonly IValueConverter ExpandedAngle =
        new FuncValueConverter<bool, double>(expanded => expanded ? 90 : 0);

    private sealed class EqualsParameterConverter : IValueConverter
    {
        public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture) =>
            Equals(value, parameter);

        public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture) =>
            throw new NotSupportedException();
    }
}

// Sets the `selected` class on a navigation entry while its Key equals the
// Current highlighted key. Both sides bind, which a converter parameter cannot.
public static class NavigationHighlight
{
    public static readonly AttachedProperty<object?> KeyProperty =
        AvaloniaProperty.RegisterAttached<StyledElement, object?>("Key", typeof(NavigationHighlight));

    public static readonly AttachedProperty<object?> CurrentProperty =
        AvaloniaProperty.RegisterAttached<StyledElement, object?>("Current", typeof(NavigationHighlight));

    static NavigationHighlight()
    {
        KeyProperty.Changed.AddClassHandler<StyledElement>((element, _) => Apply(element));
        CurrentProperty.Changed.AddClassHandler<StyledElement>((element, _) => Apply(element));
    }

    public static object? GetKey(StyledElement element) => element.GetValue(KeyProperty);
    public static void SetKey(StyledElement element, object? value) => element.SetValue(KeyProperty, value);
    public static object? GetCurrent(StyledElement element) => element.GetValue(CurrentProperty);
    public static void SetCurrent(StyledElement element, object? value) => element.SetValue(CurrentProperty, value);

    private static void Apply(StyledElement element)
    {
        var key = GetKey(element);
        element.Classes.Set("selected", key is not null && Equals(key, GetCurrent(element)));
    }
}
