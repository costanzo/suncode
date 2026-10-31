using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Tests;

public sealed class TerminalViewModelTests
{
    [Fact]
    public void OpeningSessionsKeepsOnlyOneSelected()
    {
        var host = new FakeViewModelHost
        {
            SelectedProject = new ProjectItem("project", "Project", "/tmp/project")
        };
        using var terminal = new TerminalViewModel(host);

        var first = terminal.OpenSession();
        var second = terminal.OpenSession();

        Assert.True(second.IsSelected);
        Assert.False(first.IsSelected);
        Assert.Equal("/tmp/project", second.WorkingDirectory);
        Assert.Equal(2, terminal.Sessions.Count);
    }

    [Fact]
    public void ClosingSelectedSessionSelectsTheRemainingSession()
    {
        var host = new FakeViewModelHost
        {
            SelectedProject = new ProjectItem("project", "Project", "/tmp/project")
        };
        using var terminal = new TerminalViewModel(host);
        var first = terminal.OpenSession();
        var second = terminal.OpenSession();

        terminal.CloseSession(second);

        Assert.Single(terminal.Sessions);
        Assert.Same(first, terminal.SelectedSession);
        Assert.True(first.IsSelected);
    }

    [Fact]
    public void CannotOpenWithoutAProject()
    {
        using var terminal = new TerminalViewModel(new FakeViewModelHost());

        Assert.False(terminal.CanOpen);
        Assert.Throws<InvalidOperationException>(() => terminal.OpenSession());
    }
}
