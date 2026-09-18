using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Tests;

public sealed class RecentContentTests
{
    [Fact]
    public void CurrentTitleUsesSessionOrActiveFileName()
    {
        var session = Session("session-1", "Workspace information architecture");
        var file = new ExplorerNode("ProjectWorkspace.axaml", "Views/ProjectWorkspace.axaml", "file");

        Assert.Equal(
            "Workspace information architecture",
            DesktopViewModel.CurrentContentTitleFor(session, null));
        Assert.Equal(
            "ProjectWorkspace.axaml",
            DesktopViewModel.CurrentContentTitleFor(session, file));
        Assert.Equal("No content selected", DesktopViewModel.CurrentContentTitleFor(null, null));
    }

    [Fact]
    public void MixedHistoryKeepsCurrentInternallyButOffersOnlyOtherItems()
    {
        using var viewModel = new DesktopViewModel();
        var session = Session("session-1", "Workspace information architecture");
        var file = new ExplorerNode("ProjectWorkspace.axaml", "Views/ProjectWorkspace.axaml", "file");

        viewModel.RememberRecentSession(session);
        viewModel.RememberRecentFile(file);

        Assert.Collection(
            viewModel.RecentContents,
            item =>
            {
                Assert.True(item.IsFile);
                Assert.True(item.IsCurrent);
                Assert.Equal(file, item.File);
            },
            item =>
            {
                Assert.True(item.IsSession);
                Assert.False(item.IsCurrent);
                Assert.Equal(session, item.Session);
            });
        Assert.Collection(
            viewModel.RecentContentOptions,
            item => Assert.Equal("session:session-1", item.ContentId));
        Assert.DoesNotContain(viewModel.RecentContentOptions, item => item.IsCurrent);
        Assert.Equal("1 / 20", viewModel.RecentContentCountText);

        viewModel.RememberRecentSession(session);

        Assert.Equal(2, viewModel.RecentContents.Count);
        Assert.Equal("session:session-1", viewModel.RecentContents[0].ContentId);
        Assert.True(viewModel.RecentContents[0].IsCurrent);
        Assert.False(viewModel.RecentContents[1].IsCurrent);
        Assert.Collection(
            viewModel.RecentContentOptions,
            item => Assert.Equal(RecentContentItem.FromFile(file).ContentId, item.ContentId));
    }

    [Fact]
    public void ACurrentItemByItselfProducesAnEmptyMenu()
    {
        using var viewModel = new DesktopViewModel();

        viewModel.RememberRecentSession(Session("session-1", "Only session"));

        Assert.Single(viewModel.RecentContents);
        Assert.Empty(viewModel.RecentContentOptions);
        Assert.False(viewModel.HasRecentContentOptions);
        Assert.Equal("0 / 20", viewModel.RecentContentCountText);
    }

    [Fact]
    public void HistoryRetainsOnlyTheTwentyMostRecentUniqueItems()
    {
        using var viewModel = new DesktopViewModel();

        for (var index = 0; index <= DesktopViewModel.RecentContentLimit; index++)
            viewModel.RememberRecentSession(Session($"session-{index}", $"Session {index}"));

        Assert.Equal(DesktopViewModel.RecentContentLimit, viewModel.RecentContents.Count);
        Assert.Equal("session:session-20", viewModel.RecentContents[0].ContentId);
        Assert.DoesNotContain(
            viewModel.RecentContents,
            item => item.ContentId == "session:session-0");
        Assert.Equal(DesktopViewModel.RecentContentLimit - 1, viewModel.RecentContentOptions.Count);
        Assert.DoesNotContain(viewModel.RecentContentOptions, item => item.IsCurrent);
        Assert.Equal("19 / 20", viewModel.RecentContentCountText);
    }

    [Fact]
    public void SessionReloadUpdatesRenamedItemsAndRemovesArchivedItems()
    {
        using var viewModel = new DesktopViewModel();
        viewModel.RememberRecentSession(Session("session-1", "Original title"));
        var renamed = Session("session-1", "Renamed title");
        viewModel.Sessions.Add(renamed);

        viewModel.RefreshRecentSessionReferences();

        Assert.Single(viewModel.RecentContents);
        Assert.Equal("Renamed title", viewModel.RecentContents[0].Title);
        Assert.Same(renamed, viewModel.RecentContents[0].Session);
        Assert.Empty(viewModel.RecentContentOptions);

        viewModel.Sessions.Clear();
        viewModel.RefreshRecentSessionReferences();

        Assert.Empty(viewModel.RecentContents);
        Assert.Empty(viewModel.RecentContentOptions);
        Assert.False(viewModel.HasRecentContentOptions);
    }

    [Fact]
    public void FileIdentityIncludesDependencyScopeAndUsesBoundedDisplayPath()
    {
        var projectFile = new ExplorerNode("README.md", "README.md", "file");
        var dependencyFile = new ExplorerNode("README.md", "README.md", "file", "shared-ui");

        var projectItem = RecentContentItem.FromFile(projectFile);
        var dependencyItem = RecentContentItem.FromFile(dependencyFile);

        Assert.NotEqual(projectItem.ContentId, dependencyItem.ContentId);
        Assert.Equal("README.md", projectItem.Detail);
        Assert.Equal("dependency:shared-ui/README.md", dependencyItem.Detail);
        Assert.Equal("FILE", dependencyItem.KindLabel);
    }

    [Fact]
    public void ChildSessionHistoryUsesAgentIdentityAndRefreshesState()
    {
        using var viewModel = new DesktopViewModel();
        var child = Child("child-1", "Implement settings", "running");
        viewModel.RememberRecentChildSession(child);

        var item = Assert.Single(viewModel.RecentContents);
        Assert.True(item.IsChildSession);
        Assert.Equal("CHILD SESSION", item.KindLabel);
        Assert.Equal("Software Engineering Agent · Running", item.Detail);

        var completed = Child("child-1", "Implement settings", "completed");
        viewModel.ChildSessions.Add(completed);
        viewModel.RefreshRecentChildSessionReferences();

        Assert.Equal("Software Engineering Agent · Completed", viewModel.RecentContents[0].Detail);
        Assert.Same(completed, viewModel.RecentContents[0].ChildSession);
    }

    private static SessionItem Session(string id, string title) =>
        new(id, title, "2026-09-08T00:00:00Z", false);

    private static ChildSessionItem Child(string id, string title, string state) =>
        new(id, "parent-1", title, "builtin.swe.v1", "Software Engineering Agent", state,
            "2026-09-18T00:00:00Z", "gpt-5.5", "Implement", string.Empty, string.Empty);
}
