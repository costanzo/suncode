namespace SunCode.Desktop.Controls;

public sealed record SCComboBoxItem(string Label, object? Value = null, string? SecondaryText = null, bool IsEnabled = true)
{
    public bool HasSecondaryText => !string.IsNullOrWhiteSpace(SecondaryText);

    public override string ToString() => Label;
}

public sealed record SCComboBoxGroup(string Label, IReadOnlyList<SCComboBoxItem> Items);
