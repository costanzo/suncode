using System.Text.Json;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Data;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Markup.Xaml.Styling;
using Avalonia.Styling;
using Avalonia.Themes.Fluent;
using Avalonia.Threading;
using Avalonia.VisualTree;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Views.ProjectWorkspace.Chat;
using SunCode.Desktop.Views.ProjectWorkspace.Review;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Tests;

[Collection("Avalonia UI")]
public sealed class ToolActivityNavigationTests(AvaloniaUiFixture ui)
{
    [Theory]
    [InlineData(1000, false, "en-US", false)]
    [InlineData(1000, true, "zh-CN", true)]
    [InlineData(640, false, "zh-CN", true)]
    [InlineData(640, true, "en-US", false)]
    public Task ConversationClickSelectsRevealsAndFocusesTheExactCall(int width, bool dark, string locale, bool keyboard) => ui.Run(() => WithResources(locale, () =>
    {
        using var model = new DesktopViewModel();
        typeof(DesktopViewModel).GetProperty(nameof(DesktopViewModel.SelectedSession))!
            .SetValue(model, new SessionItem("session", "Session", "", false));
        var older = model.ToolActivity.EnsureTurn("older");
        for (var index = 0; index < 25; index++)
            model.ToolActivity.Upsert(older, $"old-{index}", "read", "succeeded", "{}", "{}", "", "");
        older.Update("completed");
        Apply(model, "turn.state", new(TurnId: "target", State: "resolving_calls"));
        var targetTurn = model.ToolActivity.Turns.Last();
        for (var index = 0; index < 25; index++)
            model.ToolActivity.Upsert(targetTurn, $"previous-{index}", "read", "succeeded", "{}", "{}", "", "");
        Apply(model, "tool.requested", new(TurnId: "target", ToolCallId: "exact-call", Name: "bash",
            Arguments: JsonSerializer.SerializeToElement(new { command = "build", files = Enumerable.Range(0, 80).Select(index => $"file-{index}") })));
        Apply(model, "tool.state", new(TurnId: "target", ToolCallId: "exact-call", Name: "bash", State: "executing"));
        var target = targetTurn.Tools.Last();
        model.ToolActivity.Select(older, older.Tools[0]);
        targetTurn.IsExpanded = false;
        model.Layout.OpenBottomDrawer(WorkspaceBottomDrawer.Terminal);
        var chat = new ChatArea { DataContext = model };
        chat.ToolDetailRequested += message => model.ShowToolActivity(message.TurnId, message.ToolCallId);
        var viewer = new ToolActivityViewer { DataContext = model };
        viewer.Bind(Visual.IsVisibleProperty, new Binding("Layout.ToolActivityVisible"));
        var content = new Grid { RowDefinitions = new("180,300") };
        content.Children.Add(chat);
        Grid.SetRow(viewer, 1);
        content.Children.Add(viewer);
        var window = new Window { Width = width, Height = 480, Content = content, RequestedThemeVariant = dark ? ThemeVariant.Dark : ThemeVariant.Light };
        try
        {
            window.Show();
            Flush(window);
            var jump = chat.GetVisualDescendants().OfType<Button>().Single(button => button.Classes.Contains("tool-card") && button.IsEffectivelyVisible);
            if (keyboard)
            {
                jump.Focus(NavigationMethod.Tab);
                window.KeyPress(Key.Enter, RawInputModifiers.None, PhysicalKey.Enter, null);
                window.KeyRelease(Key.Enter, RawInputModifiers.None, PhysicalKey.Enter, null);
            }
            else jump.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
            Flush(window);

            Assert.True(model.Layout.ToolActivityVisible);
            Assert.False(model.Layout.TerminalVisible);
            Assert.False(model.Layout.TurnDiffVisible);
            Assert.Same(target, model.ToolActivity.SelectedTool);
            Assert.True(targetTurn.IsExpanded);
            var tree = viewer.FindControl<TreeView>("ToolTree")!;
            Assert.Same(target, tree.SelectedItem);
            var row = tree.GetVisualDescendants().OfType<TreeViewItem>().Single(item => ReferenceEquals(item.DataContext, target));
            Assert.True(row.IsSelected);
            var position = row.TranslatePoint(default, tree)!.Value;
            Assert.InRange(position.Y, -1, tree.Bounds.Height - row.Bounds.Height + 1);
            var heading = viewer.FindControl<Grid>("ToolDetailHeading")!;
            var scroll = viewer.FindControl<ScrollViewer>("ToolDetailScroll")!;
            Assert.True(heading.IsFocused);
            Assert.Equal(0, scroll.Offset.Y);
            Assert.Contains(viewer.GetVisualDescendants().OfType<TextBlock>(), text => text.Text == target.ToolCallId && text.IsEffectivelyVisible);

            // Repeating a jump must reveal the same call even when selection did not change.
            targetTurn.IsExpanded = false;
            scroll.Offset = new Vector(0, 200);
            Flush(window);
            Assert.True(scroll.Offset.Y > 0);
            jump.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
            Flush(window);
            Assert.True(targetTurn.IsExpanded);
            Assert.Same(target, tree.SelectedItem);
            Assert.True(heading.IsFocused);
            Assert.Equal(0, scroll.Offset.Y);

            // Unknown ids leave the selected call and the open drawer intact.
            model.ShowToolActivity("target", "missing");
            Assert.Same(target, tree.SelectedItem);
            Assert.True(model.Layout.ToolActivityVisible);

            Apply(model, "tool.state", new(TurnId: "target", ToolCallId: "other-call", Name: "read", State: "executing"));
            Flush(window);
            Assert.Same(target, tree.SelectedItem);
            Assert.Same(target, model.ToolActivity.SelectedTool);

            // Completion between activation and layout still opens the clicked call.
            model.ShowToolActivity("target", "exact-call");
            Apply(model, "tool.state", new(TurnId: "target", ToolCallId: "exact-call", Name: "bash", State: "succeeded"));
            Flush(window);
            Assert.Same(target, model.ToolActivity.SelectedTool);
            Assert.True(heading.IsFocused);
        }
        finally { window.Close(); }
    }));

    [Theory]
    [InlineData(WorkspaceBottomDrawer.Git)]
    [InlineData(WorkspaceBottomDrawer.ProviderTrace)]
    [InlineData(WorkspaceBottomDrawer.Terminal)]
    [InlineData(WorkspaceBottomDrawer.TurnDiff)]
    public void OpeningDetailsUsesTheMutuallyExclusiveDrawer(WorkspaceBottomDrawer previous)
    {
        using var model = new DesktopViewModel();
        var turn = model.ToolActivity.EnsureTurn("turn");
        model.ToolActivity.Upsert(turn, "call", "read", "succeeded", "{}", "{}", "", "");
        model.Layout.OpenBottomDrawer(previous);
        model.ShowToolActivity("turn", "call");
        Assert.True(model.Layout.ToolActivityVisible);
        Assert.False(model.Layout.GitVisible);
        Assert.False(model.Layout.ProviderTraceVisible);
        Assert.False(model.Layout.TerminalVisible);
        Assert.False(model.Layout.TurnDiffVisible);
    }

    [Fact]
    public Task AQueuedJumpDoesNotFocusANewSessionOrAClosedDrawer() => ui.Run(() => WithResources("en-US", () =>
    {
        using var first = new DesktopViewModel();
        var turn = first.ToolActivity.EnsureTurn("turn");
        first.ToolActivity.Upsert(turn, "call", "read", "succeeded", "{}", "{}", "", "");
        using var second = new DesktopViewModel();
        var viewer = new ToolActivityViewer { DataContext = first };
        var focusTarget = new Button { Content = "Keep focus" };
        var root = new Grid { RowDefinitions = new("40,*") };
        root.Children.Add(focusTarget);
        Grid.SetRow(viewer, 1);
        root.Children.Add(viewer);
        var window = new Window { Width = 900, Height = 400, Content = root };
        try
        {
            window.Show(); Flush(window);
            focusTarget.Focus();
            first.ShowToolActivity("turn", "call");
            viewer.DataContext = second;
            Flush(window);
            Assert.True(focusTarget.IsFocused);
            Assert.Null(viewer.FindControl<TreeView>("ToolTree")!.SelectedItem);

            viewer.DataContext = first;
            first.ShowToolActivity("turn", "call");
            first.ClearSessionForTests();
            Flush(window);
            Assert.True(focusTarget.IsFocused);

            first.ToolActivity.Upsert(first.ToolActivity.EnsureTurn("turn"), "call", "read", "succeeded", "{}", "{}", "", "");
            first.ShowToolActivity("turn", "call");
            viewer.IsVisible = false;
            Flush(window);
            Assert.True(focusTarget.IsFocused);
        }
        finally { window.Close(); }
    }));

    private static void Apply(DesktopViewModel model, string type, AgentEventPayload payload) =>
        model.ApplyEvent(new AgentEvent("session", "2026-11-12T10:00:00Z", type, payload), live: true);

    private static void Flush(Window window)
    {
        Dispatcher.UIThread.RunJobs(); window.UpdateLayout(); Dispatcher.UIThread.RunJobs(); window.UpdateLayout();
    }

    private static void WithResources(string locale, Action action)
    {
        var app = Application.Current!;
        var theme = new FluentTheme();
        var foundation = new StyleInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/FoundationStyles.axaml") };
        var controls = new StyleInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/ControlStyles.axaml") };
        var resources = new ResourceInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/ThemeResources.axaml") };
        var language = LocalizationService.CreateDictionary(locale);
        app.Styles.Add(theme); app.Styles.Add(foundation); app.Styles.Add(controls);
        app.Resources.MergedDictionaries.Add(resources); app.Resources.MergedDictionaries.Add(language);
        try { action(); }
        finally
        {
            app.Resources.MergedDictionaries.Remove(language); app.Resources.MergedDictionaries.Remove(resources);
            app.Styles.Remove(controls); app.Styles.Remove(foundation); app.Styles.Remove(theme);
        }
    }
}
