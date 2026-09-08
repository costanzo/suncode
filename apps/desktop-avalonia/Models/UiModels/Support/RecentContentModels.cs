using SunCode.Desktop.Infrastructure;

namespace SunCode.Desktop.Models;

public sealed class RecentContentItem : ObservableObject
{
    private SessionItem? _session;
    private ExplorerNode? _file;
    private string _title;
    private string _detail;
    private bool _isCurrent;

    private RecentContentItem(
        string contentId,
        string kind,
        string title,
        string detail,
        SessionItem? session,
        ExplorerNode? file)
    {
        ContentId = contentId;
        Kind = kind;
        _title = title;
        _detail = detail;
        _session = session;
        _file = file;
    }

    public string ContentId { get; }
    public string Kind { get; }
    public string Title { get => _title; private set => SetProperty(ref _title, value); }
    public string Detail { get => _detail; private set => SetProperty(ref _detail, value); }
    public SessionItem? Session { get => _session; private set => SetProperty(ref _session, value); }
    public ExplorerNode? File { get => _file; private set => SetProperty(ref _file, value); }
    public bool IsCurrent { get => _isCurrent; internal set => SetProperty(ref _isCurrent, value); }
    public bool IsSession => Kind == "session";
    public bool IsFile => Kind == "file";
    public string KindLabel => IsFile ? "FILE" : "SESSION";
    public string IconPath => IsFile ? File?.IconPath ?? "/Assets/icons/file-text.svg" : "/Assets/icons/message.svg";

    public static RecentContentItem FromSession(SessionItem session) => new(
        $"session:{session.SessionId}",
        "session",
        session.DisplayTitle,
        session.RelativeActivity,
        session,
        null);

    public static RecentContentItem FromFile(ExplorerNode file) => new(
        $"file:{file.DependencyId ?? "project"}:{file.Path}",
        "file",
        file.Name,
        DisplayPath(file),
        null,
        file);

    internal void UpdateFrom(RecentContentItem item)
    {
        Title = item.Title;
        Detail = item.Detail;
        Session = item.Session;
        File = item.File;
        OnPropertyChanged(nameof(IconPath));
    }

    internal static string DisplayPath(ExplorerNode file) => file.DependencyId is null
        ? file.Path
        : $"dependency:{file.DependencyId}/{file.Path}";
}
