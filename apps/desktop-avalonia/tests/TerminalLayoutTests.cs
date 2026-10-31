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
using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Views.ProjectWorkspace.Terminal;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Tests;

[Collection("Avalonia UI")]
public sealed class TerminalLayoutTests(AvaloniaUiFixture ui)
{
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
