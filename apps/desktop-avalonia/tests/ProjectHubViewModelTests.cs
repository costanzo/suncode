using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Tests;

public sealed class ProjectHubViewModelTests
{
    [Fact]
    public void HubExposesProjectListFromBackingViewModel()
    {
        using var hub = new ProjectHubViewModel(new DesktopViewModel());

        Assert.Same(hub.Services.Projects, hub.Projects);
        Assert.False(hub.HasProjects);
        Assert.False(hub.CanOpenProjects);

        hub.Services.Projects.Add(new ProjectItem("project", "Project", "/tmp/project"));
        Assert.True(hub.HasProjects);
    }

    [Fact]
    public void HubForwardsOnlyItsOwnPropertyChanges()
    {
        using var hub = new ProjectHubViewModel(new DesktopViewModel());
        var changed = new List<string?>();
        hub.PropertyChanged += (_, args) => changed.Add(args.PropertyName);

        hub.Services.ReportPresentationError("Agent unavailable");
        hub.Services.OnPropertyChanged(nameof(DesktopViewModel.HasProjects));
        hub.Services.OnPropertyChanged(nameof(DesktopViewModel.SessionTitle));

        Assert.Equal([nameof(ProjectHubViewModel.StatusText), nameof(ProjectHubViewModel.HasProjects)], changed);
        Assert.Equal("Agent unavailable", hub.StatusText);
    }

    [Fact]
    public void DisposingHubDetachesFromBackingViewModel()
    {
        var hub = new ProjectHubViewModel(new DesktopViewModel());
        var changed = 0;
        hub.PropertyChanged += (_, _) => changed++;

        hub.Dispose();
        hub.Services.OnPropertyChanged(nameof(DesktopViewModel.HasProjects));

        Assert.Equal(0, changed);
    }
}
