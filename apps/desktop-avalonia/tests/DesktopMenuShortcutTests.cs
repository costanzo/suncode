using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Media;
using SunCode.Desktop.Controls;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Views.ProjectWorkspace;

namespace SunCode.Desktop.Tests;

[Collection("Avalonia UI")]
public sealed class DesktopMenuShortcutTests(AvaloniaUiFixture ui)
{
    private static KeyEventArgs Press(Control target, Key key, KeyModifiers modifiers = KeyModifiers.Control)
    {
        var args = new KeyEventArgs { RoutedEvent = InputElement.KeyDownEvent, Key = key, KeyModifiers = modifiers };
        target.RaiseEvent(args);
        return args;
    }

    [Fact]
    public Task OpenProjectWorksWithoutMountingMenusAndRechecksConnectionState() => ui.Run(() =>
    {
        if (OperatingSystem.IsMacOS()) return; // AppKit owns native menu gestures on macOS.
        using var model = new DesktopViewModel();
        var input = new TextBox();
        var window = new Window { Content = input };
        var opened = 0;
        _ = new DesktopMenuController(() => window, () => model, () => null,
            () => { ++opened; return Task.CompletedTask; });
        try
        {
            window.Show();
            Assert.False(Press(input, Key.O).Handled);
            Assert.Equal(0, opened);
            ((IViewModelHost)model).ReportSuccess("Connected");
            // The menu has not opened or refreshed since the connection changed.
            input.KeyDown += (_, args) => args.Handled = true;
            Assert.True(Press(input, Key.O).Handled);
            Assert.Equal(1, opened);
        }
        finally { window.Close(); }
    });

    [Fact]
    public Task NewSessionRechecksSelectedProjectBusyStateAndModalState() => ui.Run(() =>
    {
        if (OperatingSystem.IsMacOS()) return;
        using var first = new DesktopViewModel();
        using var second = new DesktopViewModel();
        second.SetSelectedProjectForTests(new ProjectItem("second", "Second", Path.GetTempPath()));
        DesktopViewModel current = first;
        var resources = new ResourceDictionary { ["HitTestTransparentBrush"] = Brushes.Transparent };
        Application.Current!.Resources.MergedDictionaries.Add(resources);
        Window? window = null;
        try
        {
            var workspace = new ProjectWorkspace();
            var input = new TextBox();
            window = new Window { Content = input };
            _ = new DesktopMenuController(() => window, () => current, () => workspace, () => Task.CompletedTask);
            window.Show();
            Assert.False(Press(input, Key.N).Handled);
            current = second;
            Assert.True(Press(input, Key.N).Handled);
            ((IViewModelHost)second).SetBusy(true);
            Assert.False(Press(input, Key.N).Handled);
            ((IViewModelHost)second).SetBusy(false);
            workspace.FindControl<SCModal>("ExpandedComposerModal")!.IsOpen = true;
            Assert.False(Press(input, Key.N).Handled);
        }
        finally { window?.Close(); Application.Current!.Resources.MergedDictionaries.Remove(resources); }
    });

    [Fact]
    public Task WindowShortcutsExecuteOnceAndIgnoreExtraModifiers() => ui.Run(() =>
    {
        if (OperatingSystem.IsMacOS()) return;
        var input = new TextBox();
        var window = new Window { Content = input };
        var closes = 0;
        EventHandler<WindowClosingEventArgs> closing = (_, args) => { ++closes; args.Cancel = true; };
        window.Closing += closing;
        _ = new DesktopMenuController(() => window, () => null, () => null, () => Task.CompletedTask);
        window.Show();
        try
        {
            Assert.False(Press(input, Key.W, KeyModifiers.Control | KeyModifiers.Shift).Handled);
            Assert.False(Press(input, Key.W, KeyModifiers.Control | KeyModifiers.Alt).Handled);
            Assert.False(Press(input, Key.W, KeyModifiers.None).Handled);
            Assert.Equal(0, closes);
            Assert.True(Press(input, Key.W).Handled);
            Assert.Equal(1, closes);
        }
        finally { window.Closing -= closing; window.Close(); }
    });

    [Fact]
    public Task EditingKeysStayWithTheFocusedControl() => ui.Run(() =>
    {
        if (OperatingSystem.IsMacOS()) return;
        var input = new TextBox { Text = "draft" };
        var window = new Window { Content = input };
        var reachedInput = false;
        _ = new DesktopMenuController(() => window, () => null, () => null, () => Task.CompletedTask);
        input.KeyDown += (_, _) => reachedInput = true;
        try
        {
            window.Show();
            Press(input, Key.C);
            Assert.True(reachedInput);
        }
        finally { window.Close(); }
    });
}
