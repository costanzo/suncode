using System.Text.Json;
using SunCode.Desktop.Models;
using SunCode.Sdk.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Tests;

public sealed class LanguageServersViewModelTests
{
    [Fact]
    public void FailedStatusDetailUsesRuntimeError()
    {
        var item = new LanguageServerItem(new LanguageServer(
            "jdtls",
            "jdtls",
            "Eclipse JDT Language Server",
            "Java semantic analysis",
            "Java",
            true,
            "1.57.0",
            "/opt/homebrew/bin/jdtls",
            "/test/jdk",
            true,
            "/opt/homebrew/bin/jdtls",
            [],
            ["java"],
            ["pom.xml"],
            JsonDocument.Parse("{}").RootElement.Clone(),
            ["JAVA_HOME"],
            120,
            30,
            "/test/jdk",
            "managed",
            "/tmp/jdtls",
            "pom.xml",
            true,
            0,
            1,
            "failed",
            0,
            "Language server initialization timed out",
            string.Empty,
            string.Empty));

        Assert.Equal("Language server initialization timed out", item.StatusDetail);
    }

    [Fact]
    public async Task MutationsFailWithoutSdkAndLeaveStateUntouched()
    {
        var host = new FakeViewModelHost();
        var viewModel = new LanguageServersViewModel(host);

        Assert.False(await viewModel.RetryServerAsync(null!));
        await viewModel.LoadServersAsync();
        await viewModel.StartProjectAsync();

        Assert.Empty(viewModel.Servers);
        Assert.False(viewModel.HasProject);
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
