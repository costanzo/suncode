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
    public void VtslsItemExposesServerSpecificConfiguration()
    {
        var item = new LanguageServerItem(new LanguageServer(
            "vtsls", "vtsls", "vtsls", "TypeScript semantic analysis", "JavaScript / TypeScript",
            true, null, "/usr/local/bin/vtsls", null, true, "/usr/local/bin/vtsls", ["--stdio"],
            ["javascript", "typescript"], ["package.json"], JsonDocument.Parse("{}").RootElement.Clone(),
            [], 120, 30, null, string.Empty, string.Empty, "package.json", false, 10, 1, "disabled",
            0, null, string.Empty, string.Empty,
            new VtslsLanguageServerConfiguration(
                "/usr/local/bin/vtsls", "/usr/local/bin/node", "/usr/local/bin/node", "22.0.0",
                "workspace-preferred", null, 3072)));

        Assert.True(item.IsVtsls);
        Assert.Equal("/usr/local/bin/node", item.NodePath);
        Assert.Equal("workspace-preferred", item.TypeScriptSdkMode);
        Assert.Equal(3072UL, item.MaxTsServerMemory);
    }

    [Fact]
    public void BasedpyrightItemExposesServerSpecificConfiguration()
    {
        var item = new LanguageServerItem(new LanguageServer(
            "basedpyright", "basedpyright", "basedpyright", "Python semantic analysis", "Python",
            true, null, "/opt/homebrew/bin/basedpyright-langserver", null, true,
            "/opt/homebrew/bin/basedpyright-langserver", ["--stdio"], ["python"], ["pyproject.toml"],
            JsonDocument.Parse("{}").RootElement.Clone(), [], 60, 30, null, string.Empty, string.Empty,
            "pyproject.toml", false, 20, 1, "disabled", 0, null, string.Empty, string.Empty, null,
            new BasedpyrightLanguageServerConfiguration(
                "/opt/homebrew/bin/basedpyright-langserver", "/project/.venv/bin/python",
                "/project/.venv/bin/python", "3.12.7", "openFilesOnly", "recommended",
                "automatic", null)));

        Assert.True(item.IsBasedpyright);
        Assert.Equal("/project/.venv/bin/python", item.PythonPath);
        Assert.Equal("openFilesOnly", item.DiagnosticMode);
        Assert.Equal("recommended", item.TypeCheckingMode);
        Assert.Equal("automatic", item.ConfigPathMode);
    }

    [Fact]
    public void RustAnalyzerItemExposesServerSpecificConfiguration()
    {
        var item = new LanguageServerItem(new LanguageServer(
            "rust-analyzer", "rust-analyzer", "rust-analyzer", "Rust semantic analysis", "Rust",
            true, "1.95.0", "/opt/homebrew/bin/rust-analyzer", null, true,
            "/opt/homebrew/bin/rust-analyzer", [], ["rust"], ["Cargo.toml"],
            JsonDocument.Parse("{}").RootElement.Clone(), [], 120, 30, null, string.Empty,
            string.Empty, "Cargo.toml", false, 30, 1, "disabled", 0, null, string.Empty,
            string.Empty, null, null,
            new RustAnalyzerLanguageServerConfiguration(
                "/opt/homebrew/bin/rust-analyzer", "1.95.0", "1.95.0", "custom",
                ["serde"], true, true, true, true, "clippy", "custom",
                "wasm32-unknown-unknown")));

        Assert.True(item.IsRustAnalyzer);
        Assert.Equal("custom", item.CargoFeatureMode);
        Assert.Equal(["serde"], item.CargoFeatures);
        Assert.True(item.AllCargoTargets);
        Assert.Equal("clippy", item.RustCheckCommand);
        Assert.Equal("wasm32-unknown-unknown", item.RustTarget);
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
