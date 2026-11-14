using Avalonia;
using Avalonia.Controls;
using Avalonia.Markup.Xaml.Styling;
using Avalonia.Styling;
using Avalonia.Themes.Fluent;
using Avalonia.Threading;
using Avalonia.VisualTree;
using SunCode.Desktop.Controls;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Views.ProjectWorkspace;

namespace SunCode.Desktop.Tests;

[Collection("Avalonia UI")]
public sealed class MergedWorkspaceWindowTests(AvaloniaUiFixture ui)
{
    [Theory]
    [InlineData(0, 0, false)]
    [InlineData(20, 10, false)]
    [InlineData(28, 0, true)]
    [InlineData(0, -28, true)]
    [InlineData(21, 21, true)]
    public void TearOffRequiresThresholdDistance(double deltaX, double deltaY, bool expected)
    {
        Assert.Equal(expected, MergedWorkspaceWindow.ShouldTearOff(deltaX, deltaY));
    }

    [Fact]
    public Task InactiveProjectsRetainStylesThroughSwitchingAndTearOff() => ui.Run(() =>
    {
        foreach (var (dark, width, locale, tearOffActive) in new[]
        {
            (false, 620, "en-US", false),
            (false, 1440, "zh-CN", false),
            (true, 620, "zh-CN", false),
            (true, 1440, "en-US", false),
            (false, 620, "zh-CN", true),
            (false, 1440, "en-US", true),
            (true, 620, "en-US", true),
            (true, 1440, "zh-CN", true)
        })
            VerifyInactiveProjectStyles(dark, width, locale, tearOffActive);
    });

    private static void VerifyInactiveProjectStyles(bool dark, int width, string locale, bool tearOffActive)
    {
        var app = Application.Current!;
        var theme = new FluentTheme();
        var foundation = new StyleInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/FoundationStyles.axaml") };
        var controls = new StyleInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/ControlStyles.axaml") };
        var resources = new ResourceInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/ThemeResources.axaml") };
        var language = LocalizationService.CreateDictionary(locale);
        app.Styles.Add(theme);
        app.Styles.Add(foundation);
        app.Styles.Add(controls);
        app.Resources.MergedDictionaries.Add(resources);
        app.Resources.MergedDictionaries.Add(language);
        using var firstModel = new DesktopViewModel();
        using var secondModel = new DesktopViewModel();
        firstModel.BeginProjectOpening(new ProjectItem("first", "First", ""));
        secondModel.BeginProjectOpening(new ProjectItem("second", "Second", ""));
        firstModel.ComposerText = "First draft";
        secondModel.ComposerText = "Second draft";
        var variant = dark ? ThemeVariant.Dark : ThemeVariant.Light;
        var first = new WorkspaceWindow { DataContext = firstModel, RequestedThemeVariant = variant, Width = width };
        var second = new WorkspaceWindow { DataContext = secondModel, RequestedThemeVariant = variant, Width = width };
        var merged = new MergedWorkspaceWindow(firstModel) { RequestedThemeVariant = variant, Width = width };
        WorkspaceWindow? detached = null;
        try
        {
            first.Show();
            second.Show();
            Dispatcher.UIThread.RunJobs();
            first.UpdateLayout();
            second.UpdateLayout();
            var switcher = second.Workspace.FindControl<ProjectSwitcher>("ProjectSwitcherControl")!.FindControl<Button>("ProjectSwitcherButton")!;
            var background = switcher.Background;
            var padding = switcher.Padding;
            var border = switcher.BorderThickness;
            var light = second.Workspace.FindControl<TrafficLightButton>("ProjectCloseLight")!;
            var lightTemplate = light.Template;
            Assert.NotNull(lightTemplate);

            merged.AddProject("first", first);
            first.ReleaseForMerge();
            merged.AddProject("second", second);
            second.ReleaseForMerge();
            merged.Show();
            Dispatcher.UIThread.RunJobs();
            merged.UpdateLayout();
            for (var iteration = 0; iteration < 3; iteration++)
            {
                // Pending layout/template work may apply styles while a tab is inactive.
                foreach (var element in second.Workspace.GetVisualDescendants().OfType<StyledElement>().Prepend(second.Workspace).ToArray())
                    element.ApplyStyling();
                merged.SelectProject("second");
                Dispatcher.UIThread.RunJobs();
                merged.UpdateLayout();
                Assert.Equal(background, switcher.Background);
                Assert.Equal(padding, switcher.Padding);
                Assert.Equal(border, switcher.BorderThickness);
                Assert.Same(lightTemplate, light.Template);
                Assert.Same(secondModel, merged.DataContext);
                Assert.Same(secondModel, second.Workspace.DataContext);
                Assert.True(second.Workspace.IsEffectivelyVisible);
                Assert.False(first.Workspace.IsEffectivelyVisible);
                merged.SelectProject("first");
                Assert.Same(merged, TopLevel.GetTopLevel(second.Workspace));
            }

            if (tearOffActive) merged.SelectProject("second");
            var released = merged.RemoveProject("second")!;
            Assert.Null(TopLevel.GetTopLevel(released.Workspace));
            detached = new WorkspaceWindow(released.Workspace, adoptedWorkspace: true)
            {
                DataContext = secondModel,
                RequestedThemeVariant = variant,
                Width = width
            };
            detached.Show();
            Dispatcher.UIThread.RunJobs();
            detached.UpdateLayout();
            Assert.Same(detached, TopLevel.GetTopLevel(second.Workspace));
            Assert.True(second.Workspace.IsEffectivelyVisible);
            Assert.Equal(background, switcher.Background);
            Assert.Equal(padding, switcher.Padding);
            Assert.Same(lightTemplate, light.Template);
            Assert.Equal("First draft", firstModel.ComposerText);
            Assert.Equal("Second draft", secondModel.ComposerText);
        }
        finally
        {
            merged.RemoveProject("first");
            merged.RemoveProject("second");
            detached?.Close();
            merged.Close();
            first.Close();
            second.Close();
            app.Styles.Remove(controls);
            app.Styles.Remove(foundation);
            app.Styles.Remove(theme);
            app.Resources.MergedDictionaries.Remove(language);
            app.Resources.MergedDictionaries.Remove(resources);
        }
    }
}
