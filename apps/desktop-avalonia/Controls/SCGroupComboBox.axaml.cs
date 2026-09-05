using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.Interactivity;

namespace SunCode.Desktop.Controls;

public sealed partial class SCGroupComboBox : UserControl
{
    public static readonly StyledProperty<IEnumerable<SCComboBoxGroup>?> GroupSourceProperty = AvaloniaProperty.Register<SCGroupComboBox, IEnumerable<SCComboBoxGroup>?>(nameof(GroupSource));
    public static readonly StyledProperty<SCComboBoxItem?> SelectedItemProperty = AvaloniaProperty.Register<SCGroupComboBox, SCComboBoxItem?>(nameof(SelectedItem), defaultBindingMode: Avalonia.Data.BindingMode.TwoWay);
    public static readonly StyledProperty<string?> PlaceholderTextProperty = AvaloniaProperty.Register<SCGroupComboBox, string?>(nameof(PlaceholderText));
    public static readonly StyledProperty<bool> UseMonospaceProperty = AvaloniaProperty.Register<SCGroupComboBox, bool>(nameof(UseMonospace));
    public static readonly DirectProperty<SCGroupComboBox, string> DisplayTextProperty = AvaloniaProperty.RegisterDirect<SCGroupComboBox, string>(nameof(DisplayText), control => control.DisplayText);

    private string _displayText = string.Empty;
    private SCComboBoxGroup? _activeGroup;
    public event EventHandler<SelectionChangedEventArgs>? SelectionChanged;

    public SCGroupComboBox() { InitializeComponent(); Loaded += (_, _) => SyncView(); SyncView(); }
    public IEnumerable<SCComboBoxGroup>? GroupSource { get => GetValue(GroupSourceProperty); set => SetValue(GroupSourceProperty, value); }
    public SCComboBoxItem? SelectedItem { get => GetValue(SelectedItemProperty); set => SetValue(SelectedItemProperty, value); }
    public string? PlaceholderText { get => GetValue(PlaceholderTextProperty); set => SetValue(PlaceholderTextProperty, value); }
    public bool UseMonospace { get => GetValue(UseMonospaceProperty); set => SetValue(UseMonospaceProperty, value); }
    public string DisplayText => _displayText;

    protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
    {
        base.OnPropertyChanged(change);
        if (change.Property == GroupSourceProperty || change.Property == SelectedItemProperty || change.Property == PlaceholderTextProperty || change.Property == UseMonospaceProperty) SyncView();
    }

    private void SyncView()
    {
        if (GroupedLabel is null) return;
        GroupedLabel.Classes.Set("mono", UseMonospace);
        var groups = (GroupSource ?? []).Where(group => group.Items.Count > 0).ToArray();
        GroupItemsControl.ItemsSource = groups.Select(group => new SCComboBoxGroupOption(group) { UseMonospace = UseMonospace }).ToArray();
        _activeGroup = groups.FirstOrDefault(group => group.Items.Any(item => Equals(item, SelectedItem))) ?? groups.FirstOrDefault();
        SyncMenuItems();
        var previousText = _displayText;
        _displayText = SelectedItem?.Label ?? PlaceholderText ?? string.Empty;
        if (!string.Equals(previousText, _displayText, StringComparison.Ordinal)) RaisePropertyChanged(DisplayTextProperty, previousText, _displayText);
        GroupedLabel.Text = _displayText;
    }

    private void OpenGroupedMenu(object? sender, RoutedEventArgs e)
    {
        if ((GroupSource ?? []).All(group => group.Items.Count == 0)) return;
        _activeGroup ??= GroupSource!.First(group => group.Items.Count > 0);
        SyncMenuItems();
    }

    private void SelectGroup(object? sender, RoutedEventArgs e)
    {
        if (sender is not Button { DataContext: SCComboBoxGroupOption option }) return;
        _activeGroup = option.Group;
        SyncMenuItems();
    }

    private void SelectItem(object? sender, RoutedEventArgs e)
    {
        if (sender is not Button { DataContext: SCComboBoxItemOption option } || !option.IsEnabled) return;
        var previous = SelectedItem;
        SelectedItem = option.Item;
        (GroupedButton.Flyout as Flyout)?.Hide();
        SelectionChanged?.Invoke(this, new SelectionChangedEventArgs(SelectingItemsControl.SelectionChangedEvent,
            previous is null ? new List<object?>() : new List<object?> { previous },
            new List<object?> { option.Item }));
    }

    private void SyncMenuItems()
    {
        if (GroupItemsControl is null || ItemItemsControl is null || _activeGroup is null) return;
        if (GroupItemsControl.ItemsSource is IEnumerable<SCComboBoxGroupOption> groups)
            foreach (var group in groups) group.IsActive = ReferenceEquals(group.Group, _activeGroup);
        ItemItemsControl.ItemsSource = _activeGroup.Items.Select(item => new SCComboBoxItemOption(item)
        {
            IsSelected = Equals(item, SelectedItem),
            UseMonospace = UseMonospace
        }).ToArray();
    }
}
