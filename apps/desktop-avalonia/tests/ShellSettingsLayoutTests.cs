using Avalonia;
using Avalonia.Controls;
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
public sealed class ShellSettingsLayoutTests(AvaloniaUiFixture ui)
{
    [Theory]
    [InlineData(628, 552, false, "en-US")]
    [InlineData(448, 432, true, "zh-CN")]
    [InlineData(448, 432, false, "en-US")]
    [InlineData(628, 552, true, "zh-CN")]
    public Task ExistingInstallationFitsAndDownloadControlsStayHidden(int width, int height, bool dark, string locale) => ui.Run(() =>
    {
        var application = Application.Current!;
        var theme = new FluentTheme();
        var styles = new StyleInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/ControlStyles.axaml") };
        var resources = new ResourceInclude(new Uri("avares://SunCode/")) { Source = new("avares://SunCode/Styles/Application/ThemeResources.axaml") };
        var language = LocalizationService.CreateDictionary(locale);
        application.Styles.Add(theme); application.Styles.Add(styles);
        application.Resources.MergedDictionaries.Add(resources); application.Resources.MergedDictionaries.Add(language);
        using var model = new ShellSettingsViewModel(new FakeViewModelHost());
        var existing = PowerShellDiscoveryTests.Available(@"C:\Program Files\PowerShell\7\a-long-installation-directory\pwsh.exe");
        model.ApplySettings(new(existing, existing, null, "unused", "windows", "x64", existing));
        var panel = new TerminalSettingsControl { DataContext = model };
        var window = new Window { Width = width, Height = height, Content = panel, RequestedThemeVariant = dark ? ThemeVariant.Dark : ThemeVariant.Light };
        try
        {
            window.Show(); Dispatcher.UIThread.RunJobs(); window.UpdateLayout();
            var button = Assert.Single(panel.GetVisualDescendants().OfType<Button>(), item => ReferenceEquals(item.Command, model.UseExistingCommand));
            Assert.True(button.IsEffectivelyVisible);
            Assert.False(button.IsEffectivelyEnabled);
            Assert.InRange(button.Bounds.Width, 1, width - 32);
            var download = Assert.Single(panel.GetVisualDescendants().OfType<Button>(), item => ReferenceEquals(item.Command, model.InstallCommand));
            Assert.False(download.IsEffectivelyVisible);
            var scroll = panel.GetVisualDescendants().OfType<ScrollViewer>().First();
            Assert.True(scroll.Extent.Width <= scroll.Viewport.Width + 1);
            var path = Assert.Single(panel.GetVisualDescendants().OfType<TextBlock>(), item => item.Text == existing.Path && item.IsEffectivelyVisible);
            Assert.InRange(path.Bounds.Width, 1, width - 32);
        }
        finally
        {
            window.Close();
            application.Resources.MergedDictionaries.Remove(language); application.Resources.MergedDictionaries.Remove(resources);
            application.Styles.Remove(styles); application.Styles.Remove(theme);
        }
    });
}
