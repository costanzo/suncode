namespace SunCode.Desktop.Controls;

public sealed record SCComboBoxItem(string Label, object? Value = null, string? SecondaryText = null, bool IsEnabled = true)
{
    public bool HasSecondaryText => !string.IsNullOrWhiteSpace(SecondaryText);

    public override string ToString() => Label;
}

public sealed record SCComboBoxGroup(string Label, IReadOnlyList<SCComboBoxItem> Items);

public sealed class SCComboBoxGroupOption : System.ComponentModel.INotifyPropertyChanged
{
    private bool _isActive;
    public bool UseMonospace { get; set; }
    public SCComboBoxGroupOption(SCComboBoxGroup group) => Group = group;
    public SCComboBoxGroup Group { get; }
    public string Label => Group.Label;
    public bool IsActive { get => _isActive; set { if (_isActive == value) return; _isActive = value; PropertyChanged?.Invoke(this, new(nameof(IsActive))); } }
    public event System.ComponentModel.PropertyChangedEventHandler? PropertyChanged;
}

public sealed class SCComboBoxItemOption : System.ComponentModel.INotifyPropertyChanged
{
    private bool _isSelected;
    public bool UseMonospace { get; set; }
    public SCComboBoxItemOption(SCComboBoxItem item) => Item = item;
    public SCComboBoxItem Item { get; }
    public string Label => Item.Label;
    public bool IsEnabled => Item.IsEnabled;
    public bool IsSelected { get => _isSelected; set { if (_isSelected == value) return; _isSelected = value; PropertyChanged?.Invoke(this, new(nameof(IsSelected))); } }
    public event System.ComponentModel.PropertyChangedEventHandler? PropertyChanged;
}
