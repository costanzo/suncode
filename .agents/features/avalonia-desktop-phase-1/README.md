# Avalonia Desktop Phase 1

**Status:** Implemented and focused-tested

SunCode's only Phase 1 production client is the .NET 10 Avalonia application under `apps/desktop-avalonia/`. C# and XAML own presentation and transient interaction state; the client calls the Rust SDK through a hand-written P/Invoke adapter and never opens SQLite, contacts providers, reads project files, invokes Git, or executes operations directly.

## Implemented workflows

- Project hub, recent/open projects, independent project windows, duplicate-window activation, and project-scoped session create/select/rename/archive/reopen/pin.
- Conversation streaming with normalized snapshots, latest-selection-wins loading, lagged-subscription resync, stable variable-height layout, copyable final responses, expandable process history, cancellation, queued input, and retryable loading errors.
- Hierarchical provider-and-model selection through one chat-composer menu, collapsible Settings provider navigation, editable and resettable provider URLs, credential status/store/remove, project default model, reasoning-effort control, tool-call budget, dark/light theme, and persisted diagnostics/logging settings.
- Approval deny, allow-once, and allow-for-session flows with readable scope/action details, raw request inspection, and persistent Full Control warning state.
- Structured question prompts with single-select, multi-select, custom answers, submit/skip, snapshot restoration, and live events. Current-turn todos restore from normalized conversation snapshots and update live.
- Tool activity details, touched paths, checkpoints, conflict-aware undo, runtime health/diagnostics, provider trace drawer, Git status/diff review, and read-only project dependency Explorer.
- Responsive navigation and review bays, in-window dialogs, keyboard toggles, native macOS menu integration, and the shared design-system review pages under `design-system/`.

The desktop logger opens its default rotating `desktop.log` during process startup, before persisted settings load. SDK operations and ViewModel actions record error-level failures with operation names; process, task, and Avalonia dispatcher unhandled exceptions are recorded with bounded, single-line exception chains. Diagnostics exclude credentials, prompts, model responses, tool inputs/results, file contents, and raw native envelopes.

## Boundary and verification

### Visual implementation notes

- Shared Avalonia theme resources use the design-system's neutral graphite/silver primary action palette with separate success, warning, and danger semantics in both dark and light themes.
- The desktop has five top-level window roles: ProjectHub, Workspace, DialogWindow, Settings, and About. Workspace alone uses `WindowDecorations=BorderOnly` with the 36 px custom title-bar content and compact traffic lights while delegating the outer border and resize behavior to the platform; it has no transparent outer margin, rounded frame wrapper, or manual resize handles. On macOS, double-clicking its title bar toggles normal and maximized states, while the green traffic light remains the full-screen action. ProjectHub, DialogWindow, Settings, and About use full system decorations without duplicate client-side title bars. DialogWindow hosts secondary confirmation and opens its dialog only after an explicit user action.
- ProjectHub uses a 62 px toolbar, 70 px recent-project rows, 24 px content insets, and a compact first-run empty state. The project workspace uses 4 px shell gaps, 26 px gutters, 272/312 px default side panes, mutually exclusive bottom drawers, and a 20 px status bar.
- Sessions use 48 px rows with fixed pin/status/action columns and a compact empty state. Explorer uses 30 px tree rows, 12 px depth increments, explicit chevrons, root/dependency semantic styling, monospace path subtitles, and horizontal scrolling for deep paths.
- Conversation uses 14 px assistant Markdown, compact process/tool rows, a 24 px no-session empty treatment, a recycling history container, and an active-turn work indicator. For models advertising image input, up to three bounded images can be uploaded from file or clipboard, previewed, removed before submission, submitted as message-owned references, and restored above their owning user message. Non-vision models keep image selection disabled.
- Review prioritizes approval and question requests before turn changes, touched files, runtime health, todos, and checkpoints. Git and provider trace use compact semantic hierarchies. Settings uses a 238 px surface-backed navigation column, 58 px toolbar, 32 px navigation rows, 30 px provider rows, shared two-column setting rows with a 220 px control column, provider overview and detail pages, megabyte log sizing, image storage, a 720 px content cap, 11 px hints, and compact warning/credential surfaces.
- Responsive workspace presentation hides review below 1100 px, navigation below 860 px, and gutters, drawers, and status details at 620 px or below while preserving user visibility preferences for restoration.

Native calls run off the UI thread, subscription payloads are copied and marshalled to `Dispatcher.UIThread`, and subscriptions close before the shared runtime handle. Focused tests are colocated under `apps/desktop-avalonia/tests/` and run with:

```text
dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj
```

CLI, TUI, Web, mobile, IDE-plugin, hosted, and Electron surfaces remain deferred.
