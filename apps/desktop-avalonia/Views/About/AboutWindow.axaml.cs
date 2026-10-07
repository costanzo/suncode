using Avalonia.Controls;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Sdk;

namespace SunCode.Desktop.Views.About;

public sealed partial class AboutWindow : Window
{
    public AboutWindow()
    {
        InitializeComponent();
        WindowDecorations = Avalonia.Controls.WindowDecorations.Full;
        Icon = AppIcon.Window;
        DesktopVersionText.Text = AppInfo.DisplayVersion;
        AgentSdkVersionText.Text = "Loading...";
        Opened += OnOpened;
    }

    private async void OnOpened(object? sender, EventArgs e)
    {
        Opened -= OnOpened;
        try
        {
            var result = await AgentSdk.GetVersionAsync();
            AgentSdkVersionText.Text = AppInfo.FormatVersion(result.Version);
        }
        catch (Exception exception)
        {
            AgentSdkVersionText.Text = "Unavailable";
            DiagnosticLog.Error("about.version", exception, "component=agent-sdk");
        }
    }
}
