# Avalonia Desktop Phase 1

**Status:** Implemented and focused-tested

SunCode's only Phase 1 production client is the .NET 10 Avalonia application under `apps/desktop-avalonia/`. C# and XAML own presentation and transient interaction state; the client calls the Rust SDK through a hand-written P/Invoke adapter and never opens SQLite, contacts providers, reads project files, invokes Git, or executes operations directly.

## Implemented workflows

- Project hub, recent/open projects, independent project windows, duplicate-window activation, and project-scoped session create/select/rename/archive/reopen/pin.
- Conversation streaming with normalized snapshots, latest-selection-wins loading, lagged-subscription resync, stable variable-height layout, copyable final responses, expandable process history, cancellation, queued input, and retryable loading errors.
- Model selection, credential status/store/remove, project default model, reasoning-effort control, tool-call budget, dark/light theme, and persisted diagnostics/logging settings.
- Approval deny, allow-once, and allow-for-session flows with readable scope/action details, raw request inspection, and persistent Full Control warning state.
- Structured question prompts with single-select, multi-select, custom answers, submit/skip, snapshot restoration, and live events. Current-turn todos restore from normalized conversation snapshots and update live.
- Tool activity details, touched paths, checkpoints, conflict-aware undo, runtime health/diagnostics, provider trace drawer, Git status/diff review, and read-only project dependency Explorer.
- Responsive navigation and review bays, in-window dialogs, keyboard toggles, native macOS menu integration, and the shared design-system review pages under `design-system/`.

The desktop logger opens its default rotating `desktop.log` during process startup, before persisted settings load. SDK operations and ViewModel actions record error-level failures with operation names; process, task, and Avalonia dispatcher unhandled exceptions are recorded with bounded, single-line exception chains. Diagnostics exclude credentials, prompts, model responses, tool inputs/results, file contents, and raw native envelopes.

## Boundary and verification

Native calls run off the UI thread, subscription payloads are copied and marshalled to `Dispatcher.UIThread`, and subscriptions close before the shared runtime handle. Focused tests are colocated under `apps/desktop-avalonia/tests/` and run with:

```text
dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj
```

CLI, TUI, Web, mobile, IDE-plugin, hosted, and Electron surfaces remain deferred.
