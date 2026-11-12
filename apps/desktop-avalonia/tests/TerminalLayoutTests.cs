using System.Text;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Presenters;
using Avalonia.Controls.Templates;
using Avalonia.LogicalTree;
using Avalonia.Styling;
using Avalonia.Themes.Fluent;
using Avalonia.Threading;
using Avalonia.VisualTree;
using RoyalTerminal.Avalonia.Controls;
using RoyalTerminal.Terminal.Theming;
using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Views.ProjectWorkspace.Terminal;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Tests;

[Collection("Avalonia UI")]
public sealed class TerminalLayoutTests(AvaloniaUiFixture ui)
{
    [Fact]
    public Task AppearanceChangesPreserveShellForegroundAndBackgroundPairs() => ui.Run(() =>
    {
        using var session = new TerminalSessionViewModel("colors", Path.GetTempPath()) { State = "exited" };
        var view = new TerminalSessionView { DataContext = session };
        var window = new Window { Width = 620, Height = 180, Content = view, RequestedThemeVariant = ThemeVariant.Light };
        try
        {
            window.Show(); Dispatcher.UIThread.RunJobs();
            var terminal = view.FindControl<TerminalControl>("Terminal")!;
            terminal.WriteOutput(Encoding.UTF8.GetBytes(
                "\u001b[37;44mD\u001b[30;42mG\u001b[30;43mM\u001b[32;40mS\u001b[30;47mT" +
                "\u001b[38;5;254;48;5;25mI\u001b[38;2;1;2;3;48;2;4;5;6mR\u001b[0mX"));
            var palette = TerminalTheme.Dark.Palette;
            var expectedPairs = new (uint Foreground, uint Background)[]
            {
                (palette[7], palette[4]), (palette[0], palette[2]),
                (palette[0], palette[3]), (palette[2], palette[0]),
                (palette[0], palette[7]), (palette[254], palette[25]),
                (0xFF010203, 0xFF040506),
            };

            foreach (var variant in new[] { ThemeVariant.Light, ThemeVariant.Dark, ThemeVariant.Light })
            {
                window.RequestedThemeVariant = variant;
                Dispatcher.UIThread.RunJobs();
                var theme = terminal.Theme!;
                var light = variant == ThemeVariant.Light;
                Assert.Equal(light ? 0xFF26313Au : 0xFFD9E0E6u, theme.DefaultForeground);
                Assert.Equal(light ? 0xFFF2F5F7u : 0xFF0D1115u, theme.DefaultBackground);
                Assert.Equal(palette.ToArray(), theme.Palette.ToArray());
                var row = terminal.Screen!.GetRow(0);
                for (var index = 0; index < expectedPairs.Length; index++)
                {
                    Assert.Equal(expectedPairs[index].Foreground, row[index].Foreground);
                    Assert.Equal(expectedPairs[index].Background, row[index].Background);
                }
                Assert.Equal(theme.DefaultForeground, row[7].Foreground);
                Assert.Equal(theme.DefaultBackground, row[7].Background);
                if (light)
                {
                    Assert.Equal(0xFF26313Au, theme.CursorColor);
                    Assert.Equal(0xFFF2F5F7u, theme.CursorTextColor);
                    Assert.Equal(0xFF26313Au, theme.SelectionForeground);
                    Assert.Equal(0xFFBAC6D0u, theme.SelectionBackground);
                }
            }
        }
        finally { window.Close(); }
    });

    [Fact]
    public Task ShellStartupWaitsForVisibleMeasuredLayout() => ui.Run(() =>
    {
        var resolutions = 0;
        var pending = new TaskCompletionSource<ResolvedShell>();
        using var session = new TerminalSessionViewModel("deferred", Path.GetTempPath())
        {
            ResolveShellAsync = () => { resolutions++; return pending.Task; }
        };
        var view = new TerminalSessionView { DataContext = session };
        var container = new Border { IsVisible = false, Child = view };
        var window = new Window { Width = 620, Height = 180, Content = container };
        try
        {
            window.Show(); Dispatcher.UIThread.RunJobs();
            Assert.Equal(0, resolutions);
            container.IsVisible = true;
            window.UpdateLayout(); Dispatcher.UIThread.RunJobs();
            var terminal = view.FindControl<TerminalControl>("Terminal")!;
            Assert.True(terminal.Bounds.Width > 0);
            Assert.True(terminal.Bounds.Height > 0);
            Assert.Equal(1, resolutions);
            Assert.True(terminal.Renderer!.GlyphCache.RegularTypeface.IsFixedPitch);
        }
        finally { session.RequestStop(); window.Close(); pending.TrySetCanceled(); }
    });

    [Theory]
    [InlineData("exited")]
    [InlineData("error")]
    public Task LayoutChangesDoNotRestartEndedShells(string state) => ui.Run(() =>
    {
        var resolutions = 0;
        var pending = new TaskCompletionSource<ResolvedShell>();
        using var session = new TerminalSessionViewModel("ended", Path.GetTempPath())
        {
            State = state,
            ResolveShellAsync = () => { resolutions++; return pending.Task; }
        };
        var view = new TerminalSessionView { DataContext = session };
        var window = new Window { Width = 620, Height = 180, Content = view };
        try
        {
            window.Show(); Dispatcher.UIThread.RunJobs();
            window.Width = 1100;
            window.UpdateLayout(); Dispatcher.UIThread.RunJobs();
            Assert.Equal(0, resolutions);
            Assert.Equal(state, session.State);
            session.RequestRestart(); Dispatcher.UIThread.RunJobs();
            Assert.Equal(1, resolutions);
            Assert.Equal("starting", session.State);
        }
        finally { session.RequestStop(); window.Close(); pending.TrySetCanceled(); }
    });

    [Theory]
    [InlineData(620, 180, false)]
    [InlineData(1100, 280, true)]
    public Task SelectedTerminalFitsTheDrawerAndRetainsItsView(int width, int height, bool dark) => ui.Run(() =>
    {
        using var model = new DesktopViewModel();
        model.Layout.TerminalVisible = true;
        var pending = new TaskCompletionSource<ResolvedShell>();
        var first = new TerminalSessionViewModel("first", Path.GetTempPath()) { ResolveShellAsync = () => pending.Task };
        var second = new TerminalSessionViewModel("second", Path.GetTempPath()) { ResolveShellAsync = () => pending.Task };
        model.Terminal.Sessions.Add(first);
        model.Terminal.Sessions.Add(second);
        model.Terminal.SelectSession(first);
        var panel = new TerminalPanel { DataContext = model };
        var window = new Window
        {
            Width = width, Height = height, Content = panel,
            RequestedThemeVariant = dark ? ThemeVariant.Dark : ThemeVariant.Light
        };
        var theme = new FluentTheme();
        Application.Current!.Styles.Add(theme);
        // The shared headless fixture has no application control themes.
        var sessions = panel.GetLogicalDescendants().OfType<ItemsControl>().Single(item => item is not ListBox);
        sessions.Template = new FuncControlTemplate<ItemsControl>((control, _) => new ItemsPresenter
        {
            Name = "PART_ItemsPresenter", ItemsPanel = control.ItemsPanel
        });
        try
        {
            window.Show();
            Dispatcher.UIThread.RunJobs();
            window.UpdateLayout();
            var views = panel.GetVisualDescendants().OfType<TerminalSessionView>().ToArray();
            Assert.Equal(2, views.Length);
            var view = views.Single(item => ReferenceEquals(item.DataContext, first));
            var terminal = Assert.Single(view.GetVisualDescendants().OfType<TerminalControl>());
            var body = view.GetVisualAncestors().OfType<Grid>().First(grid => Grid.GetRow(grid) == 2);
            Assert.True(terminal.Bounds.Height > 0);
            Assert.Equal(body.Bounds.Height, terminal.Bounds.Height, precision: 1);
            Assert.Equal(body.Bounds.Width, terminal.Bounds.Width, precision: 1);

            model.Terminal.SelectSession(second);
            window.UpdateLayout();
            model.Terminal.SelectSession(first);
            panel.IsVisible = false;
            window.UpdateLayout();
            panel.IsVisible = true;
            window.UpdateLayout();
            Assert.Contains(view, panel.GetVisualDescendants().OfType<TerminalSessionView>());
            Assert.Equal(body.Bounds.Height, terminal.Bounds.Height, precision: 1);
        }
        finally
        {
            model.Terminal.CloseAll();
            pending.TrySetCanceled();
            window.Close();
            Application.Current!.Styles.Remove(theme);
        }
    });
}
