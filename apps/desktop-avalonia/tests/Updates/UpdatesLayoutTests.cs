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
        }
        finally
        {
            window.Close();application.Resources.MergedDictionaries.Remove(language);application.Resources.MergedDictionaries.Remove(resources);application.Styles.Remove(styles);application.Styles.Remove(theme);
        }
    });
}
