# Project-Per-Window UI Amendment

- Date: 2026-08-07
- Status: **Accepted and normative.** Retained as a separate document because it is long and self-contained; `requirement.md` and `architecture.md` point here rather than duplicating it.
- Applies to: `requirement.md`, `architecture.md`, `plan.md`, `todo.md`, and `test-plan.md` in this directory
- Supersedes: all clauses that place multiple projects or project groups in the left sidebar, allow a project window to switch its active project, or describe Sessions and Projects as peer top-level destinations

## Decision

Suncode desktop uses a **project-per-window** model similar to IntelliJ IDEA and WebStorm:

- Each top-level project window is permanently bound to exactly one project for that window's lifetime.
- The left sidebar lists only sessions belonging to that project.
- The left sidebar never lists projects, project groups, or sessions from another project.
- Opening another project creates a new top-level window or focuses the window already bound to that project.
- Projects are opened through the native application menu, a project launcher, operating-system file integration, or a recent-project action outside the session sidebar.

**Project** is the term in every layer, per `ADR-20260807-domain-vocabulary`. An earlier version of this amendment kept "workspace" as the runtime's internal name for the same concept; that split was retired, because renaming the most central noun at a layer boundary produces lasting ambiguity in documents, protocol messages, and logs. Each project window binds to one runtime project identifier and must not change that binding in place.

## Rationale

Mixing projects and sessions in one sidebar creates two navigation hierarchies in the same narrow surface. It also makes project scope less obvious when users submit prompts, inspect changes, or approve machine operations. A project-per-window model gives each window an unambiguous filesystem, permission, settings, draft, and session boundary while still allowing the shared runtime to serve multiple open windows.

## Window model

### Project launcher

When Suncode starts without a project to restore or open, it shows a lightweight launcher containing:

- **Open Project...** using the native directory picker.
- A bounded, searchable recent-project list.
- Remove-from-recents and clear-unavailable actions that do not delete project data.
- Global Settings.
- Runtime status and Diagnostics.
- Exit/Quit.

The launcher is not a project dashboard. It does not display sessions, aggregated activity, approvals, changes, or conversations from recent projects.

The launcher closes when the first project opens unless the platform convention or user preference keeps it open. Closing the last project window may return to the launcher or quit according to a configurable platform-appropriate preference.

### Project window

Each project window has an immutable binding containing:

- Stable project/project identifier.
- Display name and disambiguated path or remote identity.
- Runtime target and authorization scope.
- Project-scoped settings identity.
- Project-scoped event-subscription scope.

The binding is established before session content becomes interactive. A window cannot retarget itself to another project. Choosing another project invokes the open/focus flow and leaves the current project window unchanged.

### Multiple windows

- Multiple project windows may exist concurrently in one Qt application process.
- Each project window maintains independent selected session, navigation width, inspector state, drafts, search query, and project-scoped dialogs.
- Global provider/runtime settings changes may propagate to every window through runtime events.
- Project-scoped settings changes affect only windows bound to that project.
- Closing one project window must not close, retarget, clear, or interrupt another project window.
- The shared local runtime may continue running sessions after a window closes according to the separately approved runtime lifecycle policy.

## Project opening behavior

### Application menu

The native File menu must include:

- **Open Project...**
- **Open Recent** submenu
- **Close Project** or **Close Window**, following platform wording
- **New Window** only if it has a clear no-project launcher behavior; it must not create an unbound project work surface
- Exit/Quit where platform conventions place it

Suggested shortcuts must follow platform conventions. `Ctrl+O`/`Cmd+O` may be used for Open Project only if it does not conflict with a future file-opening action; the final shortcut is decided during platform review.

### Open Project

1. The user invokes Open Project.
2. Suncode opens a native directory picker or approved remote-project selector.
3. The client submits the selected project identity to the runtime for canonicalization, availability, and authorization.
4. If the project is already open, Suncode focuses and raises its existing window instead of creating a duplicate.
5. Otherwise, Suncode creates a new project window in a non-interactive loading state.
6. The window becomes interactive after project identity, project settings, session summaries, runtime capabilities, and event scope are established.
7. Failure leaves the current window untouched and shows a recoverable error in the launcher or originating window.

Opening a project must never replace the project binding of the originating project window.

### Open Recent

- Recent projects appear only in the launcher, File menu, operating-system recent-items integration, or a dedicated quick-open dialog.
- Each entry shows project name and enough path/remote information to disambiguate it.
- Missing, moved, unauthorized, offline, and remote-unavailable entries have explicit states.
- Selecting an already open entry focuses its existing window.
- Removing an entry from recents must not delete the project or its sessions.

### Operating-system integration

- Opening a project through shell association, dock/taskbar recent item, command-line launch, or IDE/plugin handoff follows the same deduplicating open/focus flow.
- A second application launch should hand the project-open request to the existing per-user application instance when the approved single-instance architecture supports it.
- Window activation must respect platform focus-stealing rules.

## Revised primary layout

At a comfortable desktop width, a project window contains three regions:

```text
+----------------------+------------------------------------+--------------------------+
| Sessions             | Active session                     | Context inspector        |
|                      |                                    |                          |
| New session          | Header: project/session/status     | Changes / Files /        |
| Search sessions      |                                    | Activity                  |
|                      | Conversation timeline              |                          |
| This project's       |                                    | Selected diff, file,      |
| recent sessions      |                                    | command, or metadata      |
|                      |                                    |                          |
| Project settings     | Composer and turn controls         |                          |
+----------------------+------------------------------------+--------------------------+
```

- The sidebar is a session navigator, not a project navigator.
- Project name is visible in the native title bar or application header and available in approval prompts.
- The current session is the primary center pane.
- The contextual inspector remains optional and project-scoped.
- Narrow and medium responsive behavior may collapse the session sidebar into a drawer, but the drawer still contains only the current project's sessions.

## Session sidebar requirements

- Show a prominent **New session** action.
- Search only sessions from the current project.
- Order sessions by recent activity by default, with pinned sessions first if pinning is enabled.
- Show session title, concise state, approval-needed/unread indicator, and relative last activity.
- Support resume, rename, pin/unpin, archive/unarchive, and delete according to runtime capability.
- Never render a project heading above the session list merely to recreate the removed project hierarchy.
- Never aggregate sessions from multiple project windows, even if they share one runtime or provider account.
- Background session state shown in the sidebar must belong to the current project.
- Global cross-project notifications may identify another project, but activating one focuses that project's existing window or opens it through the normal project flow.

## Session terminology

The desktop UI should use **session** for the durable conversation/work unit. Existing requirement references to **session** in navigation, history, lifecycle, header, drafts, event resume, and inspector context should be interpreted as **session** unless they refer specifically to an agent turn or future background session.

The runtime/client contract should expose stable session IDs. UI copy must not inconsistently alternate among session, chat, thread, and session.

## Settings behavior

- Global settings are reachable through the application menu and launcher.
- Project settings are reachable from the current project window and clearly show project scope.
- Settings must distinguish global defaults, project overrides, session overrides, and one-turn selections.
- Opening global settings from a project window may use a dedicated application settings window or a modal/sheet appropriate to the platform; it must not turn the session sidebar into global navigation.
- Project settings must never offer an in-place project switch.

## Window identity and restoration

- Window title includes the project name and Suncode product name using platform conventions.
- Two projects with the same directory name are disambiguated through path, owner, or remote identity in window-management surfaces.
- Persist geometry, maximized/full-screen state, sidebar width, inspector width, selected session, and collapsed state per project window/device.
- On application restart, restoration follows a user preference: reopen previous project windows, show the launcher, or platform-default restoration.
- Restoration validates each project before enabling interaction and handles missing or unauthorized projects independently.
- A failed restored project must not prevent other project windows from restoring.
- Draft retention remains per session and project; drafts must never appear in another project window.

## Architecture amendments

### Application shell

The Qt application shell owns a process-level window manager with:

- One optional launcher window.
- A registry mapping canonical project/project IDs to project-window controllers.
- Open, focus, close, restore, and enumerate-window operations.
- Global settings and runtime-status presentation.
- Platform activation and recent-project integration.

The registry is device-local presentation state. The runtime remains authoritative for canonical project identity and availability.

### Project window controller

Each controller owns:

- Immutable project binding.
- One project-scoped client API facade.
- One project-scoped event subscription and reconnect cursor set.
- Session-list projection restricted to its project.
- Selected-session projection and per-session presentation state.
- Project window geometry and pane preferences.
- Project-scoped cancellation of client requests when the window closes.

It must reject or ignore any snapshot, event, approval, artifact, or mutation acknowledgement whose project/project scope does not match its immutable binding. A wrong-scope event is a security/diagnostic error, not merely a hidden row.

### Runtime API changes

The client-runtime contract must support:

- Canonicalize/open project and return stable project/project identity.
- List recent-project metadata separately from session lists.
- Detect whether an open request refers to an already known project identity.
- Query sessions by mandatory project/project scope.
- Subscribe and replay events by mandatory project/project scope.
- Resolve session, approval, artifact, settings, and model availability within that project scope.
- Return typed missing, moved, unavailable, unauthorized, incompatible, and remote-offline errors.

The API must not expose an unscoped `list all sessions across all projects` operation to the Phase 1 project-window UI.

### Startup flow

#### No project supplied

1. Start or attach to the authenticated runtime.
2. Load safe recent-project metadata and global settings.
3. Show the launcher.
4. Do not subscribe to project session streams.

#### Project supplied or restored

1. Start or attach to the authenticated runtime.
2. Canonicalize and authorize the project.
3. Focus an existing bound window or create a new controller.
4. Load project settings and project-scoped session summaries.
5. Establish the project-scoped event subscription.
6. Restore the selected session when valid.
7. Enable mutations only after scope and capability initialization succeeds.

## Security and failure requirements

- A project window must include its immutable project/project ID in every scoped client request.
- The runtime independently validates project, session, approval, and artifact relationships.
- A malicious or buggy client cannot retarget a session by changing display names or paths.
- Cross-project events, approvals, drafts, search results, file references, changes, and artifacts must never appear in the wrong window.
- An approval prompt identifies the project even though the window is already scoped, because native notifications and multiple windows may obscure origin.
- If a project becomes unavailable, its window enters a project-specific offline/error state; other windows remain usable.
- Runtime reconnect restores every project subscription independently and does not merge event cursors.

## Revised acceptance criteria

1. Opening two different projects produces two independent top-level project windows.
2. Each window's sidebar contains only sessions belonging to its bound project.
3. `Open Project...` from an existing project window leaves that window unchanged and opens or focuses another window.
4. Reopening an already open project focuses the existing window and does not create a duplicate project window.
5. Starting without a project shows the launcher, which contains recent projects but no session aggregation.
6. Closing one project window does not affect another project's selection, drafts, subscriptions, or active sessions.
7. Search, approvals, artifacts, settings, notifications, and reconnect replay remain correctly project-scoped.
8. Window restoration can recover multiple projects independently, including partial failure.
9. A cross-project event or response is rejected and logged as a scoped diagnostic error.
10. No primary project-window surface lists all projects or all sessions across projects.

## Test additions

- Open two projects and verify separate window identities and sidebar datasets.
- Invoke Open Project from project A, choose project B, and verify A is not retargeted.
- Open project A twice through menu, command-line handoff, and recent projects; verify one project window is focused.
- Verify project A session search cannot return project B sessions.
- Deliver cross-project events, approvals, artifacts, and mutation acknowledgements to a window fixture; verify rejection.
- Disconnect and reconnect with several project windows and independent event cursors.
- Restore several windows when one project is missing or unauthorized.
- Close a project window with active and idle sessions and verify the approved runtime lifecycle behavior.
- Verify global settings propagate appropriately while project settings remain isolated.
- Verify launcher accessibility and keyboard operation without exposing session history.

