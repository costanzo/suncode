using Avalonia;
using Avalonia.Controls;
using Avalonia.Markup.Xaml.Styling;
using Avalonia.Styling;
using Avalonia.Themes.Fluent;
using Avalonia.Threading;
using Avalonia.VisualTree;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Infrastructure.Updates;
using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Views.Settings.Controls;

namespace SunCode.Desktop.Tests.Updates;

[Collection("Avalonia UI")]
public sealed class UpdatesLayoutTests(AvaloniaUiFixture ui)
{
    [Theory]
    [InlineData("en-US", "check_timed_out", "Checking for updates timed out", "Check your connection and try again.", "Check for updates")]
    [InlineData("zh-CN", "check_timed_out", "检查更新超时", "请检查网络连接后重试。", "检查更新")]
    [InlineData("en-US", "download_timed_out", "The download timed out", "Try downloading again.", "Download again")]
    [InlineData("zh-CN", "download_timed_out", "下载更新超时", "请重新下载。", "重新下载")]
    [InlineData("en-US", "download_stalled", "The download stopped responding", "No data was received for 30 seconds.", "Download again")]
    [InlineData("zh-CN", "download_stalled", "下载长时间无响应", "连续 30 秒未收到数据。", "重新下载")]
    public Task TimeoutErrorsUseLocalizedRecoveryAndKeepRetryAvailable(string locale, string code, string heading, string hint, string action) => ui.Run(async () =>
    {
        using var f = new UpdateFixture();
        using var coordinator = new UpdateCoordinator(f.Distribution, f.Store, f.Environment, new StubPlatformVerifier(), f.Handler, f.Time);
        using var model = new UpdatesSettingsViewModel(coordinator, item => item(), () => locale);
        var dictionary = LocalizationService.CreateDictionary(locale);
        Application.Current!.Resources.MergedDictionaries.Add(dictionary);
        try
        {
            if (code != "check_timed_out") await coordinator.CheckAsync();
            f.Handler.Respond = (_, _) => throw new UpdateException(code, "Test timeout");
            if (code == "check_timed_out") await coordinator.CheckAsync(); else await coordinator.DownloadAsync();
            Assert.Equal(heading, model.Heading); Assert.Contains(hint, model.Description);
            Assert.Equal(action, model.PrimaryText); Assert.True(model.PrimaryCommand.CanExecute(null));
            Assert.Equal(StatusTone.Danger, model.Tone); Assert.False(model.IsReady);
            Assert.Null(f.Store.Read().Staged); Assert.Equal("0.1.0", f.Store.Read().Installation!.Sdk!.Version);
        }
        finally { Application.Current!.Resources.MergedDictionaries.Remove(dictionary); }
    });
    [Theory]
    [InlineData("en-US", "Automatic installation is unavailable for this build", "You can keep using SunCode.")]
    [InlineData("zh-CN", "此构建暂不支持自动安装更新", "你可以继续使用 SunCode。")]
    public Task UnmanagedBuildUsesLocalizedWarningInsteadOfRepairText(string locale, string heading, string hint) => ui.Run(async () =>
    {
        using var f = new UpdateFixture();
        f.Store.Change(state => state.Installation = null);
        using var coordinator = new UpdateCoordinator(f.Distribution, f.Store, f.Environment, new StubPlatformVerifier(), f.Handler, f.Time);
        coordinator.ReadRunningVersionsAsync = () => Task.FromResult(new RunningUpdateVersions("0.1.0", 27, "0.1.0", 27));
        using var model = new UpdatesSettingsViewModel(coordinator, action => action(), () => locale);
        var dictionary = LocalizationService.CreateDictionary(locale);
        var application = Application.Current!;
        var theme = new FluentTheme();
        var styles = new StyleInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/ControlStyles.axaml") };
        var resources = new ResourceInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/ThemeResources.axaml") };
        application.Styles.Add(theme); application.Styles.Add(styles);
        application.Resources.MergedDictionaries.Add(resources); application.Resources.MergedDictionaries.Add(dictionary);
        var panel = new UpdatesSettingsControl { DataContext = model };
        var window = new Window { Width = 662, Height = 586, Content = panel, RequestedThemeVariant = locale == "zh-CN" ? ThemeVariant.Dark : ThemeVariant.Light };
        try
        {
            await coordinator.CheckAsync();
            Assert.Equal(heading, model.Heading);
            Assert.Contains(hint, model.Description);
            Assert.Equal(StatusTone.Warning, model.Tone);
            window.Show(); Dispatcher.UIThread.RunJobs(); window.UpdateLayout();
            var title = Assert.Single(panel.GetVisualDescendants().OfType<TextBlock>(), text => text.Text == heading);
            var expected = Assert.IsAssignableFrom<Avalonia.Media.ISolidColorBrush>(title.FindResource(window.ActualThemeVariant, "WarningBrush"));
            var actual = Assert.IsAssignableFrom<Avalonia.Media.ISolidColorBrush>(title.Foreground);
            Assert.Equal(expected.Color, actual.Color);
        }
        finally
        {
            window.Close(); application.Resources.MergedDictionaries.Remove(dictionary); application.Resources.MergedDictionaries.Remove(resources);
            application.Styles.Remove(styles); application.Styles.Remove(theme);
        }
    });
    [Theory]
    [InlineData("en-US", "The download was interrupted", "Try downloading again.")]
    [InlineData("zh-CN", "下载已中断", "请重新下载。")]
    public Task DownloadFailureUsesLocalizedRecoveryText(string locale, string heading, string hint) => ui.Run(async () =>
    {
        using var f = new UpdateFixture();
        using var coordinator = new UpdateCoordinator(f.Distribution, f.Store, f.Environment, new StubPlatformVerifier(), f.Handler, f.Time);
        using var model = new UpdatesSettingsViewModel(coordinator, action => action(), () => locale);
        var dictionary = LocalizationService.CreateDictionary(locale);
        Application.Current!.Resources.MergedDictionaries.Add(dictionary);
        try
        {
            await coordinator.CheckAsync();
            f.Handler.Respond = (_, _) => throw new HttpRequestException();
            await coordinator.DownloadAsync();
            Assert.Equal(heading, model.Heading);
            Assert.Contains(hint, model.Description);
        }
        finally { Application.Current!.Resources.MergedDictionaries.Remove(dictionary); }
    });
    [Theory]
    [InlineData(662,586,false,"en-US")]
    [InlineData(482,466,true,"zh-CN")]
    [InlineData(482,466,false,"en-US")]
    [InlineData(662,586,true,"zh-CN")]
    public Task VersionsActionsAndLocalizedNotesFitDefaultAndMinimumPanels(int width,int height,bool dark,string locale) => ui.Run(async () =>
    {
        using var f=new UpdateFixture();
        const string maximum="2147483647.2147483647.2147483647";
        f.Store.Change(s=>s.Installation=f.Installed with { Sdk=f.Installed.Sdk! with {Version="2147483646.2147483647.2147483647"} });
        f.Catalog=f.Catalog with {Components=f.Catalog.Components with {Sdk=f.Catalog.Components.Sdk with {Version=maximum}},CompatiblePairs=[new(){Desktop="0.1.0",Sdk=maximum}]};
        using var coordinator=new UpdateCoordinator(f.Distribution,f.Store,f.Environment,new StubPlatformVerifier(),f.Handler,f.Time);
        await coordinator.CheckAsync();
        var application=Application.Current!;
        var theme=new FluentTheme();
        var styles=new StyleInclude(new Uri("avares://SunCode/")){Source=new("avares://SunCode/Styles/Application/ControlStyles.axaml")};
        var resources=new ResourceInclude(new Uri("avares://SunCode/")){Source=new("avares://SunCode/Styles/Application/ThemeResources.axaml")};
        var language=LocalizationService.CreateDictionary(locale);
        application.Styles.Add(theme);application.Styles.Add(styles);application.Resources.MergedDictionaries.Add(resources);application.Resources.MergedDictionaries.Add(language);
        using var model=new UpdatesSettingsViewModel(coordinator,action=>action(),()=>locale);
        var panel=new UpdatesSettingsControl{DataContext=model};
        var window=new Window{Width=width,Height=height,Content=panel,RequestedThemeVariant=dark?ThemeVariant.Dark:ThemeVariant.Light};
        try
        {
            window.Show();Dispatcher.UIThread.RunJobs();window.UpdateLayout();
            var scroll=panel.GetVisualDescendants().OfType<ScrollViewer>().First();
            Assert.True(scroll.Extent.Width<=scroll.Viewport.Width+1,$"Horizontal overflow: {scroll.Extent.Width} > {scroll.Viewport.Width}");
            var primary=Assert.Single(panel.GetVisualDescendants().OfType<Button>(),button=>ReferenceEquals(button.Command,model.PrimaryCommand));
            Assert.True(primary.IsEffectivelyVisible);Assert.True(primary.IsEffectivelyEnabled);
            Assert.InRange(primary.Bounds.Width,1,width-32);
            Assert.Contains(locale=="zh-CN"?"改进 SDK":"SDK improvements",model.Notes);
            Assert.True(model.ShowSdkTarget);Assert.False(model.ShowDesktopTarget);
            var toggle = Assert.Single(panel.GetVisualDescendants().OfType<ToggleSwitch>());
            Assert.Contains("sc-toggle-switch", toggle.Classes);
            Assert.Null(toggle.OnContent); Assert.Null(toggle.OffContent);
            var track = Assert.Single(toggle.GetVisualDescendants().OfType<Border>(), border => border.Name == "OuterBorder");
            Assert.Equal(34, track.Bounds.Width); Assert.Equal(20, track.Bounds.Height);
            foreach (var knob in toggle.GetVisualDescendants().OfType<Avalonia.Controls.Shapes.Ellipse>().Where(ellipse => ellipse.Name is "SwitchKnobOn" or "SwitchKnobOff"))
            { Assert.Equal(14, knob.Width); Assert.Equal(14, knob.Height); }
            toggle.IsChecked = false; Dispatcher.UIThread.RunJobs();
            Assert.False(f.Store.Read().AutomaticChecks);
            toggle.IsChecked = true; Dispatcher.UIThread.RunJobs();
            Assert.True(f.Store.Read().AutomaticChecks);
        }
        finally
        {
            window.Close();application.Resources.MergedDictionaries.Remove(language);application.Resources.MergedDictionaries.Remove(resources);application.Styles.Remove(styles);application.Styles.Remove(theme);
        }
    });
}
