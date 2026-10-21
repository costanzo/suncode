using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Sdk;

namespace SunCode.Desktop.Tests;

public sealed class PreviewFailureTests
{
    [Theory]
    [InlineData("start")]
    [InlineData("refresh")]
    [InlineData("stop")]
    public async Task PreviewFailureDoesNotDisableConversationOrProjectControls(string operation)
    {
        using var viewModel = new DesktopViewModel();
        ((IViewModelHost)viewModel).ReportSuccess("Connected");
        typeof(DesktopViewModel).GetProperty(nameof(DesktopViewModel.SelectedSession))!
            .SetValue(viewModel, new SessionItem("session", "Session", "now", false));
        viewModel.SelectedModel = new ModelItem("test-model", "test", "Test", "configured", true);
        viewModel.ComposerText = "Keep this draft";

        Assert.True(viewModel.CanCompose);
        Assert.True(viewModel.CanChooseModel);
        Assert.True(viewModel.CanOpenProjects);
        Assert.True(viewModel.CanSubmit);

        var succeeded = await viewModel.RunPreviewOperationAsync(operation, async () =>
        {
            await Task.Yield();
            throw new SdkException("preview_failed", "process executable could not be found");
        });

        Assert.False(succeeded);
        Assert.StartsWith($"Could not {operation} project preview:", viewModel.StatusText);
        Assert.Contains("process executable could not be found", viewModel.StatusText);
        Assert.Equal("connected", viewModel.ConnectionState);
        Assert.True(viewModel.CanCompose);
        Assert.True(viewModel.CanChooseModel);
        Assert.True(viewModel.CanOpenProjects);
        Assert.True(viewModel.CanSubmit);
        Assert.Equal("Keep this draft", viewModel.ComposerText);
    }
}
