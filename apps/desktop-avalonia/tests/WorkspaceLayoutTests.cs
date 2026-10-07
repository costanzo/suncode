using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Models;

namespace SunCode.Desktop.Tests;

public sealed class WorkspaceLayoutTests
{
    private static WorkspaceLayoutViewModel CreateLayout(ProjectItem? project = null) =>
        new(new FakeViewModelHost { SelectedProject = project });

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
        var layout = CreateLayout();
        layout.NavigationVisible = true;
        layout.ReviewVisible = true;
        layout.GitVisible = true;

        layout.UpdateLayoutWidth(1000);

        Assert.True(layout.EffectiveNavigationVisible);
        Assert.False(layout.EffectiveReviewVisible);
        Assert.True(layout.EffectiveGitVisible);
        Assert.Equal(4, layout.NavigationGap.Value);
        Assert.Equal(0, layout.ReviewGap.Value);
        Assert.Equal(4, layout.BottomDrawerGap.Value);
        Assert.True(layout.NavigationVisible);
        Assert.True(layout.ReviewVisible);

        layout.UpdateLayoutWidth(800);

        Assert.False(layout.EffectiveNavigationVisible);
        Assert.False(layout.EffectiveReviewVisible);
        Assert.True(layout.EffectiveGitVisible);
        Assert.True(layout.WorkspaceGuttersVisible);

        layout.UpdateLayoutWidth(620);

        Assert.False(layout.EffectiveGitVisible);
        Assert.False(layout.WorkspaceGuttersVisible);
        Assert.Equal(0, layout.WorkspaceGutterWidth.Value);
        Assert.Equal(0, layout.WorkspaceGutterGap.Value);
        Assert.Equal(0, layout.NavigationGap.Value);
        Assert.Equal(0, layout.ReviewGap.Value);
        Assert.Equal(0, layout.BottomDrawerGap.Value);
        Assert.False(layout.WorkspaceStatusDetailsVisible);
        Assert.True(layout.GitVisible);

        layout.UpdateLayoutWidth(1440);

        Assert.True(layout.EffectiveNavigationVisible);
        Assert.True(layout.EffectiveReviewVisible);
        Assert.True(layout.EffectiveGitVisible);
        Assert.Equal(34, layout.WorkspaceGutterWidth.Value);
        Assert.Equal(4, layout.WorkspaceGutterGap.Value);
    }

    [Fact]
    public void ExplicitlyHiddenPanesStayHiddenAcrossResponsiveChanges()
    {
        var layout = CreateLayout();
        layout.NavigationVisible = false;
        layout.ReviewVisible = false;
        layout.ProviderTraceVisible = true;

        layout.UpdateLayoutWidth(620);
        layout.UpdateLayoutWidth(1440);

        Assert.False(layout.EffectiveNavigationVisible);
        Assert.False(layout.EffectiveReviewVisible);
        Assert.True(layout.EffectiveProviderTraceVisible);
    }

    [Fact]
    public void TogglingADrawerNotifiesItsEffectiveVisibilityAndGap()
    {
        var layout = CreateLayout();
        layout.UpdateLayoutWidth(1440);
        var changed = new List<string?>();
        layout.PropertyChanged += (_, args) => changed.Add(args.PropertyName);

        layout.GitVisible = true;

        Assert.Contains(nameof(WorkspaceLayoutViewModel.EffectiveGitVisible), changed);
        Assert.Contains(nameof(WorkspaceLayoutViewModel.BottomDrawerGap), changed);
        Assert.Equal(4, layout.BottomDrawerGap.Value);
    }

    [Fact]
    public void ReviewAndChildSessionPanelsAreMutuallyExclusive()
    {
        var layout = CreateLayout();
        layout.ReviewVisible = true;

        layout.ChildSessionsVisible = true;

        Assert.False(layout.ReviewVisible);
        Assert.True(layout.ChildSessionsVisible);
        Assert.True(layout.EffectiveChildSessionsVisible);
        Assert.False(layout.EffectiveReviewInspectorVisible);

        layout.ReviewVisible = true;

        Assert.True(layout.ReviewVisible);
        Assert.False(layout.ChildSessionsVisible);
        Assert.True(layout.EffectiveReviewInspectorVisible);
    }

    [Fact]
    public void ReviewGutterSwitchesAwayFromBrowserPreview()
    {
        var layout = CreateLayout(new ProjectItem("project", "Project", "/tmp/project"));
        layout.PreviewVisible = true;
        layout.ReviewVisible = true;
        layout.UpdateLayoutWidth(1440);

        var closedPreview = layout.ToggleReviewDestination();

        Assert.True(closedPreview);
        Assert.False(layout.PreviewVisible);
        Assert.True(layout.ReviewVisible);
        Assert.True(layout.EffectiveReviewVisible);
    }

    [Fact]
    public void ChildSessionsGutterSwitchesAwayFromBrowserPreview()
    {
        var layout = CreateLayout(new ProjectItem("project", "Project", "/tmp/project"));
        layout.PreviewVisible = true;
        layout.UpdateLayoutWidth(1440);

        var closedPreview = layout.ToggleChildSessionsDestination();

        Assert.True(closedPreview);
        Assert.False(layout.PreviewVisible);
        Assert.True(layout.ChildSessionsVisible);
        Assert.False(layout.ReviewVisible);
        Assert.True(layout.EffectiveChildSessionsVisible);
    }

    [Fact]
    public void TurnChangesStayHiddenUntilTheActiveTurnTouchesAFile()
    {
        using var viewModel = new DesktopViewModel();

        viewModel.ApplySnapshot(new SessionSnapshotProjection(
            [], [], [], [], [], null, null, "turn-1", "running"));

        Assert.False(viewModel.IsReviewChangesVisible);

        viewModel.ApplySnapshot(new SessionSnapshotProjection(
            [], [], [], ["src/App.cs"], [], null, null, "turn-1", "running"));

        Assert.True(viewModel.IsReviewChangesVisible);
    }

    [Theory]
    [InlineData(WorkspaceBottomDrawer.Git)]
    [InlineData(WorkspaceBottomDrawer.ProviderTrace)]
    [InlineData(WorkspaceBottomDrawer.ToolActivity)]
    public void OpeningABottomDrawerClosesTheOthers(WorkspaceBottomDrawer drawer)
    {
        var layout = CreateLayout();
        foreach (var other in Enum.GetValues<WorkspaceBottomDrawer>().Where(value => value != drawer))
            layout.ToggleBottomDrawer(other);

        var opened = layout.ToggleBottomDrawer(drawer);

        Assert.True(opened);
        foreach (var value in Enum.GetValues<WorkspaceBottomDrawer>())
            Assert.Equal(value == drawer, layout.IsBottomDrawerVisible(value));
    }

    [Fact]
    public void ClosingABottomDrawerLeavesTheOthersAndItsHeightAlone()
    {
        var layout = CreateLayout();
        layout.ToggleBottomDrawer(WorkspaceBottomDrawer.Git);
        layout.BottomDrawerHeight = 100;

        var opened = layout.ToggleBottomDrawer(WorkspaceBottomDrawer.Git);

        Assert.False(opened);
        Assert.False(layout.GitVisible);
        Assert.False(layout.ProviderTraceVisible);
        Assert.False(layout.ToolActivityVisible);
        Assert.Equal(100, layout.BottomDrawerHeight);
    }

    [Fact]
    public void OpeningACollapsedBottomDrawerRestoresTheDefaultHeight()
    {
        var layout = CreateLayout();
        layout.BottomDrawerHeight = WorkspaceLayoutViewModel.MinimumBottomDrawerHeight - 1;

        layout.ToggleBottomDrawer(WorkspaceBottomDrawer.ProviderTrace);

        Assert.Equal(WorkspaceLayoutViewModel.DefaultBottomDrawerHeight, layout.BottomDrawerHeight);
        Assert.True(layout.EffectiveProviderTraceVisible);
    }

    [Fact]
    public void OpeningABottomDrawerKeepsAUsableCustomHeight()
    {
        var layout = CreateLayout();
        layout.BottomDrawerHeight = WorkspaceLayoutViewModel.MinimumBottomDrawerHeight;

        layout.ToggleBottomDrawer(WorkspaceBottomDrawer.ToolActivity);

        Assert.Equal(WorkspaceLayoutViewModel.MinimumBottomDrawerHeight, layout.BottomDrawerHeight);
    }

    [Fact]
    public void SessionsToggleSwitchesFromExplorerBeforeHidingNavigation()
    {
        var layout = CreateLayout();
        layout.ExplorerVisible = true;

        layout.ToggleSessions();

        Assert.True(layout.NavigationVisible);
        Assert.False(layout.ExplorerVisible);

        layout.ToggleSessions();

        Assert.False(layout.NavigationVisible);

        layout.ToggleSessions();

        Assert.True(layout.SessionSidebarVisible);
    }

    [Fact]
    public void ExplorerToggleSwitchesFromSessionsBeforeHidingNavigation()
    {
        var layout = CreateLayout();

        Assert.True(layout.ToggleExplorer());
        Assert.True(layout.ExplorerSidebarVisible);

        Assert.False(layout.ToggleExplorer());
        Assert.False(layout.NavigationVisible);
        Assert.True(layout.ExplorerVisible);
    }

    [Fact]
    public void NavigationTogglesRestoreACollapsedWidth()
    {
        var layout = CreateLayout();
        layout.NavigationPaneWidth = WorkspaceLayoutViewModel.MinimumNavigationPaneWidth - 1;
        layout.ToggleSessions();
        Assert.Equal(WorkspaceLayoutViewModel.DefaultNavigationPaneWidth, layout.NavigationPaneWidth);

        layout.NavigationPaneWidth = WorkspaceLayoutViewModel.MinimumNavigationPaneWidth - 1;
        layout.ToggleExplorer();
        Assert.Equal(WorkspaceLayoutViewModel.DefaultNavigationPaneWidth, layout.NavigationPaneWidth);

        layout.NavigationPaneWidth = 250;
        layout.ToggleSessions();
        Assert.Equal(250, layout.NavigationPaneWidth);
    }

    [Fact]
    public void RightGutterTogglesRestoreACollapsedReviewWidth()
    {
        var layout = CreateLayout();
        layout.ReviewPaneWidth = WorkspaceLayoutViewModel.MinimumReviewPaneWidth - 1;
        layout.ToggleReview();
        Assert.Equal(WorkspaceLayoutViewModel.DefaultReviewPaneWidth, layout.ReviewPaneWidth);

        layout.ReviewPaneWidth = WorkspaceLayoutViewModel.MinimumReviewPaneWidth - 1;
        layout.ToggleChildSessions();
        Assert.Equal(WorkspaceLayoutViewModel.DefaultReviewPaneWidth, layout.ReviewPaneWidth);
        Assert.True(layout.ChildSessionsVisible);
    }

    [Fact]
    public void DraggingARegionBelowItsMinimumClosesItOnRelease()
    {
        var layout = CreateLayout();
        layout.UpdateLayoutSize(1440, 900);
        layout.ToggleBottomDrawer(WorkspaceBottomDrawer.Git);

        layout.ApplyResize(WorkspaceResizeTarget.Navigation, layout.NavigationPaneWidth, -400, 0);
        layout.CompleteResize(WorkspaceResizeTarget.Navigation);
        layout.ApplyResize(WorkspaceResizeTarget.BottomDrawer, layout.BottomDrawerHeight, 0, 300);
        layout.CompleteResize(WorkspaceResizeTarget.BottomDrawer);

        Assert.Equal(0, layout.NavigationPaneWidth);
        Assert.False(layout.NavigationVisible);
        Assert.False(layout.GitVisible);
    }

    [Fact]
    public void BottomDrawerResizeKeepsTheMinimumWorkspaceHeight()
    {
        var layout = CreateLayout();
        layout.UpdateLayoutSize(1440, 900);

        layout.ApplyResize(WorkspaceResizeTarget.BottomDrawer, layout.BottomDrawerHeight, 0, -1000);

        // 900 window height minus title bar, gap, minimum workspace, and footer.
        Assert.Equal(900 - 36 - 4 - 360 - 20, layout.BottomDrawerHeight);
    }

    [Fact]
    public void LayoutChangesAreSignalledForPersistence()
    {
        var layout = CreateLayout();
        var regions = 0;
        var geometry = 0;
        layout.RegionStateChanged += () => regions++;
        layout.PanelGeometryChanged += () => geometry++;

        layout.ToggleBottomDrawer(WorkspaceBottomDrawer.Git);
        layout.ReviewPaneWidth = 300;

        Assert.Equal(1, regions);
        Assert.Equal(1, geometry);
    }

    [Fact]
    public void GutterAttentionFollowsTheLayoutRightRegion()
    {
        using var viewModel = new DesktopViewModel();
        var changed = new List<string?>();
        viewModel.PropertyChanged += (_, args) => changed.Add(args.PropertyName);

        viewModel.Layout.ChildSessionsVisible = true;

        Assert.Contains(nameof(DesktopViewModel.ReviewGutterAttention), changed);
        Assert.Contains(nameof(DesktopViewModel.ChildSessionsGutterAttention), changed);
    }
}
