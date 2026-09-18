using SunCode.Desktop.Infrastructure;

namespace SunCode.Desktop.Models;

public sealed class RecentContentItem : ObservableObject
{
    private SessionItem? _session;
    private ExplorerNode? _file;
    private ChildSessionItem? _childSession;
    private string _title;
    private string _detail;
    private bool _isCurrent;

    private RecentContentItem(
        string contentId,
        string kind,
        string title,
        string detail,
        SessionItem? session,
        ExplorerNode? file,
        ChildSessionItem? childSession)
    {
        ContentId = contentId;
        Kind = kind;
        _title = title;
        _detail = detail;
        _session = session;
        _file = file;
        _childSession = childSession;
    }

    public string ContentId { get; }
    public string Kind { get; }
    public string Title { get => _title; private set => SetProperty(ref _title, value); }
    public string Detail { get => _detail; private set => SetProperty(ref _detail, value); }
    public SessionItem? Session { get => _session; private set => SetProperty(ref _session, value); }
    public ExplorerNode? File { get => _file; private set => SetProperty(ref _file, value); }
    public ChildSessionItem? ChildSession { get => _childSession; private set => SetProperty(ref _childSession, value); }
    public bool IsCurrent { get => _isCurrent; internal set => SetProperty(ref _isCurrent, value); }
    public bool IsSession => Kind == "session";
    public bool IsFile => Kind == "file";
    public bool IsChildSession => Kind == "child-session";
    public string KindLabel => IsFile ? "FILE" : IsChildSession ? "CHILD SESSION" : "SESSION";
    public string IconPath => IsFile ? File?.IconPath ?? "/Assets/icons/file-text.svg" : IsChildSession ? "/Assets/icons/agent.svg" : "/Assets/icons/message.svg";

    public static RecentContentItem FromSession(SessionItem session) => new(
        $"session:{session.SessionId}",
        "session",
        session.DisplayTitle,
        session.RelativeActivity,
        session,
        null,
        null);

    public static RecentContentItem FromChildSession(ChildSessionItem child) => new(
        $"child-session:{child.SessionId}",
        "child-session",
        child.Title,
        $"{child.AgentDisplayName} · {child.StateText}",
        null,
        null,
        child);

    public static RecentContentItem FromFile(ExplorerNode file) => new(
        $"file:{file.DependencyId ?? "project"}:{file.Path}",
        "file",
        file.Name,
        DisplayPath(file),
        null,
        file,
        null);

    internal void UpdateFrom(RecentContentItem item)
    {
        Title = item.Title;
        Detail = item.Detail;
        Session = item.Session;
        File = item.File;
        ChildSession = item.ChildSession;
        OnPropertyChanged(nameof(IconPath));
    }

    internal static string DisplayPath(ExplorerNode file) => file.DependencyId is null
        ? file.Path
        : $"dependency:{file.DependencyId}/{file.Path}";
}
