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
            Layout = { PreviewVisible = true }
        };
        viewModel.Projects.Add(new ProjectItem("project", "Project", "/tmp/project"));
        viewModel.SetSelectedProjectForTests(viewModel.Projects[0]);
        viewModel.Layout.UpdateLayoutSize(1440, 900);

        Assert.True(viewModel.Layout.EffectivePreviewVisible);
        Assert.True(viewModel.Layout.PreviewChatVisible);
        Assert.False(viewModel.Layout.EffectiveNavigationVisible);
        Assert.False(viewModel.Layout.EffectiveReviewVisible);
        Assert.Equal(2, viewModel.Layout.ConversationWidth.Value);
        Assert.Equal(3, viewModel.Layout.PreviewWidth.Value);
    }

    [Fact]
    public void PreviewDropsChatAtNarrowWidth()
    {
        using var viewModel = new DesktopViewModel { Layout = { PreviewVisible = true } };
        viewModel.Layout.UpdateLayoutWidth(900);

        Assert.False(viewModel.Layout.PreviewChatVisible);
        Assert.Equal(0, viewModel.Layout.ConversationWidth.Value);
        Assert.Equal(0, viewModel.Layout.PreviewGap.Value);
    }
}
