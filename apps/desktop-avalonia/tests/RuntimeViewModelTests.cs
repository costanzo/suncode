using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Tests;

public sealed class RuntimeViewModelTests
{
    [Fact]
    public async Task BrowserProjectActionsRequireAnOpenProject()
    {
        var browser = new BrowserRuntimeViewModel(new FakeViewModelHost());

        Assert.False(await browser.StartBrowserProjectAsync());
        Assert.Equal("Open a project to manage its browser runtime.", browser.StatusText);
        Assert.Null(browser.Runtime);
    }

    [Fact]
    public async Task BrowserActionsFailWithoutSdk()
    {
        var host = new FakeViewModelHost { SelectedProject = new ProjectItem("project", "Project", "/tmp/project") };
        var browser = new BrowserRuntimeViewModel(host);

        Assert.False(await browser.SetBrowserUseEnabledAsync(true));
        Assert.False(await browser.StopBrowserRuntimeAsync());
        Assert.False(await browser.ClearBrowserProfileAsync());
        await browser.LoadAsync();

        Assert.Null(browser.Runtime);
        Assert.Equal(string.Empty, browser.StatusText);
        Assert.Empty(host.Errors);
    }

    [Fact]
    public async Task ComputerActionsFailWithoutSdk()
    {
        var host = new FakeViewModelHost();
        var computer = new ComputerRuntimeViewModel(host);

        Assert.False(await computer.SetComputerUseEnabledAsync(true));
        Assert.False(await computer.EmergencyStopComputerUseAsync());
        Assert.False(await computer.TakeComputerControlAsync());
        await computer.LoadAsync();

        Assert.Null(computer.Runtime);
        Assert.Equal(string.Empty, computer.StatusText);
        Assert.Empty(host.Errors);
    }
}
