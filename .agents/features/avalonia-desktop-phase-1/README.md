# Avalonia Desktop Phase 1

**Status:** Implemented and focused-tested

SunCode's only Phase 1 production client is the .NET 10 Avalonia application under `apps/desktop-avalonia/`. C# and XAML own presentation and transient interaction state; the client calls the Rust SDK through a hand-written P/Invoke adapter and never opens SQLite, contacts providers, reads project files, invokes Git, or executes operations directly.

## Implemented workflows

- Project hub, recent/open projects, independent project windows, duplicate-window activation, and project-scoped session create/select/rename/archive/reopen/pin.
- Conversation streaming with normalized snapshots, latest-selection-wins loading, lagged-subscription resync, stable variable-height layout, copyable final responses, expandable process history, cancellation, queued input, and retryable loading errors.
- Hierarchical provider-and-model selection through one chat-composer menu, credential status/store/remove, project default model, reasoning-effort control, tool-call budget, dark/light theme, and persisted diagnostics/logging settings.
- Approval deny, allow-once, and allow-for-session flows with readable scope/action details, raw request inspection, and persistent Full Control warning state.
- Structured question prompts with single-select, multi-select, custom answers, submit/skip, snapshot restoration, and live events. Current-turn todos restore from normalized conversation snapshots and update live.
- Tool activity details, touched paths, checkpoints, conflict-aware undo, runtime health/diagnostics, provider trace drawer, Git status/diff review, and read-only project dependency Explorer.
- Responsive navigation and review bays, in-window dialogs, keyboard toggles, native macOS menu integration, and the shared design-system review pages under `design-system/`.

The desktop logger opens its default rotating `desktop.log` during process startup, before persisted settings load. SDK operations and ViewModel actions record error-level failures with operation names; process, task, and Avalonia dispatcher unhandled exceptions are recorded with bounded, single-line exception chains. Diagnostics exclude credentials, prompts, model responses, tool inputs/results, file contents, and raw native envelopes.

## Boundary and verification

### Visual implementation notes

- Shared Avalonia theme resources use the design-system's neutral graphite/silver primary action palette with separate success, warning, and danger semantics in both dark and light themes.
- Production windows use a shared 36 px title bar, 14 px outer frame radius, compact traffic lights, and the existing native drag, resize, minimize, maximize, and close behavior.
- ProjectHub uses a 62 px toolbar, 70 px recent-project rows, 24 px content insets, and a compact first-run empty state. The project workspace uses 4 px shell gaps, 26 px gutters, 272/312 px default side panes, mutually exclusive bottom drawers, and a 20 px status bar.
- Sessions use 48 px rows with fixed pin/status/action columns and a compact empty state. Explorer uses 30 px tree rows, 12 px depth increments, explicit chevrons, root/dependency semantic styling, monospace path subtitles, and horizontal scrolling for deep paths.
- Conversation uses 12 px message typography with compact process/tool rows, a 24 px no-session empty treatment, and an active-turn work indicator. Composer image attachments are session-owned placeholder state: up to three images can be uploaded from file or clipboard, previewed, removed, and opened in a larger preview window; the current text-only SDK submit-turn contract still ignores them, so they persist separately and are not cleared by text submission.
- Review, Git, and provider trace surfaces use compact semantic cards, 30/22 px Git file/diff rows, and 54/48/40 px trace hierarchy rows. Settings uses a 238 px navigation column, 58 px toolbar, 720 px content cap, 11 px hints, and compact warning/credential surfaces.
- Responsive workspace presentation hides review below 1100 px, navigation below 860 px, and gutters, drawers, and status details at 620 px or below while preserving user visibility preferences for restoration.

Native calls run off the UI thread, subscription payloads are copied and marshalled to `Dispatcher.UIThread`, and subscriptions close before the shared runtime handle. Focused tests are colocated under `apps/desktop-avalonia/tests/` and run with:

```text
dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj
```

CLI, TUI, Web, mobile, IDE-plugin, hosted, and Electron surfaces remain deferred.
