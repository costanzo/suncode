using Avalonia.Controls;
using SunCode.Desktop.Models;

namespace SunCode.Desktop.Views.About;

public sealed partial class AboutWindow : Window
{
    public AboutWindow()
    {
        InitializeComponent();
        WindowDecorations = Avalonia.Controls.WindowDecorations.Full;
        Icon = new WindowIcon(Avalonia.Platform.AssetLoader.Open(new Uri("avares://SunCode/Assets/logo/suncode-logo-128.png")));
        VersionText.Text = AppInfo.DisplayVersion;
    }
}
