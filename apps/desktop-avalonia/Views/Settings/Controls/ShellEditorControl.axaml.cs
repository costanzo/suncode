using Avalonia.Controls;
namespace SunCode.Desktop.Views.Settings.Controls;
public sealed partial class ShellEditorControl : UserControl
{
    public ShellEditorControl()
    {
        InitializeComponent();
        SizeChanged += (_, _) => ShellRow.ColumnDefinitions[1].Width = new GridLength(Bounds.Width <= 476 ? 180 : 220);
    }
}
