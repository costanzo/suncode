using Avalonia;
using Avalonia.Controls;
using Avalonia.Automation.Peers;
using Avalonia.Markup.Xaml.Styling;
using Avalonia.Styling;
using Avalonia.Themes.Fluent;
using Avalonia.Threading;
using Avalonia.VisualTree;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Views.Settings.Controls;

namespace SunCode.Desktop.Tests;

[Collection("Avalonia UI")]
public sealed class BrowserComponentLayoutTests(AvaloniaUiFixture ui)
{
    [Theory]
    [InlineData(662, 586, false, "en-US", false)]
    [InlineData(482, 466, true, "zh-CN", false)]
    [InlineData(482, 466, false, "en-US", true)]
    [InlineData(662, 586, true, "zh-CN", true)]
    public Task ComponentSettingsFitBothThemesAndLanguages(int width, int height, bool dark, string locale, bool failed) => ui.Run(async () =>
    {
        using var fixture = new BrowserComponentTests.Fixture { BadDigest = failed };
        var service = fixture.Service();
        if (failed) await service.InstallAsync(BrowserComponentTests.Network);
        var application = Application.Current!;
        var theme = new FluentTheme();
        var styles = new StyleInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/ControlStyles.axaml") };
        var resources = new ResourceInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/ThemeResources.axaml") };
        var language = LocalizationService.CreateDictionary(locale);
        application.Styles.Add(theme); application.Styles.Add(styles);
        application.Resources.MergedDictionaries.Add(resources); application.Resources.MergedDictionaries.Add(language);
        var host = new FakeViewModelHost { SelectedProject = new("project", "Project", fixture.Root) };
        var model = new BrowserRuntimeViewModel(host, service)
        {
            Runtime = new(false, "missing", "not_started", BrowserComponentCatalog.Target, "", "CEF", 1, "unverified", null, fixture.Root, null, 0, "full", null),
        };
        model.SelectedDownloadSource = model.DownloadSources[1];
        var panel = new BrowserUseSettingsControl { DataContext = model };
        var window = new Window { Width = width, Height = height, Content = panel, RequestedThemeVariant = dark ? ThemeVariant.Dark : ThemeVariant.Light };
        try
        {
            window.Show(); Dispatcher.UIThread.RunJobs(); window.UpdateLayout();
            var download = Assert.Single(panel.GetVisualDescendants().OfType<Button>(), button => ReferenceEquals(button.Command, model.InstallComponentCommand));
            Assert.True(download.IsEffectivelyVisible);
            Assert.True(download.IsEffectivelyEnabled);
            var source = Assert.Single(panel.GetVisualDescendants().OfType<ComboBox>());
            Assert.Equal(LocalizationService.GetString("LocBrowserDownloadSource", "Download source"), ControlAutomationPeer.CreatePeerForElement(source)!.GetName());
            Assert.Equal(model.DownloadSources[1], source.SelectedItem);
            var scroll = panel.GetVisualDescendants().OfType<ScrollViewer>().First();
            Assert.True(scroll.Extent.Width <= scroll.Viewport.Width + 1);
            Assert.DoesNotContain(panel.GetVisualDescendants().OfType<TextBlock>(), text => text.IsEffectivelyVisible && text.Text?.StartsWith("LocBrowser", StringComparison.Ordinal) == true);
            var expander = panel.GetVisualDescendants().OfType<Expander>().Single();
            expander.IsExpanded = true;
            Dispatcher.UIThread.RunJobs(); window.UpdateLayout();
            Assert.True(scroll.Extent.Width <= scroll.Viewport.Width + 1);
            Assert.Equal(8, model.ComponentDiagnostics.Count);
            Assert.Contains(model.ComponentDiagnostics, row => row.Value == BrowserComponentCatalog.Current?.Sha256);
            var copyProfile = Assert.Single(panel.GetVisualDescendants().OfType<Button>(), button => ReferenceEquals(button.Command, model.CopyProfilePathCommand));
            Assert.True(copyProfile.IsEffectivelyVisible);
            Assert.True(copyProfile.IsEffectivelyEnabled);
            Assert.Contains(panel.GetVisualDescendants().OfType<TextBlock>(), text => text.IsEffectivelyVisible && text.Text == model.VisibilityText);
            var uninstall = Assert.Single(panel.GetVisualDescendants().OfType<Button>(), button => ReferenceEquals(button.Command, model.UninstallComponentCommand));
            Assert.False(uninstall.IsVisible);
            if (!failed)
            {
                await service.InstallAsync(BrowserComponentTests.Network);
                model.RefreshPresentation();
                Dispatcher.UIThread.RunJobs();
                Assert.True(uninstall.IsVisible);
                Assert.True(uninstall.IsEnabled);
                service.MarkLoaded(service.RequireReady());
                await service.UninstallAsync();
                model.RefreshPresentation();
                Dispatcher.UIThread.RunJobs();
                window.UpdateLayout();
                Assert.True(model.ComponentRestartRequired);
                Assert.False(download.IsVisible);
                Assert.False(uninstall.IsVisible);
                Assert.False(source.IsEffectivelyEnabled);
                Assert.True(scroll.Extent.Width <= scroll.Viewport.Width + 1);
            }
        }
        finally
        {
            window.Close();
            application.Resources.MergedDictionaries.Remove(language); application.Resources.MergedDictionaries.Remove(resources);
            application.Styles.Remove(styles); application.Styles.Remove(theme);
        }
    });
}
