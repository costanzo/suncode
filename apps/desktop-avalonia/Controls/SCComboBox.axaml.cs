using System.Collections.Generic;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.Interactivity;
using System.Linq;

namespace SunCode.Desktop.Controls;

public sealed partial class SCComboBox : UserControl
{
    public static readonly StyledProperty<IEnumerable<SCComboBoxItem>?> ItemsSourceProperty =
        AvaloniaProperty.Register<SCComboBox, IEnumerable<SCComboBoxItem>?>(nameof(ItemsSource));

    public static readonly StyledProperty<IEnumerable<SCComboBoxGroup>?> GroupSourceProperty =
        AvaloniaProperty.Register<SCComboBox, IEnumerable<SCComboBoxGroup>?>(nameof(GroupSource));

    public static readonly StyledProperty<SCComboBoxItem?> SelectedItemProperty =
        AvaloniaProperty.Register<SCComboBox, SCComboBoxItem?>(nameof(SelectedItem), defaultBindingMode: Avalonia.Data.BindingMode.TwoWay);

    public static readonly StyledProperty<string?> PlaceholderTextProperty =
        AvaloniaProperty.Register<SCComboBox, string?>(nameof(PlaceholderText));

    public static readonly StyledProperty<bool> IsHierarchicalProperty =
        AvaloniaProperty.Register<SCComboBox, bool>(nameof(IsHierarchical));

    public static readonly StyledProperty<bool> UseMonospaceProperty =
        AvaloniaProperty.Register<SCComboBox, bool>(nameof(UseMonospace));

    public event EventHandler<SelectionChangedEventArgs>? SelectionChanged;

    public SCComboBox()
    {
        InitializeComponent();
        FlatCombo.SelectionChanged += FlatSelectionChanged;
        SyncView();
    }

    public IEnumerable<SCComboBoxItem>? ItemsSource
    {
        get => GetValue(ItemsSourceProperty);
        set => SetValue(ItemsSourceProperty, value);
    }

    public IEnumerable<SCComboBoxGroup>? GroupSource
    {
        get => GetValue(GroupSourceProperty);
        set => SetValue(GroupSourceProperty, value);
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

    public bool IsHierarchical
    {
        get => GetValue(IsHierarchicalProperty);
        set => SetValue(IsHierarchicalProperty, value);
    }

    public bool UseMonospace
    {
        get => GetValue(UseMonospaceProperty);
        set => SetValue(UseMonospaceProperty, value);
    }

    protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
    {
        base.OnPropertyChanged(change);
        if (change.Property == ItemsSourceProperty
            || change.Property == GroupSourceProperty
            || change.Property == SelectedItemProperty
            || change.Property == PlaceholderTextProperty
            || change.Property == IsHierarchicalProperty
            || change.Property == UseMonospaceProperty)
        {
            SyncView();
        }
    }

    private void SyncView()
    {
        FlatCombo.IsVisible = !IsHierarchical;
        GroupedButton.IsVisible = IsHierarchical;
        FlatCombo.ItemsSource = ItemsSource;
        FlatCombo.SelectedItem = SelectedItem;

        FlatCombo.Classes.Set("mono", UseMonospace);
        GroupedLabel.Classes.Set("mono", UseMonospace);
        GroupedLabel.Text = SelectedItem?.Label ?? PlaceholderText ?? string.Empty;
        GroupedLabel.Foreground = SelectedItem is null
            ? this.FindResource("TextMutedBrush") as Avalonia.Media.IBrush
            : this.FindResource("TextBrush") as Avalonia.Media.IBrush;
    }

    private void FlatSelectionChanged(object? sender, SelectionChangedEventArgs e)
    {
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

    private void OpenGroupedMenu(object? sender, RoutedEventArgs e)
    {
        var menu = new MenuFlyout
        {
            Placement = PlacementMode.TopEdgeAlignedRight
        };

        foreach (var group in GroupSource ?? [])
        {
            var groupItem = new MenuItem
            {
                Header = group.Label
            };

            foreach (var item in group.Items)
            {
                var itemButton = new MenuItem
                {
                    Header = item.Label,
                    CommandParameter = item,
                    IsEnabled = item.IsEnabled,
                    ToggleType = MenuItemToggleType.Radio,
                    GroupName = "sc-combo-groups",
                    IsChecked = Equals(item, SelectedItem)
                };
                itemButton.Click += GroupedSelectionClicked;
                groupItem.Items.Add(itemButton);
            }

            if (groupItem.Items.Count > 0)
            {
                menu.Items.Add(groupItem);
            }
        }

        menu.ShowAt(GroupedButton);
    }

    private void GroupedSelectionClicked(object? sender, RoutedEventArgs e)
    {
        if (sender is not MenuItem { CommandParameter: SCComboBoxItem item }) return;
        var previous = SelectedItem;
        SelectedItem = item;
        GroupedLabel.Text = item.Label;
        var removedItems = previous is null
            ? new List<object?>()
            : new List<object?> { previous };
        var addedItems = new List<object?> { item };
        SelectionChanged?.Invoke(
            this,
            new SelectionChangedEventArgs(
                SelectingItemsControl.SelectionChangedEvent,
                removedItems,
                addedItems));
    }
}
