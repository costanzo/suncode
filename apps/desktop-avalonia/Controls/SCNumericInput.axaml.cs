using Avalonia;
using Avalonia.Controls;

namespace SunCode.Desktop.Controls;

/// <summary>
/// Design-system numeric field with optional unit text and Fluent's native
/// NumericUpDown spinner/editing behavior.
/// </summary>
public sealed partial class SCNumericInput : UserControl
{
    public static readonly StyledProperty<decimal?> ValueProperty =
        AvaloniaProperty.Register<SCNumericInput, decimal?>(nameof(Value), defaultBindingMode: Avalonia.Data.BindingMode.TwoWay);

    public static readonly StyledProperty<decimal> MinimumProperty =
        AvaloniaProperty.Register<SCNumericInput, decimal>(nameof(Minimum), decimal.MinValue);

    public static readonly StyledProperty<decimal> MaximumProperty =
        AvaloniaProperty.Register<SCNumericInput, decimal>(nameof(Maximum), decimal.MaxValue);

    public static readonly StyledProperty<decimal> IncrementProperty =
        AvaloniaProperty.Register<SCNumericInput, decimal>(nameof(Increment), 1m);

    public static readonly StyledProperty<string> FormatStringProperty =
        AvaloniaProperty.Register<SCNumericInput, string>(nameof(FormatString), string.Empty);

    public static readonly StyledProperty<string?> PlaceholderTextProperty =
        AvaloniaProperty.Register<SCNumericInput, string?>(nameof(PlaceholderText));

    public static readonly StyledProperty<string?> UnitProperty =
        AvaloniaProperty.Register<SCNumericInput, string?>(nameof(Unit));

    public static readonly StyledProperty<bool> IsReadOnlyProperty =
        AvaloniaProperty.Register<SCNumericInput, bool>(nameof(IsReadOnly));

    public static readonly StyledProperty<bool> AllowSpinProperty =
        AvaloniaProperty.Register<SCNumericInput, bool>(nameof(AllowSpin), true);

    public static readonly StyledProperty<bool> ShowButtonSpinnerProperty =
        AvaloniaProperty.Register<SCNumericInput, bool>(nameof(ShowButtonSpinner), true);

    public SCNumericInput()
    {
        InitializeComponent();
        Input.PropertyChanged += InputPropertyChanged;
        SyncInput();
    }

    public decimal? Value
    {
        get => GetValue(ValueProperty);
        set => SetValue(ValueProperty, value);
    }

    public decimal Minimum
    {
        get => GetValue(MinimumProperty);
        set => SetValue(MinimumProperty, value);
    }

    public decimal Maximum
    {
        get => GetValue(MaximumProperty);
        set => SetValue(MaximumProperty, value);
    }

    public decimal Increment
    {
        get => GetValue(IncrementProperty);
        set => SetValue(IncrementProperty, value);
    }

    public string FormatString
    {
        get => GetValue(FormatStringProperty);
        set => SetValue(FormatStringProperty, value);
    }

    public string? PlaceholderText
    {
        get => GetValue(PlaceholderTextProperty);
        set => SetValue(PlaceholderTextProperty, value);
    }

    public string? Unit
    {
        get => GetValue(UnitProperty);
        set => SetValue(UnitProperty, value);
    }

    public bool IsReadOnly
    {
        get => GetValue(IsReadOnlyProperty);
        set => SetValue(IsReadOnlyProperty, value);
    }

    public bool AllowSpin
    {
        get => GetValue(AllowSpinProperty);
        set => SetValue(AllowSpinProperty, value);
    }

    public bool ShowButtonSpinner
    {
        get => GetValue(ShowButtonSpinnerProperty);
        set => SetValue(ShowButtonSpinnerProperty, value);
    }

    protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
    {
        base.OnPropertyChanged(change);
        if (Input is null || UnitText is null) return;

        if (change.Property == ValueProperty
            || change.Property == MinimumProperty
            || change.Property == MaximumProperty
            || change.Property == IncrementProperty
            || change.Property == FormatStringProperty
            || change.Property == PlaceholderTextProperty
            || change.Property == IsReadOnlyProperty
            || change.Property == AllowSpinProperty
            || change.Property == ShowButtonSpinnerProperty)
        {
            SyncInput();
        }

        if (change.Property == UnitProperty)
        {
            UnitText.Text = Unit;
            UnitText.IsVisible = !string.IsNullOrWhiteSpace(Unit);
        }
    }

    private void InputPropertyChanged(object? sender, AvaloniaPropertyChangedEventArgs change)
    {
        if (Input is not null && change.Property == NumericUpDown.ValueProperty)
        {
            SetCurrentValue(ValueProperty, Input.Value);
        }
    }

    private void SyncInput()
    {
        if (Input is null || UnitText is null) return;

        Input.Value = Value;
        Input.Minimum = Minimum;
        Input.Maximum = Maximum;
        Input.Increment = Increment;
        Input.FormatString = FormatString;
        Input.PlaceholderText = PlaceholderText;
        Input.IsReadOnly = IsReadOnly;
        Input.AllowSpin = AllowSpin;
        Input.ShowButtonSpinner = ShowButtonSpinner;
        UnitText.Text = Unit;
        UnitText.IsVisible = !string.IsNullOrWhiteSpace(Unit);
    }
}
