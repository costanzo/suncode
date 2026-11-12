using System.Text.Json;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Markup.Xaml.Styling;
using Avalonia.Media;
using Avalonia.Styling;
using Avalonia.Themes.Fluent;
using Avalonia.Threading;
using Avalonia.VisualTree;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Views.ProjectWorkspace.Review;
using SunCode.Sdk.Models;
using SvgControl = Avalonia.Svg.Skia.Svg;

namespace SunCode.Desktop.Tests;

public sealed class TodoPresentationTests
{
    private static readonly JsonSerializerOptions JsonOptions = new() { PropertyNameCaseInsensitive = true };

    [Theory]
    [InlineData("completed")]
    [InlineData("failed")]
    [InlineData("cancelled")]
    [InlineData("interrupted")]
    [InlineData("awaiting_approval")]
    [InlineData("awaiting_question")]
    [InlineData("compacting")]
    public void TodoCardStaysVisibleUntilTheNextTurnIsAdmitted(string state)
    {
        using var viewModel = new DesktopViewModel();
        viewModel.ApplyEvent(TurnState("turn-1", "admitted"), live: true);
        Assert.False(viewModel.IsReviewTodosVisible);
        viewModel.ApplyEvent(Event("""
        {"event_type":"todo.updated","payload":{"turn_id":"turn-1","todos":[
          {"content":"Inspect project","status":"completed","priority":"high"}
        ]}}
        """), live: true);
        Assert.True(viewModel.IsReviewTodosVisible);

        viewModel.ApplyEvent(TurnState("turn-1", state), live: true);

        Assert.True(viewModel.IsReviewTodosVisible);
        Assert.Equal("completed", Assert.Single(viewModel.CurrentTodos).Status);

        viewModel.ApplyEvent(TurnState("turn-2", "admitted"), live: true);

        Assert.False(viewModel.IsReviewTodosVisible);
        Assert.Empty(viewModel.CurrentTodos);
    }

    [Theory]
    [InlineData("completed")]
    [InlineData("failed")]
    [InlineData("cancelled")]
    [InlineData("interrupted")]
    public void SnapshotRestoresTerminalTodosAndClearsThemForANewerEmptyTurn(string state)
    {
        using var viewModel = new DesktopViewModel();
        var snapshot = Snapshot($$"""
        {"conversationTurns":[{
          "turnId":"turn-1","state":"{{state}}",
          "todos":[{"content":"Inspect project","status":"completed","priority":"high","ordinal":0}]
        }]}
        """);

        viewModel.ApplySnapshot(DesktopViewModel.ProjectSnapshot(snapshot));

        Assert.True(viewModel.IsReviewTodosVisible);
        Assert.Equal("Inspect project", Assert.Single(viewModel.CurrentTodos).Content);

        var nextTurn = Snapshot("""
        {"conversationTurns":[{"turnId":"turn-2","state":"admitted","todos":[]}]}
        """).ConversationTurns[0];
        viewModel.ApplySnapshot(DesktopViewModel.ProjectSnapshot(snapshot with
        {
            ConversationTurns = [.. snapshot.ConversationTurns, nextTurn]
        }));

        Assert.False(viewModel.IsReviewTodosVisible);
        Assert.Empty(viewModel.CurrentTodos);
    }

    private static SessionSnapshot Snapshot(string json)
    {
        var snapshot = JsonSerializer.Deserialize<SessionSnapshot>(json, JsonOptions)!;
        return snapshot with
        {
            Messages = [],
            Images = [],
            ConversationTurns = snapshot.ConversationTurns.Select(turn => turn with
            {
                Messages = [],
                ToolUses = [],
                Todos = turn.Todos ?? []
            }).ToArray()
        };
    }

    private static AgentEvent Event(string json)
    {
        var value = JsonSerializer.Deserialize<AgentEvent>(json, JsonOptions)!;
        return value with { SessionId = string.Empty, OccurredAt = "2026-11-12T10:00:00Z" };
    }

    private static AgentEvent TurnState(string turnId, string state) => Event($$$"""
        {"event_type":"turn.state","payload":{"turn_id":"{{{turnId}}}","state":"{{{state}}}"}}
        """);
}

[Collection("Avalonia UI")]
public sealed class TodoSidebarTests(AvaloniaUiFixture ui)
{
    [Theory]
    [InlineData(312, false, "en-US")]
    [InlineData(312, true, "zh-CN")]
    [InlineData(240, false, "zh-CN")]
    [InlineData(240, true, "en-US")]
    public Task SidebarRetainsTheTodoCardAfterCompletion(int width, bool dark, string locale) => ui.Run(() =>
    {
        var app = Application.Current!;
        var theme = new FluentTheme();
        var foundation = new StyleInclude(new Uri("avares://SunCode/"))
        {
            Source = new("avares://SunCode/Styles/Application/FoundationStyles.axaml")
        };
        var resources = new ResourceInclude(new Uri("avares://SunCode/"))
        {
            Source = new("avares://SunCode/Styles/Application/ThemeResources.axaml")
        };
        var language = LocalizationService.CreateDictionary(locale);
        app.Styles.Add(theme);
        app.Styles.Add(foundation);
        app.Resources.MergedDictionaries.Add(resources);
        app.Resources.MergedDictionaries.Add(language);
        using var model = new DesktopViewModel();
        var sidebar = new AgentSidebar { DataContext = model };
        var window = new Window
        {
            Width = width,
            Height = 800,
            Content = sidebar,
            RequestedThemeVariant = dark ? ThemeVariant.Dark : ThemeVariant.Light
        };
        try
        {
            model.ApplyEvent(TurnState("turn-1", "admitted"), live: true);
            model.ApplyEvent(new AgentEvent("session", "2026-11-12T10:00:00Z", "todo.updated",
                new AgentEventPayload(TurnId: "turn-1", Todos:
                [
                    new("Inspect Avalonia workspace", "completed", "high"),
                    new("Build focused modules", "in_progress", "high"),
                    new("Verify responsive routes and 中文内容 wrapping inside a constrained sidebar", "pending", "medium"),
                    new("Remove stale checkpoint", "cancelled", "low")
                ])), live: true);
            window.Show();
            Dispatcher.UIThread.RunJobs();
            window.UpdateLayout();
            var items = sidebar.GetVisualDescendants().OfType<ItemsControl>()
                .Single(control => ReferenceEquals(control.ItemsSource, model.CurrentTodos));
            Assert.True(items.IsEffectivelyVisible);
            var card = sidebar.FindControl<Border>("TodoCard")!;
            var heading = sidebar.FindControl<TextBlock>("TodoHeading")!;
            Assert.Equal(new Thickness(11), card.Padding);
            Assert.Equal(new CornerRadius(6), card.CornerRadius);
            Assert.Equal(9, heading.FontSize);
            Assert.Equal(FontWeight.Normal, heading.FontWeight);
            Assert.Contains("JetBrains Mono", heading.FontFamily.Name);
            Assert.Equal(0.54, heading.LetterSpacing);
            var rows = items.GetVisualDescendants().OfType<Grid>()
                .Where(grid => grid.Classes.Contains("todo-row")).ToArray();
            Assert.Equal(4, rows.Length);
            var cssKeys = new[] { "SuccessSvgCss", "AccentSvgCss", "MutedIconSvgCss", "DangerSvgCss" };
            for (var index = 0; index < rows.Length; index++)
            {
                var row = rows[index];
                var todo = model.CurrentTodos[index];
                var icon = Assert.Single(row.GetVisualDescendants().OfType<SvgControl>());
                var content = Assert.Single(row.GetVisualDescendants().OfType<TextBlock>());
                Assert.Equal(todo.StatusIconPath, icon.Path);
                Assert.Equal(1, Assert.IsType<TranslateTransform>(icon.RenderTransform).Y);
                Assert.True(resources.TryGetResource(cssKeys[index], window.ActualThemeVariant, out var expectedCss));
                Assert.Equal(expectedCss, SvgControl.GetCss(icon));
                Assert.Equal(todo.Opacity, row.Opacity);
                Assert.Equal(2, row.ColumnDefinitions.Count);
                Assert.Equal(6, row.ColumnSpacing);
                Assert.Equal(16, row.MinHeight);
                Assert.Equal(todo.Content, content.Text);
                Assert.Equal(9, content.FontSize);
                Assert.Contains("Noto Sans", content.FontFamily.Name);
                Assert.Equal(TextWrapping.Wrap, content.TextWrapping);
            }

            foreach (var state in new[] { "completed", "failed", "cancelled", "interrupted" })
            {
                model.ApplyEvent(TurnState("turn-1", state), live: true);
                Dispatcher.UIThread.RunJobs();
                window.UpdateLayout();
                Assert.True(items.IsEffectivelyVisible);
                Assert.InRange(items.Bounds.Width, 1, width);
            }

            model.ApplyEvent(TurnState("turn-2", "admitted"), live: true);
            Dispatcher.UIThread.RunJobs();
            Assert.False(items.IsEffectivelyVisible);
        }
        finally
        {
            window.Close();
            app.Resources.MergedDictionaries.Remove(language);
            app.Resources.MergedDictionaries.Remove(resources);
            app.Styles.Remove(theme);
            app.Styles.Remove(foundation);
        }
    });

    private static AgentEvent TurnState(string turnId, string state) =>
        new("session", "2026-11-12T10:00:00Z", "turn.state", new AgentEventPayload(TurnId: turnId, State: state));
}
