using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Sdk;

namespace SunCode.Desktop.ViewModels;

// Read-only document state for the file shown in the Workspace center:
// bounded content loading, load/empty/error states, and file identity text.
// The window decides which file is selected and whether the editor or the
// conversation is visible.
public sealed class EditorViewModel : ObservableObject
{
    private readonly IViewModelHost _host;
    private long _loadVersion;
    private ExplorerNode? _file;
    private string _content = string.Empty;
    private string _state = "idle";
    private string _error = string.Empty;

    internal EditorViewModel(IViewModelHost host) => _host = host;

    public ExplorerNode? File
    {
        get => _file;
        private set
        {
            if (!SetProperty(ref _file, value)) return;
            OnPropertyChanged(nameof(FileName));
            OnPropertyChanged(nameof(Path));
            OnPropertyChanged(nameof(LanguageLabel));
            OnPropertyChanged(nameof(IconPath));
        }
    }

    public string Content { get => _content; private set => SetProperty(ref _content, value); }

    public string State
    {
        get => _state;
        private set
        {
            if (!SetProperty(ref _state, value)) return;
            OnPropertyChanged(nameof(IsLoading));
            OnPropertyChanged(nameof(IsReady));
            OnPropertyChanged(nameof(IsEmpty));
            OnPropertyChanged(nameof(IsError));
        }
    }

    public string Error { get => _error; private set => SetProperty(ref _error, value); }

    public bool IsLoading => State == "loading";
    public bool IsReady => State == "ready";
    public bool IsEmpty => State == "empty";
    public bool IsError => State == "error";
    public string FileName => File?.Name ?? string.Empty;
    public string Path => File is not { } file
        ? string.Empty
        : file.DependencyId is null
            ? file.Path
            : $"dependency:{file.DependencyId}/{file.Path}";
    public string LanguageLabel => EditorLanguage.FromFileName(FileName).DisplayName;
    public string IconPath => File?.IconPath ?? "/Assets/icons/file-text.svg";

    // A selected file reloads only after it failed or was never loaded.
    internal bool ShouldLoad(ExplorerNode node) =>
        !ReferenceEquals(File, node) || State is "idle" or "error";

    // Shows the file immediately, then reads it. Returns false when the read
    // failed for the load that is still current.
    internal async Task<bool> LoadAsync(ExplorerNode node, string projectId)
    {
        var loadVersion = Interlocked.Increment(ref _loadVersion);
        File = node;
        Content = string.Empty;
        Error = string.Empty;
        State = "loading";
        try
        {
            var result = await _host.Sdk!.ReadProjectFileAsync(projectId, node.DependencyId, node.Path);
            if (!IsCurrentLoad(node, projectId, loadVersion)) return true;
            Content = result.Content;
            State = result.Content.Length == 0 ? "empty" : "ready";
            return true;
        }
        catch (Exception exception)
        {
            if (!IsCurrentLoad(node, projectId, loadVersion)) return true;
            Content = string.Empty;
            Error = exception is SdkException sdkException
                ? sdkException.Message
                : "The file could not be read.";
            State = "error";
            DiagnosticLog.Error(
                "editor.read",
                $"failed project={projectId} dependency={node.DependencyId ?? "none"} path={node.Path} type={exception.GetType().Name}");
            return false;
        }
    }

    internal void Close()
    {
        Interlocked.Increment(ref _loadVersion);
        File = null;
        Content = string.Empty;
        Error = string.Empty;
        State = "idle";
    }

    private bool IsCurrentLoad(ExplorerNode node, string projectId, long loadVersion) =>
        _loadVersion == loadVersion
        && _host.SelectedProject?.ProjectId == projectId
        && ReferenceEquals(File, node);
}
