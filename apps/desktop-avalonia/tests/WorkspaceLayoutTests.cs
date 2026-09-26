using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Models;

namespace SunCode.Desktop.Tests;

public sealed class WorkspaceLayoutTests
{
    [Fact]
    public void RunningSessionFooterSwitchesBetweenQuietAndPulsingDotStates()
    {
        using var viewModel = new DesktopViewModel();

        Assert.Equal("0 running", viewModel.RunningSessionText);
        Assert.Equal("quiet", viewModel.RunningSessionDotStatus);

        viewModel.Sessions.Add(new SessionItem("active", "Active", "", false, "running"));

        Assert.Equal("1 running", viewModel.RunningSessionText);
        Assert.Equal("running", viewModel.RunningSessionDotStatus);

        viewModel.Sessions.Clear();

        Assert.Equal("0 running", viewModel.RunningSessionText);
        Assert.Equal("quiet", viewModel.RunningSessionDotStatus);
    }

    [Fact]
    public void ResponsiveLayoutSuppressesSecondarySurfacesWithoutChangingUserPreferences()
    {
        using var viewModel = new DesktopViewModel
        {
            NavigationVisible = true,
            ReviewVisible = true,
            GitVisible = true
        };

        viewModel.UpdateLayoutWidth(1000);

        Assert.True(viewModel.EffectiveNavigationVisible);
        Assert.False(viewModel.EffectiveReviewVisible);
        Assert.True(viewModel.EffectiveGitVisible);
        Assert.Equal(4, viewModel.NavigationGap.Value);
        Assert.Equal(0, viewModel.ReviewGap.Value);
        Assert.Equal(4, viewModel.BottomDrawerGap.Value);
        Assert.True(viewModel.NavigationVisible);
        Assert.True(viewModel.ReviewVisible);

        viewModel.UpdateLayoutWidth(800);

        Assert.False(viewModel.EffectiveNavigationVisible);
        Assert.False(viewModel.EffectiveReviewVisible);
        Assert.True(viewModel.EffectiveGitVisible);
        Assert.True(viewModel.WorkspaceGuttersVisible);

        viewModel.UpdateLayoutWidth(620);

        Assert.False(viewModel.EffectiveGitVisible);
        Assert.False(viewModel.WorkspaceGuttersVisible);
        Assert.Equal(0, viewModel.WorkspaceGutterWidth.Value);
        Assert.Equal(0, viewModel.WorkspaceGutterGap.Value);
        Assert.Equal(0, viewModel.NavigationGap.Value);
        Assert.Equal(0, viewModel.ReviewGap.Value);
        Assert.Equal(0, viewModel.BottomDrawerGap.Value);
        Assert.False(viewModel.WorkspaceStatusDetailsVisible);
        Assert.True(viewModel.GitVisible);

        viewModel.UpdateLayoutWidth(1440);

        Assert.True(viewModel.EffectiveNavigationVisible);
        Assert.True(viewModel.EffectiveReviewVisible);
        Assert.True(viewModel.EffectiveGitVisible);
        Assert.Equal(34, viewModel.WorkspaceGutterWidth.Value);
        Assert.Equal(4, viewModel.WorkspaceGutterGap.Value);
    }

    [Fact]
    public void ExplicitlyHiddenPanesStayHiddenAcrossResponsiveChanges()
    {
        using var viewModel = new DesktopViewModel
        {
            NavigationVisible = false,
            ReviewVisible = false,
            ProviderTraceVisible = true
        };

        viewModel.UpdateLayoutWidth(620);
        viewModel.UpdateLayoutWidth(1440);

        Assert.False(viewModel.EffectiveNavigationVisible);
        Assert.False(viewModel.EffectiveReviewVisible);
        Assert.True(viewModel.EffectiveProviderTraceVisible);
    }

    [Fact]
    public void TogglingADrawerNotifiesItsEffectiveVisibilityAndGap()
    {
        using var viewModel = new DesktopViewModel();
        viewModel.UpdateLayoutWidth(1440);
        var changed = new List<string?>();
        viewModel.PropertyChanged += (_, args) => changed.Add(args.PropertyName);

        viewModel.GitVisible = true;

        Assert.Contains(nameof(DesktopViewModel.EffectiveGitVisible), changed);
        Assert.Contains(nameof(DesktopViewModel.BottomDrawerGap), changed);
        Assert.Equal(4, viewModel.BottomDrawerGap.Value);
    }

    [Fact]
    public void ReviewAndChildSessionPanelsAreMutuallyExclusive()
    {
        using var viewModel = new DesktopViewModel { ReviewVisible = true };

        viewModel.ChildSessionsVisible = true;

        Assert.False(viewModel.ReviewVisible);
        Assert.True(viewModel.ChildSessionsVisible);
        Assert.True(viewModel.EffectiveChildSessionsVisible);

        viewModel.ReviewVisible = true;

        Assert.True(viewModel.ReviewVisible);
        Assert.False(viewModel.ChildSessionsVisible);
        Assert.True(viewModel.EffectiveReviewInspectorVisible);
    }
}
