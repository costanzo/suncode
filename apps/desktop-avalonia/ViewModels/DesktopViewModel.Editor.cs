using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Sdk;

namespace SunCode.Desktop.ViewModels;

public sealed partial class DesktopViewModel
{
    private long _editorLoadVersion;
    private ExplorerNode? _selectedEditorFile;
    private string _editorContent = string.Empty;
    private string _editorState = "idle";
    private string _editorError = string.Empty;

    public ExplorerNode? SelectedEditorFile
    {
        get => _selectedEditorFile;
        private set
        {
            if (!SetProperty(ref _selectedEditorFile, value)) return;
            OnPropertyChanged(nameof(IsEditorVisible));
            OnPropertyChanged(nameof(IsConversationVisible));
            OnPropertyChanged(nameof(EditorFileName));
            OnPropertyChanged(nameof(EditorPath));
            OnPropertyChanged(nameof(EditorLanguageLabel));
            OnPropertyChanged(nameof(EditorIconPath));
            NotifyCurrentContentChanged();
            SaveProjectUiState(saved => SaveCurrentContentState(saved));
        }
    }

    public string EditorContent
    {
        get => _editorContent;
        private set => SetProperty(ref _editorContent, value);
    }

    public string EditorState
    {
        get => _editorState;
        private set
        {
            if (!SetProperty(ref _editorState, value)) return;
            OnPropertyChanged(nameof(IsEditorLoading));
            OnPropertyChanged(nameof(IsEditorReady));
            OnPropertyChanged(nameof(IsEditorEmpty));
            OnPropertyChanged(nameof(IsEditorError));
        }
    }

    public string EditorError
    {
        get => _editorError;
        private set => SetProperty(ref _editorError, value);
    }

    public bool IsEditorVisible => SelectedEditorFile is not null;
    public bool IsConversationVisible => !IsEditorVisible;
    public bool IsEditorLoading => EditorState == "loading";
    public bool IsEditorReady => EditorState == "ready";
    public bool IsEditorEmpty => EditorState == "empty";
    public bool IsEditorError => EditorState == "error";
    public string EditorFileName => SelectedEditorFile?.Name ?? string.Empty;
    public string EditorPath => SelectedEditorFile is not { } file
        ? string.Empty
        : file.DependencyId is null
            ? file.Path
            : $"dependency:{file.DependencyId}/{file.Path}";
    public string EditorLanguageLabel => EditorLanguage.FromFileName(EditorFileName).DisplayName;
    public string EditorIconPath => SelectedEditorFile?.IconPath ?? "/Assets/icons/file-text.svg";

    public async Task SelectExplorerFileAsync(ExplorerNode node)
    {
        if (!node.IsFile || _sdk is null || SelectedProject is null) return;

        var projectId = SelectedProject.ProjectId;
        var loadVersion = Interlocked.Increment(ref _editorLoadVersion);
        SelectedEditorFile = node;
        RememberRecentFile(node);
        EditorContent = string.Empty;
        EditorError = string.Empty;
        EditorState = "loading";
        try
        {
            var result = await _sdk.ReadProjectFileTypedAsync(projectId, node.DependencyId, node.Path);
            if (!IsCurrentEditorLoad(node, projectId, loadVersion)) return;
            EditorContent = result.Content;
            EditorState = result.Content.Length == 0 ? "empty" : "ready";
        }
        catch (Exception exception)
        {
            if (!IsCurrentEditorLoad(node, projectId, loadVersion)) return;
            EditorContent = string.Empty;
            EditorError = exception is SdkException sdkException
                ? sdkException.Message
                : "The file could not be read.";
            EditorState = "error";
            DiagnosticLog.Error(
                "editor.read",
                $"failed project={projectId} dependency={node.DependencyId ?? "none"} path={node.Path} type={exception.GetType().Name}");
            if (RestoringSavedFile)
            {
                RestoringSavedFile = false;
                var fallback = Sessions.FirstOrDefault();
                if (fallback is not null) await SelectSessionAsync(fallback);
            }
        }
    }

    private bool IsCurrentEditorLoad(ExplorerNode node, string projectId, long loadVersion) =>
        !_disposed
        && _editorLoadVersion == loadVersion
        && SelectedProject?.ProjectId == projectId
        && ReferenceEquals(SelectedEditorFile, node);

    private void CloseEditor()
    {
        Interlocked.Increment(ref _editorLoadVersion);
        SelectedEditorFile = null;
        EditorContent = string.Empty;
        EditorError = string.Empty;
        EditorState = "idle";
    }
}

internal sealed record EditorLanguage(string DisplayName, string GrammarExtension)
{
    public static EditorLanguage FromFileName(string fileName)
    {
        var extension = Path.GetExtension(fileName).ToLowerInvariant();
        return extension switch
        {
            ".axaml" => new("AXAML", ".xml"),
            ".xaml" => new("XAML", ".xml"),
            ".csproj" or ".props" or ".targets" => new("MSBuild", ".xml"),
            ".cs" => new("C#", ".cs"),
            ".rs" => new("Rust", ".rs"),
            ".js" => new("JavaScript", ".js"),
            ".jsx" => new("JavaScript JSX", ".jsx"),
            ".ts" => new("TypeScript", ".ts"),
            ".tsx" => new("TypeScript JSX", ".tsx"),
            ".json" => new("JSON", ".json"),
            ".md" or ".markdown" => new("Markdown", ".md"),
            ".py" => new("Python", ".py"),
            ".go" => new("Go", ".go"),
            ".java" => new("Java", ".java"),
            ".html" or ".htm" => new("HTML", ".html"),
            ".css" => new("CSS", ".css"),
            ".scss" => new("SCSS", ".scss"),
            ".sh" => new("Shell", ".sh"),
            ".toml" => new("TOML", ".toml"),
            ".yaml" or ".yml" => new("YAML", ".yaml"),
            ".xml" => new("XML", ".xml"),
            _ => new("Plain text", string.Empty)
        };
    }
}
