using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.Markup.Xaml.Styling;
using Avalonia.Styling;
using Avalonia.Themes.Fluent;
using Avalonia.Threading;
using Avalonia.VisualTree;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Views.ProjectWorkspace.Review;

namespace SunCode.Desktop.Tests;

[Collection("Avalonia UI")]
public sealed class QuestionWindowTests(AvaloniaUiFixture ui)
{
    [Theory]
    [InlineData(840, 640, false, "en-US")]
    [InlineData(840, 640, true, "zh-CN")]
    [InlineData(560, 400, true, "en-US")]
    [InlineData(390, 500, false, "zh-CN")]
    public Task EightQuestionsScrollWhileActionsStayVisible(int width, int height, bool dark, string locale) => ui.Run(() => WithResources(locale, async () =>
    {
        using var model = new DesktopViewModel();
        var request = QuestionInteractionTests.CreateRequest(count: 8);
        model.PendingQuestion = request;
        var owner = new Window { Width = 1000, Height = 800 };
        var dialog = new QuestionWindow(model, request) { RequestedThemeVariant = dark ? ThemeVariant.Dark : ThemeVariant.Light };
        try
        {
            owner.Show();
            var shown = dialog.ShowForAsync(owner);
            dialog.FitToScreen(new Size(width + 40, height + 40));
            dialog.Width = width;
            dialog.Height = height;
            Dispatcher.UIThread.RunJobs(); dialog.UpdateLayout();
            var scroll = dialog.FindControl<ScrollViewer>("QuestionScroll")!;
            var footer = dialog.FindControl<Border>("ActionsFooter")!;
            var submit = dialog.FindControl<Button>("SubmitButton")!;
            var skip = dialog.FindControl<Button>("SkipButton")!;
            var close = dialog.FindControl<Button>("CloseButton")!;
            Assert.True(scroll.Extent.Height > scroll.Viewport.Height);
            Assert.True(scroll.Extent.Width <= scroll.Viewport.Width + 1);
            Assert.InRange(footer.Bounds.Bottom, 1, dialog.Bounds.Height);
            Assert.True(skip.IsEffectivelyVisible);
            Assert.False(submit.IsEffectivelyEnabled);
            skip.Focus();
            dialog.KeyPress(Key.Tab, RawInputModifiers.None, PhysicalKey.Tab, null);
            dialog.KeyRelease(Key.Tab, RawInputModifiers.None, PhysicalKey.Tab, null);
            Assert.True(close.IsFocused);
            dialog.KeyPress(Key.Tab, RawInputModifiers.Shift, PhysicalKey.Tab, null);
            dialog.KeyRelease(Key.Tab, RawInputModifiers.Shift, PhysicalKey.Tab, null);
            Assert.True(skip.IsFocused);
            Assert.Contains(dialog.GetVisualDescendants().OfType<CheckBox>(), control => control.IsEffectivelyVisible);
            Assert.Contains(dialog.GetVisualDescendants().OfType<RadioButton>(), control => control.IsEffectivelyVisible);
            var checkboxes = dialog.GetVisualDescendants().OfType<CheckBox>().Where(control => control.IsEffectivelyVisible).Take(2).ToArray();
            checkboxes[0].IsChecked = true;
            checkboxes[1].IsChecked = true;
            Dispatcher.UIThread.RunJobs();
            Assert.Equal(["First", "Second"], request.Questions[1].Answers);
            Assert.All(dialog.GetVisualDescendants().OfType<TextBlock>().Where(text => text.Classes.Contains("question-text")), text => Assert.Equal(14, text.FontSize));
            Assert.All(dialog.GetVisualDescendants().OfType<TextBox>(), input => Assert.Equal(80, input.MinHeight));
            request.Questions[0].CustomAnswer = "An answer";
            Dispatcher.UIThread.RunJobs();
            Assert.True(submit.IsEffectivelyEnabled);
            scroll.Offset = new Vector(0, scroll.Extent.Height);
            dialog.UpdateLayout();
            Assert.InRange(footer.Bounds.Bottom, 1, dialog.Bounds.Height);
            Assert.True(request.TryBeginResolution(skip: false));
            Dispatcher.UIThread.RunJobs();
            Assert.False(submit.IsEffectivelyEnabled);
            Assert.False(skip.IsEffectivelyEnabled);
            Assert.All(dialog.GetVisualDescendants().OfType<TextBox>(), control => Assert.False(control.IsEffectivelyEnabled));
            request.FinishResolution(failed: true);
            Dispatcher.UIThread.RunJobs(); dialog.UpdateLayout();
            Assert.Contains(dialog.GetVisualDescendants().OfType<TextBlock>(), text => text.Text == request.ErrorText && text.IsEffectivelyVisible);
            Assert.True(submit.IsEffectivelyEnabled);
            dialog.Close();
            await shown;
        }
        finally { dialog.Close(); owner.Close(); }
    }));

    [Fact]
    public Task ExpandReusesTheWindowAndEscapeRetainsTheSharedDraft() => ui.Run(() => WithResources("en-US", async () =>
    {
        using var model = new DesktopViewModel();
        var request = QuestionInteractionTests.CreateRequest();
        model.PendingQuestion = request;
        var sidebar = new AgentSidebar { DataContext = model };
        var owner = new Window { Width = 312, Height = 700, Content = sidebar };
        try
        {
            owner.Show(); Dispatcher.UIThread.RunJobs();
            var shown = sidebar.ExpandQuestionAsync();
            Dispatcher.UIThread.RunJobs();
            var dialog = sidebar.ExpandedQuestionWindow!;
            Assert.True(sidebar.HasExpandedQuestion);
            await sidebar.ExpandQuestionAsync();
            Assert.Same(dialog, sidebar.ExpandedQuestionWindow);
            var radio = dialog.GetVisualDescendants().OfType<RadioButton>().First(control => control.IsEffectivelyVisible);
            radio.IsChecked = true;
            Dispatcher.UIThread.RunJobs();
            Assert.Equal(["First"], request.Questions[0].Answers);
            Assert.True(radio.GetVisualDescendants().OfType<Avalonia.Controls.Shapes.Ellipse>().Single().IsVisible);
            var input = dialog.GetVisualDescendants().OfType<TextBox>().Single();
            input.Text = "Draft from the window";
            Dispatcher.UIThread.RunJobs();
            Assert.Equal(["Draft from the window"], request.Questions[0].Answers);
            Assert.False(radio.GetVisualDescendants().OfType<Avalonia.Controls.Shapes.Ellipse>().Single().IsVisible);
            dialog.RaiseEvent(new KeyEventArgs { RoutedEvent = InputElement.KeyDownEvent, Key = Key.Escape });
            await shown;
            Dispatcher.UIThread.RunJobs();
            Assert.False(sidebar.HasExpandedQuestion);
            Assert.Same(request, model.PendingQuestion);
            Assert.Equal("Draft from the window", sidebar.GetVisualDescendants().OfType<TextBox>().Single().Text);
            Assert.True(sidebar.FindControl<Button>("ExpandQuestionButton")!.IsFocused);
            var reopened = sidebar.ExpandQuestionAsync();
            Dispatcher.UIThread.RunJobs();
            Assert.Equal("Draft from the window", sidebar.ExpandedQuestionWindow!.GetVisualDescendants().OfType<TextBox>().Single().Text);
            model.PendingQuestion = null;
            await reopened;
            Dispatcher.UIThread.RunJobs();
            Assert.False(sidebar.HasExpandedQuestion);
            Assert.True(sidebar.FindControl<TextBlock>("ReviewHeading")!.IsFocused);
        }
        finally { sidebar.ExpandedQuestionWindow?.Close(); owner.Close(); }
    }));

    [Fact]
    public Task OwnerCloseAndSessionClearEndTheExpandedPresentation() => ui.Run(() => WithResources("en-US", async () =>
    {
        using var model = new DesktopViewModel();
        model.PendingQuestion = QuestionInteractionTests.CreateRequest();
        var owner = new Window { Width = 1000, Height = 800 };
        owner.Show();
        var dialog = new QuestionWindow(model, model.PendingQuestion);
        var shown = dialog.ShowForAsync(owner);
        model.ClearSessionForTests();
        await shown;
        Assert.False(dialog.IsVisible);
        model.PendingQuestion = QuestionInteractionTests.CreateRequest("next");
        var next = new QuestionWindow(model, model.PendingQuestion);
        var nextShown = next.ShowForAsync(owner);
        owner.Close();
        await nextShown;
        Assert.False(next.IsVisible);
    }));

    private static async Task WithResources(string locale, Func<Task> test)
    {
        var application = Application.Current!;
        var theme = new FluentTheme();
        var foundation = new StyleInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/FoundationStyles.axaml") };
        var controls = new StyleInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/ControlStyles.axaml") };
        var resources = new ResourceInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/ThemeResources.axaml") };
        var language = LocalizationService.CreateDictionary(locale);
        application.Styles.Add(theme); application.Styles.Add(foundation); application.Styles.Add(controls);
        application.Resources.MergedDictionaries.Add(resources); application.Resources.MergedDictionaries.Add(language);
        try { await test(); }
        finally
        {
            application.Resources.MergedDictionaries.Remove(language); application.Resources.MergedDictionaries.Remove(resources);
            application.Styles.Remove(controls); application.Styles.Remove(foundation); application.Styles.Remove(theme);
        }
    }
}
