using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Input;
using Avalonia.Interactivity;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Views.ProjectWorkspace;

namespace SunCode.Desktop.Infrastructure;

// Window-owned presentation only. Every callback resolves the current context so
// a merged window's menus follow its selected project rather than its first tab.
internal sealed class DesktopMenuController
{
    private readonly Func<Window?> _owner;
    private readonly Func<DesktopViewModel?> _model;
    private readonly Func<ProjectWorkspace?> _workspace;
    private readonly Func<Task> _openProject;
    private readonly List<Action> _refresh = [];
    private readonly List<Action> _populate = [];
    private readonly List<MenuItem> _barGroups = [];
    private readonly Dictionary<NativeMenu, MenuItem> _barMenus = [];
    private readonly List<(KeyGesture Gesture, Func<bool> Execute)> _shortcuts = [];

    internal NativeMenu Menu { get; } = new();
    internal IReadOnlyList<MenuItem> BarGroups => _barGroups;
    private Window? Owner => _owner();
    private DesktopViewModel? Model => _model();
    private ProjectWorkspace? Workspace => _workspace();
    private bool Enabled => Owner?.IsEnabled == true && Workspace?.HasMenuBlockingOverlay != true;
    private bool ProjectReady => Enabled && Workspace is not null && Model?.IsProjectOpen == true;
    private bool SessionReady => ProjectReady && Model is { IsSessionLoading: false, HasSessionLoadError: false, SelectedSession: not null };
    private bool EditableSession => SessionReady && Model?.SelectedSession?.IsArchived == false;
    private bool IdleSession => EditableSession && Model is { IsBusy: false, IsTurnActive: false } && Model.SelectedSession?.IsTurnActive == false;
    private static KeyModifiers CommandModifier => OperatingSystem.IsMacOS() ? KeyModifiers.Meta : KeyModifiers.Control;
    private static string L(string name, string fallback) => LocalizationService.GetString("LocMenu" + name, fallback);

    internal DesktopMenuController(
        Func<Window?> owner,
        Func<DesktopViewModel?> model,
        Func<ProjectWorkspace?> workspace,
        Func<Task> openProject)
    {
        _owner = owner;
        _model = model;
        _workspace = workspace;
        _openProject = openProject;
        BuildFile();
        BuildEdit();
        BuildNavigate();
        BuildSession();
        BuildAgent();
        BuildView();
        BuildWindow();
        BuildHelp();
        Menu.NeedsUpdate += (_, _) => Refresh();
        Refresh();
        // Collapsed menu items are not a reliable hotkey host. Handle application
        // shortcuts at the window before focused text/terminal controls consume them.
        // AppKit continues to dispatch native menu gestures on macOS.
        if (!OperatingSystem.IsMacOS() && Owner is { } window)
            window.AddHandler(InputElement.KeyDownEvent, ShortcutKeyDown, RoutingStrategies.Tunnel);
    }

    private void ShortcutKeyDown(object? sender, KeyEventArgs args)
    {
        foreach (var shortcut in _shortcuts)
        {
            if (!shortcut.Gesture.Matches(args)) continue;
            if (shortcut.Execute()) args.Handled = true;
            return;
        }
    }

    internal void Refresh()
    {
        foreach (var update in _refresh) update();
        foreach (var populate in _populate) populate();
    }

    private NativeMenu Group(string name, string fallback)
    {
        var menu = new NativeMenu();
        var group = new NativeMenuItem { Header = L(name, fallback), Menu = menu };
        var barGroup = BarItem();
        barGroup.Header = L(name, fallback);
        barGroup.Classes.Add("windows-menu-group");
        _refresh.Add(() => group.Header = L(name, fallback));
        _refresh.Add(() => barGroup.Header = L(name, fallback));
        menu.NeedsUpdate += (_, _) => Refresh();
        Menu.Items.Add(group);
        _barGroups.Add(barGroup);
        _barMenus[menu] = barGroup;
        return menu;
    }

    private NativeMenuItem Action(NativeMenu menu, string name, string fallback, Action execute,
        Func<bool>? enabled = null, KeyGesture? gesture = null, Func<bool>? check = null, bool windowShortcut = true)
    {
        var item = new NativeMenuItem { Gesture = gesture };
        var barItem = BarItem();
        barItem.InputGesture = gesture;
        if (check is not null) item.ToggleType = MenuItemToggleType.CheckBox;
        if (check is not null) barItem.ToggleType = MenuItemToggleType.CheckBox;
        void Update()
        {
            item.Header = L(name, fallback);
            item.IsEnabled = Enabled && (enabled?.Invoke() ?? true);
            barItem.Header = L(name, fallback);
            barItem.IsEnabled = Enabled && (enabled?.Invoke() ?? true);
            if (check is not null)
            {
                item.IsChecked = check();
                barItem.IsChecked = check();
            }
        }
        bool Execute()
        {
            // Recheck at invocation: context can change while a native menu is open.
            if (!Enabled || enabled?.Invoke() == false) return false;
            execute();
            if (OperatingSystem.IsWindows()) Workspace?.CloseWindowsMenu();
            Refresh();
            return true;
        }
        if (gesture is not null && windowShortcut) _shortcuts.Add((gesture, Execute));
        item.Click += (_, _) => Execute();
        barItem.Click += (_, _) => Execute();
        _refresh.Add(Update);
        Update();
        menu.Items.Add(item);
        FindBarMenu(menu).Items.Add(barItem);
        return item;
    }

    private NativeMenu Dynamic(NativeMenu parent, string name, string fallback, Action<NativeMenu> populate,
        Func<bool>? enabled = null)
    {
        var menu = new NativeMenu();
        var item = new NativeMenuItem { Menu = menu };
        var barItem = BarItem();
        _refresh.Add(() =>
        {
            item.Header = L(name, fallback);
            item.IsEnabled = Enabled && (enabled?.Invoke() ?? true);
            barItem.Header = L(name, fallback);
            barItem.IsEnabled = Enabled && (enabled?.Invoke() ?? true);
        });
        _populate.Add(() =>
        {
            menu.Items.Clear();
            barItem.Items.Clear();
            populate(menu);
        });
        menu.NeedsUpdate += (_, _) => Refresh();
        parent.Items.Add(item);
        FindBarMenu(parent).Items.Add(barItem);
        _barMenus[menu] = barItem;
        populate(menu);
        return menu;
    }

    private MenuItem FindBarMenu(NativeMenu menu) => _barMenus.TryGetValue(menu, out var item)
        ? item
        : throw new InvalidOperationException("Menu is not registered.");

    private static MenuItem BarItem()
    {
        var item = new MenuItem();
        item.Classes.Add("windows-menu-item");
        return item;
    }

    private void DynamicAction(NativeMenu menu, string title, Action execute, Func<bool> enabled, bool? selected = null)
    {
        var item = new NativeMenuItem { Header = title, IsEnabled = Enabled && enabled() };
        var barItem = BarItem();
        barItem.Header = title;
        barItem.IsEnabled = Enabled && enabled();
        if (selected is { } check)
        {
            item.ToggleType = MenuItemToggleType.Radio;
            item.IsChecked = check;
            barItem.ToggleType = MenuItemToggleType.Radio;
            barItem.IsChecked = check;
        }
        void Execute()
        {
            if (!Enabled || !enabled()) return;
            execute();
            if (OperatingSystem.IsWindows()) Workspace?.CloseWindowsMenu();
            Refresh();
        }
        item.Click += (_, _) => Execute();
        barItem.Click += (_, _) => Execute();
        menu.Items.Add(item);
        FindBarMenu(menu).Items.Add(barItem);
    }

    private void Empty(NativeMenu menu, string name, string fallback)
    {
        var label = L(name, fallback);
        menu.Items.Add(new NativeMenuItem { Header = label, IsEnabled = false });
        var barItem = BarItem();
        barItem.Header = label;
        barItem.IsEnabled = false;
        FindBarMenu(menu).Items.Add(barItem);
    }

    private void Separator(NativeMenu menu)
    {
        menu.Items.Add(new NativeMenuItemSeparator());
        var separator = new Separator();
        separator.Classes.Add("windows-menu-separator");
        FindBarMenu(menu).Items.Add(separator);
    }

    private async Task RunAsync(Func<Task> operation)
    {
        try { await operation(); }
        catch (Exception exception)
        {
            DiagnosticLog.Error("menu.command", exception, "operation=execute");
            if (Model is { } model) model.ReportPresentationError(exception.Message);
        }
    }

    private void BuildFile()
    {
        var file = Group("File", "File");
        Action(file, "OpenProject", "Open Project…", () => _ = RunAsync(_openProject),
            () => Model?.ConnectionState == "connected", new KeyGesture(Key.O, CommandModifier));
        Dynamic(file, "OpenRecentProject", "Open Recent Project", PopulateProjects);
        Separator(file);
        Action(file, "BackToProjects", "Back to Projects", () =>
        {
            (Application.Current as App)?.ShowProjectHub();
            Owner?.Close();
        }, () => ProjectReady);
        if (!OperatingSystem.IsMacOS())
        {
            Separator(file);
            Action(file, "Settings", "Settings…", () => ShowSettings(), gesture: new KeyGesture(Key.OemComma, CommandModifier));
        }
        Separator(file);
        Action(file, "CloseWindow", "Close Window", () => Owner?.Close(), gesture: new KeyGesture(Key.W, CommandModifier));
    }

    private void BuildEdit()
    {
        var edit = Group("Edit", "Edit");
        EditAction(edit, DesktopEditAction.Undo, "Undo", "Undo", Key.Z);
        EditAction(edit, DesktopEditAction.Redo, "Redo", "Redo", Key.Z, KeyModifiers.Shift);
        Separator(edit);
        EditAction(edit, DesktopEditAction.Cut, "Cut", "Cut", Key.X);
        EditAction(edit, DesktopEditAction.Copy, "Copy", "Copy", Key.C);
        EditAction(edit, DesktopEditAction.Paste, "Paste", "Paste", Key.V);
        EditAction(edit, DesktopEditAction.SelectAll, "SelectAll", "Select All", Key.A);
    }

    private void EditAction(NativeMenu menu, DesktopEditAction action, string name, string fallback, Key key,
        KeyModifiers extra = KeyModifiers.None) =>
        Action(menu, name, fallback, () => DesktopEditCommands.Execute(Owner, action),
            () => DesktopEditCommands.CanExecute(Owner, action), new KeyGesture(key, CommandModifier | extra),
            windowShortcut: false);

    private void BuildNavigate()
    {
        var navigate = Group("Navigate", "Navigate");
        Dynamic(navigate, "Projects", "Projects", PopulateProjects);
        Dynamic(navigate, "Sessions", "Sessions", menu => PopulateSessions(menu, false), () => ProjectReady);
        Dynamic(navigate, "ArchivedSessions", "Archived Sessions", menu => PopulateSessions(menu, true), () => ProjectReady);
        Separator(navigate);
        Action(navigate, "FocusConversation", "Focus Conversation", () => _ = RunAsync(() => Workspace?.FocusConversationAsync() ?? Task.CompletedTask), () => EditableSession);
    }

    private void PopulateProjects(NativeMenu menu)
    {
        var projects = Model?.Projects.ToArray() ?? [];
        foreach (var project in projects)
            DynamicAction(menu, string.IsNullOrWhiteSpace(project.DisplayName) ? project.CanonicalRoot : project.DisplayName,
                () => _ = RunAsync(() => (Application.Current as App)?.OpenProjectWindowAsync(project) ?? Task.CompletedTask),
                () => Model?.ConnectionState == "connected" && Model.Projects.Any(item => item.ProjectId == project.ProjectId),
                Model?.SelectedProject?.ProjectId == project.ProjectId);
        if (projects.Length == 0) Empty(menu, "NoRecentProjects", "No Recent Projects");
    }

    private void PopulateSessions(NativeMenu menu, bool archived)
    {
        var model = Model;
        var sessions = (archived ? model?.ArchivedSessions : model?.Sessions)?.ToArray() ?? [];
        foreach (var session in sessions)
            DynamicAction(menu, session.DisplayTitle, () => _ = RunAsync(() => model!.SelectSessionAsync(session)),
                () => ProjectReady && ReferenceEquals(Model, model) &&
                    (archived ? model!.ArchivedSessions : model!.Sessions).Any(item => item.SessionId == session.SessionId),
                model?.SelectedSession?.SessionId == session.SessionId);
        if (sessions.Length == 0) Empty(menu, archived ? "NoArchivedSessions" : "NoSessions",
            archived ? "No Archived Sessions" : "No Sessions");
    }

    private void BuildSession()
    {
        var session = Group("Session", "Session");
        Action(session, "NewSession", "New Session…", () => Workspace?.ShowSessionDialog(
            LocalizationService.GetString("LocNewSession", "New session"), string.Empty,
            LocalizationService.GetString("LocCreate", "Create"), null),
            () => ProjectReady && Model?.IsBusy == false, new KeyGesture(Key.N, CommandModifier));
        Action(session, "RenameSession", "Rename Session…", () =>
        {
            if (Model?.SelectedSession is { } selected)
                Workspace?.ShowSessionDialog(LocalizationService.GetString("LocRename", "Rename"), selected.DisplayTitle,
                    LocalizationService.GetString("LocSave", "Save"), selected);
        }, () => EditableSession && Model?.IsBusy == false);
        Action(session, "ArchiveSession", "Archive Session…", () =>
        {
            if (Model?.SelectedSession is { } selected) Workspace?.ShowArchiveDialog(selected);
        }, () => IdleSession);
        Action(session, "RestoreSession", "Restore Archived Session…", () =>
        {
            if (Model?.SelectedSession is { } selected) Workspace?.ShowRestoreDialog(selected);
        }, () => SessionReady && Model is { IsBusy: false, SelectedSession.IsArchived: true });
        Action(session, "DeleteSession", "Delete Archived Session Permanently…", () =>
        {
            if (Model?.SelectedSession is { } selected) Workspace?.ShowPermanentDeleteDialog(selected);
        }, () => SessionReady && Model is { IsBusy: false, SelectedSession.IsArchived: true });
        Separator(session);
        Action(session, "RetryLastTurn", "Retry Last Turn", () => _ = RunAsync(() => Model!.RetryLastTurnAsync()),
            () => IdleSession && Model?.Messages.Count > 0 && Model?.HasPendingApproval == false && Model?.HasPendingQuestion == false);
        Dynamic(session, "UndoAgentChanges", "Undo Agent Changes…", PopulateCheckpoints, () => IdleSession);
    }

    private void PopulateCheckpoints(NativeMenu menu)
    {
        var model = Model;
        var checkpoints = model?.Checkpoints.Where(item => item.CanReview).ToArray() ?? [];
        foreach (var checkpoint in checkpoints)
            DynamicAction(menu, $"{checkpoint.TurnId} — {checkpoint.FileCount}", () => Workspace?.ShowUndoDialog(checkpoint),
                () => IdleSession && ReferenceEquals(Model, model) && model!.Checkpoints.Contains(checkpoint) && checkpoint.CanReview);
        if (checkpoints.Length == 0) Empty(menu, "NoRestorableChanges", "No Restorable Changes");
    }

    private void BuildAgent()
    {
        var agent = Group("Agent", "Agent");
        Action(agent, "StopTurn", "Stop Current Turn", () => _ = RunAsync(() => Model!.CancelTurnAsync()),
            () => EditableSession && Model?.IsTurnActive == true);
        Separator(agent);
        Dynamic(agent, "Model", "Model", PopulateModels, () => ProjectReady && Model?.CanChooseModel == true);
        Dynamic(agent, "ReasoningEffort", "Reasoning Effort", menu =>
        {
            foreach (var effort in Model?.ReasoningEffortOptions ?? [])
                DynamicAction(menu, effort, () => Model!.SelectedReasoningEffort = effort,
                    () => ProjectReady && Model?.CanChooseReasoningEffort == true && Model.ReasoningEffortOptions.Contains(effort),
                    Model?.SelectedReasoningEffort == effort);
            if (menu.Items.Count == 0) Empty(menu, "ReasoningUnavailable", "Unavailable for This Model");
        }, () => ProjectReady && Model?.CanChooseReasoningEffort == true);
        Separator(agent);
        Action(agent, "PendingApproval", "Show Pending Approval", () => _ = RunAsync(() => Workspace?.ShowPendingApprovalAsync() ?? Task.CompletedTask),
            () => SessionReady && Model?.HasPendingApproval == true);
        Action(agent, "PendingQuestion", "Show Pending Question", () => _ = RunAsync(() => Workspace?.ShowPendingQuestionAsync() ?? Task.CompletedTask),
            () => SessionReady && Model?.HasPendingQuestion == true);
    }

    private void PopulateModels(NativeMenu menu)
    {
        foreach (var provider in Model?.Models.GroupBy(item => item.Provider) ?? [])
        {
            var choices = new NativeMenu();
            var barChoices = BarItem();
            barChoices.Header = provider.First().ProviderLabel;
            FindBarMenu(menu).Items.Add(barChoices);
            _barMenus[choices] = barChoices;
            foreach (var model in provider)
                DynamicAction(choices, model.Display, () => Model!.SelectedModel = model,
                    () => ProjectReady && Model?.CanChooseModel == true && Model.Models.Contains(model), Model?.SelectedModel?.Id == model.Id);
            menu.Items.Add(new NativeMenuItem { Header = provider.First().ProviderLabel, Menu = choices });
        }
        if (menu.Items.Count == 0) Empty(menu, "NoModels", "No Models Available");
    }

    private void BuildView()
    {
        var view = Group("View", "View");
        Action(view, "Sessions", "Sessions", () => Workspace?.ToggleSessionsPanel(), () => ProjectReady,
            check: () => Model?.Layout.SessionSidebarVisible == true);
        Action(view, "Explorer", "Explorer", () => _ = RunAsync(() => Workspace?.ToggleExplorerPanelAsync() ?? Task.CompletedTask), () => ProjectReady,
            check: () => Model?.Layout.ExplorerSidebarVisible == true);
        Action(view, "ProjectNavigation", "Project Navigation", () =>
        {
            if (Model is { } model) model.Layout.NavigationVisible = !model.Layout.NavigationVisible;
        }, () => ProjectReady, new KeyGesture(Key.D1, CommandModifier), () => Model?.Layout.NavigationVisible == true);
        Separator(view);
        if (CefPreviewRuntime.Bundled)
            Action(view, "Browser", "Built-in Browser", () => Workspace?.ToggleBrowserPanel(), () => ProjectReady,
                check: () => Model?.Layout.PreviewVisible == true);
        Action(view, "Review", "Review", () => Workspace?.ToggleReviewPanel(), () => ProjectReady,
            check: () => Model?.Layout.ReviewVisible == true && Model?.Layout.PreviewVisible == false);
        Action(view, "ChildSessions", "Child Sessions", () => _ = RunAsync(() => Workspace?.ToggleChildSessionsPanelAsync() ?? Task.CompletedTask), () => SessionReady,
            check: () => Model?.Layout.ChildSessionsVisible == true && Model?.Layout.PreviewVisible == false);
        Separator(view);
        var git = Action(view, "SourceControl", "Source Control", () => Workspace?.ToggleGitViewer(),
            () => ProjectReady && Model?.Git.IsAvailable == true, new KeyGesture(Key.D9, CommandModifier), () => Model?.Layout.GitVisible == true);
        _refresh.Add(() => git.IsVisible = Workspace is null || Model?.Git.IsAvailable == true);
        Action(view, "ProviderTrace", "Provider Trace", () => _ = RunAsync(() => Workspace?.ToggleProviderTracePanelAsync() ?? Task.CompletedTask), () => SessionReady,
            check: () => Model?.Layout.ProviderTraceVisible == true);
        Action(view, "ToolActivity", "Tool Activity", () => Workspace?.ToggleToolActivityPanel(), () => SessionReady,
            check: () => Model?.Layout.ToolActivityVisible == true);
    }

    private void BuildWindow()
    {
        var window = Group("Window", "Window");
        Action(window, "Minimize", "Minimize", () => { if (Owner is { } owner) owner.WindowState = WindowState.Minimized; },
            gesture: new KeyGesture(Key.M, CommandModifier));
        Action(window, "Zoom", "Zoom", () =>
        {
            if (Owner is { } owner) owner.WindowState = owner.WindowState == WindowState.Maximized ? WindowState.Normal : WindowState.Maximized;
        }, () => Owner?.WindowState != WindowState.FullScreen);
        Action(window, "FullScreen", "Full Screen", () =>
        {
            if (Owner is WorkspaceWindow workspace) workspace.ToggleFullScreen();
            else if (Owner is { } owner) owner.WindowState = owner.WindowState == WindowState.FullScreen ? WindowState.Normal : WindowState.FullScreen;
        }, check: () => Owner?.WindowState == WindowState.FullScreen);
        Separator(window);
        Action(window, "MergeAllWindows", "Merge All Windows", () => _ = RunAsync(() => (Application.Current as App)?.MergeAllProjectWindowsAsync() ?? Task.CompletedTask),
            () => (Application.Current as App)?.CanMergeWindows == true);
        Dynamic(window, "OpenWindows", "Open Windows", menu =>
        {
            var windows = (Application.Current?.ApplicationLifetime as IClassicDesktopStyleApplicationLifetime)?.Windows
                .Where(item => item.IsVisible && item.IsEnabled).ToArray() ?? [];
            foreach (var target in windows)
                DynamicAction(menu, target.Title ?? AppInfo.ProductName, () =>
                {
                    if (target.WindowState == WindowState.Minimized) target.WindowState = WindowState.Normal;
                    target.Activate();
                }, () => target.IsVisible && target.IsEnabled, target == Owner);
            if (windows.Length == 0) Empty(menu, "NoWindows", "No Open Windows");
        });
    }

    private void BuildHelp()
    {
        var help = Group("Help", "Help");
        Action(help, "KeyboardShortcuts", "Keyboard Shortcuts", () => ShowSettings(SettingsPage.Shortcuts));
        // Keep Help local until there is a published, product-owned documentation URL.
        if (!OperatingSystem.IsMacOS())
            Action(help, "About", "About SunCode", () => { if (Owner is { } owner) (Application.Current as App)?.ShowAbout(owner); });
    }

    private void ShowSettings(SettingsPage? page = null)
    {
        if (Owner is { } owner) (Application.Current as App)?.ShowSettings(owner, page: page);
    }
}
