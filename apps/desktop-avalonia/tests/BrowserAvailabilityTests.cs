using Avalonia;
using Avalonia.Controls;
using Avalonia.Markup.Xaml.Styling;
using Avalonia.Styling;
using Avalonia.Themes.Fluent;
using Avalonia.Threading;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Views.ProjectWorkspace;

namespace SunCode.Desktop.Tests;

[Collection("Avalonia UI")]
public sealed class BrowserAvailabilityTests(AvaloniaUiFixture ui)
{
    [Fact]
    public Task BrowserEnablementIsSharedAcrossProjectWindows() => ui.Run(() =>
    {
        var settings = new AppSettingsViewModel();
        using var first = new DesktopViewModel(appSettings: settings);
        using var second = new DesktopViewModel(appSettings: settings);
        Assert.False(first.IsBrowserAvailable);
        Assert.False(second.IsBrowserAvailable);

        settings.SetBrowserUseEnabled(true);
        Assert.True(first.IsBrowserAvailable);
        Assert.True(second.IsBrowserAvailable);
        Assert.True(first.Browser.EnabledChecked);
        Assert.True(second.Browser.EnabledChecked);
        first.Layout.PreviewVisible = true;
        second.Layout.PreviewVisible = true;

        settings.SetBrowserUseEnabled(false);
        Assert.False(first.IsBrowserAvailable);
        Assert.False(second.IsBrowserAvailable);
        Assert.False(first.Browser.EnabledChecked);
        Assert.False(second.Browser.EnabledChecked);
        Assert.False(first.Layout.PreviewVisible);
        Assert.False(second.Layout.PreviewVisible);
        first.Browser.Runtime = new(true, "ready", "background", "", "", "CEF", 1, "unverified", null, null, null, 1, "full", null);
        Assert.False(first.IsBrowserAvailable);
        Assert.False(first.Browser.EnabledChecked);
    });

    [Theory]
    [InlineData(1440, 900, false, "en-US")]
    [InlineData(620, 620, true, "zh-CN")]
    public Task DisabledBrowserHidesGutterAndBothMenuPresentations(int width, int height, bool dark, string locale) => ui.Run(() =>
    {
        var application = Application.Current!;
        var theme = new FluentTheme();
        var foundation = new StyleInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/FoundationStyles.axaml") };
        var styles = new StyleInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/ControlStyles.axaml") };
        var resources = new ResourceInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/ThemeResources.axaml") };
        var language = LocalizationService.CreateDictionary(locale);
        application.Styles.Add(theme);
        application.Styles.Add(foundation);
        application.Styles.Add(styles);
        application.Resources.MergedDictionaries.Add(resources);
        application.Resources.MergedDictionaries.Add(language);
        var settings = new AppSettingsViewModel();
        settings.SetBrowserUseEnabled(true);
        using var model = new DesktopViewModel(appSettings: settings);
        model.SetSelectedProjectForTests(new ProjectItem("browser", "Browser", Path.GetTempPath()));
        model.Layout.UpdateLayoutSize(width, height);
        var workspace = new ProjectWorkspace { DataContext = model };
        var window = new Window { Width = width, Height = height, Content = workspace, RequestedThemeVariant = dark ? ThemeVariant.Dark : ThemeVariant.Light };
        try
        {
            var menus = new DesktopMenuController(() => window, () => model, () => workspace, () => Task.CompletedTask);
            window.Show();
            Dispatcher.UIThread.RunJobs();
            window.UpdateLayout();
            var gutter = workspace.FindControl<Button>("BrowserGutterButton")!;
            var viewLabel = LocalizationService.GetString("LocMenuView", "View");
            var browserLabel = LocalizationService.GetString("LocMenuBrowser", "Built-in Browser");
            var nativeBrowser = menus.Menu.Items.OfType<NativeMenuItem>().Single(item => Equals(item.Header, viewLabel))
                .Menu!.Items.OfType<NativeMenuItem>().Single(item => Equals(item.Header, browserLabel));
            var compactBrowser = menus.BarGroups.Single(item => Equals(item.Header, viewLabel))
                .Items.OfType<MenuItem>().Single(item => Equals(item.Header, browserLabel));
            Assert.True(gutter.IsVisible);
            Assert.True(nativeBrowser.IsVisible);
            Assert.True(compactBrowser.IsVisible);
            model.Layout.PreviewVisible = true;

            settings.SetBrowserUseEnabled(false);
            menus.Refresh();
            Dispatcher.UIThread.RunJobs();
            Assert.False(gutter.IsVisible);
            Assert.False(nativeBrowser.IsVisible);
            Assert.False(compactBrowser.IsVisible);
            Assert.False(model.Layout.PreviewVisible);
            workspace.ToggleBrowserPanel();
            Assert.False(model.Layout.PreviewVisible);

            settings.SetBrowserUseEnabled(true);
            menus.Refresh();
            Dispatcher.UIThread.RunJobs();
            Assert.True(gutter.IsVisible);
            Assert.True(nativeBrowser.IsVisible);
            Assert.True(compactBrowser.IsVisible);
            Assert.False(model.Layout.PreviewVisible);
        }
        finally
        {
            window.Close();
            application.Resources.MergedDictionaries.Remove(language);
            application.Resources.MergedDictionaries.Remove(resources);
            application.Styles.Remove(styles);
            application.Styles.Remove(foundation);
            application.Styles.Remove(theme);
        }
    });
}
