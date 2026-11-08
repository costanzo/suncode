using Avalonia;
using Avalonia.Controls;
using Avalonia.Markup.Xaml.Styling;
using Avalonia.Styling;
using Avalonia.Themes.Fluent;
using Avalonia.Threading;
using Avalonia.VisualTree;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Desktop.Views.Settings.Controls;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Tests;

[Collection("Avalonia UI")]
public sealed class ProviderCredentialTests(AvaloniaUiFixture ui)
{
    [Fact]
    public void CredentialRequestContainsOnlyProviderAndKey()
    {
        var request = new SetCredentialRequest("openai", "test-key");
        Assert.Equal(["Provider", "ApiKey"], typeof(SetCredentialRequest).GetProperties().Select(property => property.Name));
        Assert.Equal("openai", request.Provider);
    }

    [Theory]
    [InlineData(628, 552, false, "en-US")]
    [InlineData(448, 432, true, "zh-CN")]
    [InlineData(448, 432, false, "en-US")]
    [InlineData(628, 552, true, "zh-CN")]
    public Task SystemCredentialFormPreservesStatesAndFits(int width, int height, bool dark, string locale) => ui.Run(() =>
    {
        var application = Application.Current!;
        var theme = new FluentTheme();
        var styles = new StyleInclude(new Uri("avares://SunCode/"))
        {
            Source = new("avares://SunCode/Styles/Application/ControlStyles.axaml"),
        };
        var resources = new ResourceInclude(new Uri("avares://SunCode/"))
        {
            Source = new("avares://SunCode/Styles/Application/ThemeResources.axaml"),
        };
        var language = LocalizationService.CreateDictionary(locale);
        application.Styles.Add(theme);
        application.Styles.Add(styles);
        application.Resources.MergedDictionaries.Add(resources);
        application.Resources.MergedDictionaries.Add(language);
        var panel = new ModelProviderSettingsControl
        {
            Providers = [new ProviderItem("openai", "OpenAI", false)],
            SelectedProviderId = "openai",
        };
        var scroll = new ScrollViewer { Content = panel, HorizontalScrollBarVisibility = Avalonia.Controls.Primitives.ScrollBarVisibility.Disabled };
        var window = new Window
        {
            Width = width,
            Height = height,
            Content = scroll,
            RequestedThemeVariant = dark ? ThemeVariant.Dark : ThemeVariant.Light,
        };
        try
        {
            window.Show();
            Dispatcher.UIThread.RunJobs();
            window.UpdateLayout();
            Assert.Empty(panel.GetVisualDescendants().OfType<RadioButton>());
            var save = panel.FindControl<Button>("SaveCredentialButton")!;
            var cancel = panel.FindControl<Button>("CancelCredentialButton")!;
            var retry = panel.FindControl<Button>("RetryCredentialButton")!;
            var remove = panel.FindControl<Button>("RemoveCredentialButton")!;
            var input = panel.FindControl<TextBox>("ApiKeyInput")!;
            var location = panel.FindControl<TextBlock>("CredentialLocationTextBlock")!;
            Assert.False(save.IsEffectivelyEnabled);
            Assert.False(cancel.IsVisible);
            Assert.False(remove.IsEffectivelyEnabled);
            Assert.Contains(OperatingSystem.IsMacOS() ? "macOS Keychain" : OperatingSystem.IsWindows() ? "Windows Credential Manager" : "Linux Secret Service", location.Text);
            Assert.NotEqual('\0', input.PasswordChar);

            panel.CredentialMask = "sk-t••••••••6789";
            panel.CredentialConfigured = true;
            panel.CredentialStatus = "configured";
            panel.CanRemoveCredential = true;
            Assert.True(remove.IsEffectivelyEnabled);
            Assert.False(retry.IsVisible);
            foreach (var status in new[] { "locked", "access_denied", "unavailable", "missing" })
            {
                panel.CredentialConfigured = false;
                panel.CredentialStatus = status;
                Assert.Equal(status != "missing", retry.IsVisible);
                Assert.Equal("sk-t••••••••6789", panel.FindControl<TextBlock>("CredentialMaskTextBlock")!.Text);
            }
            panel.ApiKeyText = "replacement-test-key";
            panel.CanSaveCredential = true;
            Assert.True(save.IsEffectivelyEnabled);
            Assert.True(cancel.IsVisible);
            panel.SetCredentialBusy(true);
            Assert.False(save.IsEffectivelyEnabled);
            Assert.False(remove.IsEffectivelyEnabled);
            Assert.False(input.IsEffectivelyEnabled);
            Assert.False(cancel.IsEffectivelyEnabled);
            panel.SetCredentialBusy(false);
            Assert.True(save.IsEffectivelyEnabled);
            panel.SetCredentialFeedback("Credential service unavailable", true);
            Assert.True(panel.FindControl<TextBlock>("CredentialFeedbackTextBlock")!.IsVisible);
            panel.SetCredentialFeedback(string.Empty);
            Assert.False(panel.FindControl<TextBlock>("CredentialFeedbackTextBlock")!.IsVisible);
            panel.ApiKeyText = string.Empty;
            panel.CanSaveCredential = false;
            Assert.False(cancel.IsVisible);
            Assert.False(save.IsEffectivelyEnabled);
            Dispatcher.UIThread.RunJobs();
            window.UpdateLayout();
            Assert.True(scroll.Extent.Width <= scroll.Viewport.Width + 1);
            Assert.InRange(input.Bounds.Width, 1, width);
        }
        finally
        {
            window.Close();
            application.Resources.MergedDictionaries.Remove(language);
            application.Resources.MergedDictionaries.Remove(resources);
            application.Styles.Remove(styles);
            application.Styles.Remove(theme);
        }
    });
}
