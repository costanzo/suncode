using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Tests;

public sealed class SettingsNavigationViewModelTests
{
    [Fact]
    public void NavigateSelectsPageAndHighlightsItsEntry()
    {
        var (navigation, _, saved) = Create();
        navigation.Restore(new UiSettingsState());

        navigation.NavigateCommand.Execute(SettingsPage.Network);

        Assert.Equal(SettingsPage.Network, navigation.SelectedPage);
        Assert.Equal(SettingsPage.Network, navigation.HighlightedNavigation);
        Assert.Equal("network", saved[^1].Page);
    }

    [Fact]
    public void ProviderSubEntryReplacesTopLevelHighlight()
    {
        var (navigation, _, saved) = Create();
        navigation.Restore(new UiSettingsState());
        var shown = new List<string?>();
        navigation.ProviderPanelShown += shown.Add;

        navigation.ShowProviderCommand.Execute("openai");

        Assert.Equal(SettingsPage.Providers, navigation.SelectedPage);
        Assert.Null(navigation.HighlightedNavigation);
        Assert.Equal("openai", navigation.HighlightedProviderId);
        Assert.Equal("openai", navigation.SelectedProviderId);
        Assert.Equal(["openai"], shown);
        Assert.Equal("openai", saved[^1].ProviderId);
    }

    [Fact]
    public void ProvidersEntryShowsOverviewAndTogglesExpansion()
    {
        var (navigation, _, _) = Create();
        navigation.Restore(new UiSettingsState());
        navigation.ShowProvider("openai");

        navigation.Navigate(SettingsPage.Providers);

        Assert.True(navigation.ProvidersExpanded);
        Assert.Null(navigation.SelectedProviderId);
        Assert.Equal(SettingsPage.Providers, navigation.HighlightedNavigation);
        Assert.Equal("openai", navigation.LastProviderId);

        navigation.Navigate(SettingsPage.Providers);
        Assert.False(navigation.ProvidersExpanded);
    }

    [Fact]
    public void AgentsEntryClearsSelectedAgentAndTogglesExpansion()
    {
        var (navigation, _, _) = Create();
        navigation.Restore(new UiSettingsState());
        navigation.ShowAgentCommand.Execute("reviewer");
        Assert.Equal("reviewer", navigation.SelectedAgentId);
        Assert.Equal("reviewer", navigation.HighlightedAgentId);

        navigation.Navigate(SettingsPage.Agents);

        Assert.Null(navigation.SelectedAgentId);
        Assert.True(navigation.AgentsExpanded);
        Assert.Equal(SettingsPage.Agents, navigation.HighlightedNavigation);
    }

    [Fact]
    public void RestoreAppliesPersistedStateWithoutSavingIt()
    {
        var (navigation, _, saved) = Create();

        navigation.Restore(new UiSettingsState
        {
            Page = "providers",
            ProviderId = "anthropic",
            ProvidersExpanded = true,
            AgentsExpanded = true
        });

        Assert.Empty(saved);
        Assert.Equal(SettingsPage.Providers, navigation.SelectedPage);
        Assert.Equal("anthropic", navigation.SelectedProviderId);
        Assert.Equal(SettingsPage.Providers, navigation.HighlightedNavigation);
        Assert.True(navigation.ProvidersExpanded);
        Assert.True(navigation.AgentsExpanded);
    }

    [Theory]
    [InlineData("lsp", SettingsPage.LanguageServers)]
    [InlineData("mcp", SettingsPage.Mcp)]
    [InlineData("logging", SettingsPage.Logging)]
    [InlineData("unknown", SettingsPage.Defaults)]
    [InlineData("3", SettingsPage.Defaults)]
    [InlineData(null, SettingsPage.Defaults)]
    public void PageKeysRoundTripAndFallBackToDefaults(string? key, SettingsPage expected)
    {
        Assert.Equal(expected, SettingsPageKeys.FromKey(key));
        Assert.Equal(expected, SettingsPageKeys.FromKey(SettingsPageKeys.ToKey(expected)));
    }

    [Fact]
    public void OnlyTheSelectedPollingPageIsActive()
    {
        var mcp = new RecordingPage();
        var computer = new RecordingPage();
        var timer = new FakePollTimer();
        using var poller = new PagePoller(timer);
        var navigation = new SettingsNavigationViewModel(
            poller,
            page => page switch { SettingsPage.Mcp => mcp, SettingsPage.Computer => computer, _ => null });
        navigation.Restore(new UiSettingsState { Page = "mcp" });

        Assert.Same(mcp, poller.Active);
        Assert.True(timer.Running);
        navigation.Navigate(SettingsPage.Computer);
        Assert.Equal(1, mcp.Deactivations);
        Assert.Same(computer, poller.Active);

        navigation.Navigate(SettingsPage.Appearance);
        Assert.Null(poller.Active);
        Assert.False(timer.Running);
        Assert.Equal(1, computer.Deactivations);
    }

    private static (SettingsNavigationViewModel Navigation, PagePoller Poller, List<UiSettingsState> Saved) Create()
    {
        var saved = new List<UiSettingsState>();
        var poller = new PagePoller(new FakePollTimer());
        return (new SettingsNavigationViewModel(poller, _ => null, saved.Add), poller, saved);
    }
}
