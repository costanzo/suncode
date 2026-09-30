using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;

namespace SunCode.Desktop.Tests;

public sealed class EditorViewModelTests
{
    [Fact]
    public void IdleEditorHasNoFileAndLoadsAnyFile()
    {
        var editor = new EditorViewModel(new FakeViewModelHost());
        var node = new ExplorerNode("App.axaml", "App.axaml", "file");

        Assert.Null(editor.File);
        Assert.Equal("idle", editor.State);
        Assert.False(editor.IsLoading || editor.IsReady || editor.IsEmpty || editor.IsError);
        Assert.Equal(string.Empty, editor.FileName);
        Assert.Equal(string.Empty, editor.Path);
        Assert.Equal("/Assets/icons/file-text.svg", editor.IconPath);
        Assert.True(editor.ShouldLoad(node));
    }

    [Fact]
    public void CloseResetsStateAndNotifiesOnlyOnChange()
    {
        var editor = new EditorViewModel(new FakeViewModelHost());
        var changed = new List<string?>();
        editor.PropertyChanged += (_, args) => changed.Add(args.PropertyName);

        editor.Close();

        Assert.Null(editor.File);
        Assert.Equal("idle", editor.State);
        Assert.Empty(changed);
    }

    [Fact]
    public void WindowShowsConversationUntilAFileIsSelected()
    {
        using var window = new DesktopViewModel();

        Assert.Null(window.SelectedEditorFile);
        Assert.True(window.IsConversationVisible);
        Assert.False(window.IsEditorVisible);
        Assert.Same(window.Editor, window.Editor);
    }
}
