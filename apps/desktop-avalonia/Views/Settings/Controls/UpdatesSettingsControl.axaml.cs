using Avalonia.Controls;

namespace SunCode.Desktop.Views.Settings.Controls;

public sealed partial class UpdatesSettingsControl : UserControl
{
    private bool? _stacked;
    public UpdatesSettingsControl()
    {
        InitializeComponent();
        DesktopRow.SizeChanged += (_, _) => ResizeRows();
    }
    private void ResizeRows()
    {
        var stacked = DesktopRow.Bounds.Width <= 420;
        if (_stacked == stacked) return;
        _stacked = stacked;
        foreach (var (row, versions, status) in new[] { (DesktopRow, DesktopVersions, DesktopStatus), (SdkRow, SdkVersions, SdkStatus) })
        {
            row.ColumnDefinitions = ColumnDefinitions.Parse(stacked ? "*" : "*,Auto");
            Grid.SetColumn(versions, stacked ? 0 : 1); Grid.SetRow(versions, stacked ? 1 : 0);
            Grid.SetRow(status, stacked ? 2 : 1); Grid.SetColumnSpan(status, stacked ? 1 : 2);
        }
    }
}
