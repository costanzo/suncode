using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Tests;

public sealed class LanguageServersViewModelTests
{
    [Fact]
    public async Task MutationsFailWithoutSdkAndLeaveStateUntouched()
    {
        var host = new FakeViewModelHost();
        var viewModel = new LanguageServersViewModel(host);

        Assert.False(await viewModel.RetryServerAsync(null!));
        Assert.False(await viewModel.DeleteServerAsync(null!));
        await viewModel.LoadServersAsync();
        await viewModel.StartProjectAsync();

        Assert.Empty(viewModel.Servers);
        Assert.False(viewModel.HasServers);
        Assert.True(viewModel.HasNoServers);
        Assert.Equal(string.Empty, viewModel.StatusText);
        Assert.Empty(host.Errors);
    }

    [Fact]
    public void WindowViewModelOwnsOneLanguageServersChild()
    {
        using var window = new DesktopViewModel();

        Assert.NotNull(window.LanguageServers);
        Assert.Same(window.LanguageServers, window.LanguageServers);
        Assert.Empty(window.LanguageServers.Servers);
    }
}
