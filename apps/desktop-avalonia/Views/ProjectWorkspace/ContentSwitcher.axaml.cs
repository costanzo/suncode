using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Media;
using SunCode.Desktop.Models;

namespace SunCode.Desktop.Views.ProjectWorkspace;

public sealed partial class ContentSwitcher : UserControl
{
    public event Action<RecentContentItem>? ContentRequested;

    public ContentSwitcher()
    {
        InitializeComponent();
    }

    public bool CloseFlyout()
    {
        if (ContentSwitcherButton.Flyout is not { IsOpen: true } flyout) return false;
        flyout.Hide();
        return true;
    }

    private void ContentFlyoutOpened(object? sender, EventArgs e) => SetChevronAngle(-90);

    private void ContentFlyoutClosed(object? sender, EventArgs e) => SetChevronAngle(90);

    private void OpenRecentContent(object? sender, RoutedEventArgs e)
    {
        if (sender is not Button { CommandParameter: RecentContentItem item }) return;
        ContentSwitcherButton.Flyout?.Hide();
        ContentRequested?.Invoke(item);
    }

    private void SetChevronAngle(double angle)
    {
        if (ContentSwitcherChevron.RenderTransform is RotateTransform rotation)
            rotation.Angle = angle;
        else
            ContentSwitcherChevron.RenderTransform = new RotateTransform(angle);
    }
}
