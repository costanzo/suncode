using SunCode.Desktop.Views.ProjectWorkspace.Preview;
using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Models;

namespace SunCode.Desktop.Tests;

public sealed class BrowserPreviewTests
{
    [Theory]
    [InlineData("http://localhost:5173/")]
    [InlineData("http://127.0.0.1:4173/app")]
    public void PreviewAcceptsLoopbackHttpUrls(string url)
        => Assert.True(BrowserPreviewPane.IsAllowedPreviewUrl(url));

    [Theory]
    [InlineData("https://localhost:5173/")]
    [InlineData("http://localhost:5173@evil.example/")]
    [InlineData("http://192.168.1.10:5173/")]
    [InlineData("file:///tmp/index.html")]
    public void PreviewRejectsUnsafeUrls(string url)
        => Assert.False(BrowserPreviewPane.IsAllowedPreviewUrl(url));

    [Fact]
    public void PreviewUsesWiderRightPaneAndSuppressesSecondaryBays()
    {
        using var viewModel = new DesktopViewModel
        {
            PreviewVisible = true
        };
        viewModel.Projects.Add(new ProjectItem("project", "Project", "/tmp/project"));
        viewModel.SetSelectedProjectForTests(viewModel.Projects[0]);
        viewModel.UpdateLayoutSize(1440, 900);

        Assert.True(viewModel.EffectivePreviewVisible);
        Assert.True(viewModel.PreviewChatVisible);
        Assert.False(viewModel.EffectiveNavigationVisible);
        Assert.False(viewModel.EffectiveReviewVisible);
        Assert.Equal(2, viewModel.ConversationWidth.Value);
        Assert.Equal(3, viewModel.PreviewWidth.Value);
    }

    [Fact]
    public void PreviewDropsChatAtNarrowWidth()
    {
        using var viewModel = new DesktopViewModel { PreviewVisible = true };
        viewModel.UpdateLayoutWidth(900);

        Assert.False(viewModel.PreviewChatVisible);
        Assert.Equal(0, viewModel.ConversationWidth.Value);
        Assert.Equal(0, viewModel.PreviewGap.Value);
    }
}
