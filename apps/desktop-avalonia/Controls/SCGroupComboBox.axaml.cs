using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.Interactivity;
using Avalonia.Layout;
using System.Linq;

namespace SunCode.Desktop.Controls;

public sealed partial class SCGroupComboBox : UserControl
{
    public static readonly StyledProperty<IEnumerable<SCComboBoxGroup>?> GroupSourceProperty =
        AvaloniaProperty.Register<SCGroupComboBox, IEnumerable<SCComboBoxGroup>?>(nameof(GroupSource));

    public static readonly StyledProperty<SCComboBoxItem?> SelectedItemProperty =
        AvaloniaProperty.Register<SCGroupComboBox, SCComboBoxItem?>(
            nameof(SelectedItem),
            defaultBindingMode: Avalonia.Data.BindingMode.TwoWay);

    public static readonly StyledProperty<string?> PlaceholderTextProperty =
        AvaloniaProperty.Register<SCGroupComboBox, string?>(nameof(PlaceholderText));

    public static readonly StyledProperty<bool> UseMonospaceProperty =
        AvaloniaProperty.Register<SCGroupComboBox, bool>(nameof(UseMonospace));

    public static readonly DirectProperty<SCGroupComboBox, string> DisplayTextProperty =
        AvaloniaProperty.RegisterDirect<SCGroupComboBox, string>(nameof(DisplayText), control => control.DisplayText);

    private string _displayText = string.Empty;
    private Flyout? _groupedFlyout;
    private StackPanel? _groupedModelsPanel;
    private StackPanel? _groupedProvidersPanel;
    private SCComboBoxGroup? _activeGroup;

    public event EventHandler<SelectionChangedEventArgs>? SelectionChanged;

    public SCGroupComboBox()
    {
        InitializeComponent();
        Loaded += (_, _) => SyncView();
        SyncView();
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

    public bool UseMonospace
    {
        get => GetValue(UseMonospaceProperty);
        set => SetValue(UseMonospaceProperty, value);
    }

    public string DisplayText => _displayText;

    protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
    {
        base.OnPropertyChanged(change);
        if (change.Property == GroupSourceProperty
            || change.Property == SelectedItemProperty
            || change.Property == PlaceholderTextProperty
            || change.Property == UseMonospaceProperty)
        {
            SyncView();
        }
    }

    private void SyncView()
    {
        if (GroupedLabel is null) return;

        GroupedLabel.Classes.Set("mono", UseMonospace);
        UpdateGroupedLabel();
    }

    private void OpenGroupedMenu(object? sender, RoutedEventArgs e)
    {
        var groups = (GroupSource ?? []).Where(group => group.Items.Count > 0).ToArray();
        if (groups.Length == 0) return;

        _activeGroup = groups.FirstOrDefault(group => group.Items.Any(item => Equals(item, SelectedItem))) ?? groups[0];
        _groupedProvidersPanel = new StackPanel { Spacing = 2, HorizontalAlignment = HorizontalAlignment.Stretch };
        _groupedModelsPanel = new StackPanel { Spacing = 2, HorizontalAlignment = HorizontalAlignment.Stretch };

        foreach (var group in groups)
        {
            var providerLabel = new TextBlock { Text = group.Label, VerticalAlignment = VerticalAlignment.Center };
            var providerArrow = new TextBlock { Text = "›", VerticalAlignment = VerticalAlignment.Center, HorizontalAlignment = HorizontalAlignment.Right };
            providerLabel.Classes.Set("mono", UseMonospace);
            Grid.SetColumn(providerArrow, 1);
            var providerButton = new Button
            {
                Classes = { "sc-combo-menu-item" },
                HorizontalAlignment = HorizontalAlignment.Stretch,
                HorizontalContentAlignment = HorizontalAlignment.Stretch,
                Height = 28,
                MinHeight = 28,
                Padding = new Thickness(8, 0),
                Tag = group,
                Content = new Grid
                {
                    ColumnDefinitions = new ColumnDefinitions("*,12"),
                    Children = { providerLabel, providerArrow }
                }
            };
            providerButton.Classes.Set("active", ReferenceEquals(group, _activeGroup));
            providerButton.Click += (_, _) =>
            {
                _activeGroup = group;
                RefreshGroupedProviderButtons();
                RefreshGroupedModels();
            };
            _groupedProvidersPanel.Children.Add(providerButton);
        }

        RefreshGroupedModels();
        var columns = new Grid
        {
            ColumnDefinitions = new ColumnDefinitions("148,1,*"),
            Children =
            {
                _groupedProvidersPanel,
                new Border
                {
                    BorderBrush = this.FindResource("BorderBrush") as Avalonia.Media.IBrush,
                    BorderThickness = new Thickness(1, 0, 0, 0)
                },
                new Border { Padding = new Thickness(4, 0, 0, 0), Child = _groupedModelsPanel }
            }
        };
        Grid.SetColumn(columns.Children[1], 1);
        Grid.SetColumn(columns.Children[2], 2);

        _groupedFlyout = new Flyout
        {
            Placement = PlacementMode.TopEdgeAlignedRight,
            ShowMode = FlyoutShowMode.Standard,
            FlyoutPresenterClasses = { "sc-combo-flyout" },
            Content = new Border
            {
                Width = 340,
                Padding = new Thickness(4),
                Child = columns
            }
        };
        _groupedFlyout.ShowAt(GroupedButton);
    }

    private void RefreshGroupedProviderButtons()
    {
        if (_groupedProvidersPanel is null) return;
        foreach (var child in _groupedProvidersPanel.Children)
        {
            if (child is Button button && button.Tag is SCComboBoxGroup group)
                button.Classes.Set("active", ReferenceEquals(group, _activeGroup));
        }
    }

    private void RefreshGroupedModels()
    {
        if (_groupedModelsPanel is null || _activeGroup is null) return;
        _groupedModelsPanel.Children.Clear();
        foreach (var item in _activeGroup.Items)
        {
            var modelLabel = new TextBlock { Text = item.Label, VerticalAlignment = VerticalAlignment.Center };
            var modelCheck = new TextBlock { Text = Equals(item, SelectedItem) ? "✓" : string.Empty, VerticalAlignment = VerticalAlignment.Center, HorizontalAlignment = HorizontalAlignment.Right };
            modelLabel.Classes.Set("mono", UseMonospace);
            Grid.SetColumn(modelCheck, 1);
            var modelButton = new Button
            {
                Classes = { "sc-combo-menu-item" },
                HorizontalAlignment = HorizontalAlignment.Stretch,
                HorizontalContentAlignment = HorizontalAlignment.Stretch,
                Height = 28,
                MinHeight = 28,
                Padding = new Thickness(8, 0),
                IsEnabled = item.IsEnabled,
                Tag = item,
                Content = new Grid
                {
                    ColumnDefinitions = new ColumnDefinitions("*,16"),
                    Children = { modelLabel, modelCheck }
                }
            };
            modelButton.Classes.Set("active", Equals(item, SelectedItem));
            modelButton.Click += (_, args) => GroupedSelectionClicked(item, args);
            _groupedModelsPanel.Children.Add(modelButton);
        }
    }

    private void GroupedSelectionClicked(SCComboBoxItem item, RoutedEventArgs e)
    {
        var previous = SelectedItem;
        SelectedItem = item;
        _groupedFlyout?.Hide();
        var removedItems = previous is null
            ? new List<object?>()
            : new List<object?> { previous };
        SelectionChanged?.Invoke(
            this,
            new SelectionChangedEventArgs(
                SelectingItemsControl.SelectionChangedEvent,
                removedItems,
                new List<object?> { item }));
    }

    private void UpdateGroupedLabel()
    {
        if (GroupedLabel is null) return;

        var nextText = SelectedItem?.Label ?? PlaceholderText ?? string.Empty;
        var previousText = _displayText;
        _displayText = nextText;
        if (!string.Equals(previousText, nextText, StringComparison.Ordinal))
            RaisePropertyChanged(DisplayTextProperty, previousText, nextText);

        GroupedLabel.Text = nextText;
        GroupedLabel.Foreground = SelectedItem is null
            ? this.FindResource("TextMutedBrush") as Avalonia.Media.IBrush
            : this.FindResource("TextBrush") as Avalonia.Media.IBrush;
    }
}
