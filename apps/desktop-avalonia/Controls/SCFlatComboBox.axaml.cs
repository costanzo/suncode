using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;

namespace SunCode.Desktop.Controls;

public sealed partial class SCFlatComboBox : UserControl
{
    public static readonly StyledProperty<IEnumerable<SCComboBoxItem>?> ItemsSourceProperty =
        AvaloniaProperty.Register<SCFlatComboBox, IEnumerable<SCComboBoxItem>?>(nameof(ItemsSource));

    public static readonly StyledProperty<SCComboBoxItem?> SelectedItemProperty =
        AvaloniaProperty.Register<SCFlatComboBox, SCComboBoxItem?>(
            nameof(SelectedItem),
            defaultBindingMode: Avalonia.Data.BindingMode.TwoWay);

    public static readonly StyledProperty<string?> PlaceholderTextProperty =
        AvaloniaProperty.Register<SCFlatComboBox, string?>(nameof(PlaceholderText));

    public static readonly StyledProperty<bool> UseMonospaceProperty =
        AvaloniaProperty.Register<SCFlatComboBox, bool>(nameof(UseMonospace));

    public static readonly DirectProperty<SCFlatComboBox, string> DisplayTextProperty =
        AvaloniaProperty.RegisterDirect<SCFlatComboBox, string>(nameof(DisplayText), control => control.DisplayText);

    private string _displayText = string.Empty;
    private bool _syncingSelection;

    public event EventHandler<SelectionChangedEventArgs>? SelectionChanged;

    public SCFlatComboBox()
    {
        InitializeComponent();
        FlatCombo.SelectionChanged += FlatSelectionChanged;
        Loaded += (_, _) => SyncView();
        SyncView();
    }

    public IEnumerable<SCComboBoxItem>? ItemsSource
    {
        get => GetValue(ItemsSourceProperty);
        set => SetValue(ItemsSourceProperty, value);
    }

    public SCComboBoxItem? SelectedItem
    {
        get => GetValue(SelectedItemProperty);
        set => SetValue(SelectedItemProperty, value);
    }

    public string? PlaceholderText
    {
        get => GetValue(PlaceholderTextProperty);
        set => SetValue(PlaceholderTextProperty, value);
    }

    public bool UseMonospace
    {
        get => GetValue(UseMonospaceProperty);
        set => SetValue(UseMonospaceProperty, value);
    }

    public string DisplayText => _displayText;

    protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
    {
        base.OnPropertyChanged(change);
        if (change.Property == ItemsSourceProperty
            || change.Property == SelectedItemProperty
            || change.Property == PlaceholderTextProperty
            || change.Property == UseMonospaceProperty)
        {
            SyncView(change.Property);
        }
    }

    private void SyncView(AvaloniaProperty? changedProperty = null)
    {
        if (FlatCombo is null || PlaceholderLabel is null) return;

        var syncItems = changedProperty is null || changedProperty == ItemsSourceProperty;
        var syncSelected = syncItems || changedProperty == SelectedItemProperty;
        if (syncItems || syncSelected)
        {
            _syncingSelection = true;
            try
            {
                if (syncItems)
                    FlatCombo.ItemsSource = ItemsSource;
                if (syncSelected)
                    FlatCombo.SelectedItem = SelectedItem;
            }
            finally
            {
                _syncingSelection = false;
            }
        }
        FlatCombo.Classes.Set("mono", UseMonospace);

        var previousText = _displayText;
        _displayText = SelectedItem?.Label ?? PlaceholderText ?? string.Empty;
        if (!string.Equals(previousText, _displayText, StringComparison.Ordinal))
        {
            RaisePropertyChanged(DisplayTextProperty, previousText, _displayText);
        }

        PlaceholderLabel.Text = _displayText;
        PlaceholderLabel.IsVisible = SelectedItem is null;
    }

    private void FlatSelectionChanged(object? sender, SelectionChangedEventArgs e)
    {
        if (_syncingSelection) return;

        if (FlatCombo.SelectedItem is SCComboBoxItem item)
        {
            SelectedItem = item;
        }
        else if (FlatCombo.SelectedItem is null)
        {
            SelectedItem = null;
        }

        SelectionChanged?.Invoke(this, e);
    }
}
