using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.Input.Platform;
using Avalonia.Interactivity;
using Avalonia.Markup.Xaml.Styling;
using Avalonia.Styling;
using Avalonia.Themes.Fluent;
using Avalonia.Threading;
using SunCode.Desktop.Controls;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Infrastructure.Git;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Views.ProjectWorkspace.Review;

namespace SunCode.Desktop.Tests;

[Collection("Avalonia UI")]
public sealed class DiffReviewerWindowTests(AvaloniaUiFixture ui)
{
    [Theory]
    [InlineData(1200, 680, false, "en-US")]
    [InlineData(820, 480, true, "zh-CN")]
    public Task WindowRendersAlignedFoldedDocumentsAndEndsWithItsOwner(int width, int height, bool dark, string locale) => ui.Run(async () =>
    {
        var app = Application.Current!;
        var theme = new FluentTheme();
        var editorStyles = new StyleInclude(new Uri("avares://SunCode/")) { Source = new("avares://AvaloniaEdit/Themes/Fluent/AvaloniaEdit.xaml") };
        var foundation = new StyleInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/FoundationStyles.axaml") };
        var controls = new StyleInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/ControlStyles.axaml") };
        var resources = new ResourceInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/ThemeResources.axaml") };
        var language = LocalizationService.CreateDictionary(locale);
        app.Styles.Add(theme); app.Styles.Add(editorStyles); app.Styles.Add(foundation); app.Styles.Add(controls);
        app.Resources.MergedDictionaries.Add(resources); app.Resources.MergedDictionaries.Add(language);
        var service = new DiffReviewerViewModelTests.ReviewerGitService();
        var lines = Enumerable.Range(1, 70).Select(line => new GitDiffLine("context", line, line, $"same {line}")).ToList();
        lines.AddRange([new("deletion", 71, null, "old"), new("addition", null, 71, "new"),
            new("context", 72, 72, "context"), new("addition", null, 73, "extra")]);
        lines.AddRange(Enumerable.Range(73, 70).Select(line => new GitDiffLine("context", line, line + 1, $"after second {line}")));
        lines.Add(new("addition", null, 144, "last"));
        service.Diff = DiffReviewDocumentTests.Patch(lines);
        using var model = new DiffReviewerViewModel(service, new("project", "Project", Path.GetTempPath()));
        var owner = new Window { Width = 1300, Height = 900 };
        var window = new DiffReviewerWindow(model) { RequestedThemeVariant = dark ? ThemeVariant.Dark : ThemeVariant.Light, Width = width, Height = height };
        try
        {
            owner.Show(); window.ShowFor(owner);
            await model.OpenAsync("all", null);
            Dispatcher.UIThread.RunJobs(); window.UpdateLayout();
            var before = window.FindControl<DiffCodePane>("BeforeEditor")!;
            var after = window.FindControl<DiffCodePane>("AfterEditor")!;
            var tree = window.FindControl<TreeView>("ChangedFilesTree")!;
            Assert.NotEqual(Avalonia.Media.Colors.Transparent, Assert.IsType<Avalonia.Media.SolidColorBrush>(before.Brush("TextBrush")).Color);
            Assert.NotEqual(Avalonia.Media.Colors.Transparent, Assert.IsType<Avalonia.Media.SolidColorBrush>(before.Brush("DiffReviewerAddedSurfaceBrush")).Color);
            Assert.True(tree.IsEffectivelyVisible);
            Assert.True(before.IsReadOnly && after.IsReadOnly);
            Assert.InRange(before.Bounds.Width, 279, width);
            Assert.InRange(after.Bounds.Width, 279, width);
            Assert.InRange(before.TextArea.TextView.DefaultLineHeight, 21.9, 22.1);
            Assert.False(window.FindControl<Button>("PreviousButton")!.IsEffectivelyEnabled);
            Assert.True(window.FindControl<Button>("NextButton")!.IsEffectivelyEnabled);
            before.ScrollToVerticalOffset(0);
            after.ScrollToVerticalOffset(0);
            before.TextArea.TextView.EnsureVisualLines();
            after.TextArea.TextView.EnsureVisualLines();
            var firstVisual = before.TextArea.TextView.VisualLines.First(line => line.FirstDocumentLine.LineNumber == 1);
            Assert.InRange(firstVisual.Height, 32, 34);
            Assert.Equal(32, Assert.IsType<Button>(Assert.IsType<AvaloniaEdit.Rendering.InlineObjectElement>(firstVisual.Elements[0]).Element).Height);
            model.ToggleContext(model.Document!.Blocks[0]);
            Dispatcher.UIThread.RunJobs(); window.UpdateLayout();
            Assert.True(((IScrollable)before.TextArea.TextView).Extent.Height > ((IScrollable)before.TextArea.TextView).Viewport.Height);
            model.MoveChange(1);
            Dispatcher.UIThread.RunJobs(); window.UpdateLayout();
            Assert.Equal("2 / 3", model.ChangePosition);
            model.ToggleContext(model.Document.Blocks[4]);
            Dispatcher.UIThread.RunJobs(); window.UpdateLayout();
            var middle = model.Document.DisplayRows.Select((row, index) => (row, index)).First(item => item.row.Line(true)?.Number == 100).index + 1;
            before.ScrollToVerticalOffset(before.TextArea.TextView.GetVisualTopByDocumentLine(middle));
            Dispatcher.UIThread.RunJobs(); window.UpdateLayout();
            Assert.Equal("2 / 3", model.ChangePosition);
            model.MoveChange(1);
            Dispatcher.UIThread.RunJobs(); window.UpdateLayout();
            Assert.Equal("3 / 3", model.ChangePosition);
            Assert.False(window.FindControl<Button>("NextButton")!.IsEffectivelyEnabled);
            Assert.InRange(Math.Abs(before.VerticalOffset - after.VerticalOffset), 0, 1);
            before.SelectAll();
            before.Copy();
            Dispatcher.UIThread.RunJobs();
            var copied = await window.Clipboard!.TryGetTextAsync();
            Assert.Equal(model.Document.Copy(true, 0, model.Document.Text(true).Length), copied);
            Assert.DoesNotContain('\uFFFC', copied!);
            before.TextArea.Selection = new AvaloniaEdit.Editing.RectangleSelection(before.TextArea,
                new AvaloniaEdit.TextViewPosition(2, 3), new AvaloniaEdit.TextViewPosition(4, 5));
            before.Copy();
            Dispatcher.UIThread.RunJobs();
            Assert.Equal("me\nme\nme", await window.Clipboard!.TryGetTextAsync());
            owner.Close();
            Assert.False(window.IsVisible);

            using var desktop = new DesktopViewModel(gitService: service);
            desktop.SetSelectedProjectForTests(new("source", "Source", Path.GetTempPath()));
            await desktop.Git.RefreshAsync();
            var source = new GitViewer { DataContext = desktop };
            var sourceOwner = new Window { Width = 620, Height = 500, Content = source };
            try
            {
                sourceOwner.Show(); Dispatcher.UIThread.RunJobs(); sourceOwner.UpdateLayout();
                var launch = source.FindControl<Button>("DiffReviewerButton")!;
                Assert.True(launch.IsEffectivelyEnabled);
                Assert.InRange(launch.TranslatePoint(new Point(launch.Bounds.Width, 0), source)!.Value.X, 0, source.Bounds.Width);
                launch.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
                var opened = Assert.Single(sourceOwner.OwnedWindows.OfType<DiffReviewerWindow>());
                launch.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
                Assert.Same(opened, Assert.Single(sourceOwner.OwnedWindows.OfType<DiffReviewerWindow>()));
                desktop.SetSelectedProjectForTests(new("next", "Next", Path.GetTempPath()));
                Assert.False(opened.IsVisible);
                Assert.Null(desktop.Git.Reviewer);
            }
            finally { sourceOwner.Close(); }
        }
        finally
        {
            window.Close(); owner.Close();
            app.Resources.MergedDictionaries.Remove(language); app.Resources.MergedDictionaries.Remove(resources);
            app.Styles.Remove(controls); app.Styles.Remove(foundation); app.Styles.Remove(editorStyles); app.Styles.Remove(theme);
        }
    });
}
