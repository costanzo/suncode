using SunCode.Desktop.Models;

namespace SunCode.Desktop.ViewModels;

public sealed partial class DesktopViewModel
{
    public EditorViewModel Editor { get; }
    public ExplorerViewModel Explorer { get; }

    public ExplorerNode? SelectedEditorFile => Editor.File;
    public bool IsEditorVisible => SelectedEditorFile is not null && SelectedChildSession is null;
    public bool IsConversationVisible => SelectedEditorFile is null && SelectedChildSession is null;

    public async Task SelectExplorerFileAsync(ExplorerNode node)
    {
        if (!node.IsFile || _sdk is null || SelectedProject is null) return;
        if (!Editor.ShouldLoad(node)) return;

        var projectId = SelectedProject.ProjectId;
        var fileChanged = !ReferenceEquals(SelectedEditorFile, node);
        var load = Editor.LoadAsync(node, projectId);
        if (fileChanged) OnEditorFileChanged();
        ClearSelectedChildSession();
        RememberRecentFile(node);
        if (await load || _disposed) return;
        if (RestoringSavedFile)
        {
            RestoringSavedFile = false;
            var fallback = Sessions.FirstOrDefault();
            if (fallback is not null) await SelectSessionAsync(fallback);
        }
    }

    private void CloseEditor()
    {
        var hadFile = SelectedEditorFile is not null;
        Editor.Close();
        if (hadFile) OnEditorFileChanged();
    }

    // Window state that depends on which file the editor shows.
    private void OnEditorFileChanged()
    {
        OnPropertyChanged(nameof(SelectedEditorFile));
        OnPropertyChanged(nameof(IsEditorVisible));
        OnPropertyChanged(nameof(IsConversationVisible));
        NotifyCurrentContentChanged();
        SaveProjectUiState(saved => SaveCurrentContentState(saved));
    }
}
