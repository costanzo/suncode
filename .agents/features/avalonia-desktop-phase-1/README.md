# Avalonia Desktop Phase 1

**Status:** Implemented and focused-tested

SunCode's Phase 1 production client is a .NET 10 Avalonia desktop application under `apps/desktop-avalonia/`. CLI, TUI, Web, mobile, and IDE clients remain deferred.

## Boundary

Avalonia XAML owns layout and semantic styles. C# view models own navigation and transient presentation state. The client embeds the Rust runtime through a hand-written P/Invoke adapter over C ABI version 1. It does not open SQLite, contact model providers, invoke Git, or read project files directly.

The Rust crate emits a `cdylib`; the desktop build invokes Cargo and copies the platform library beside the managed executable. Native calls run off the UI thread. All desktop windows share one reference-counted native runtime handle while keeping independent project/session state. SDK subscription callbacks are copied and marshalled through `Dispatcher.UIThread`; subscriptions close before the runtime handle.

## Implemented workflows

- project hub, local folder opening, independent project windows, duplicate-window activation, and recent project selection
- project-scoped session create, select, rename, and archive
- normalized session snapshots and live conversation/activity events with snapshot resync after lag
- Codex-style turn timelines: running turns show ordered assistant progress and tool activity without copy actions; terminal turns retain that process history collapsed above the final visible, copyable assistant response and can expand it on demand, including after snapshot reload
- latest-selection-wins session loading that discards stale snapshots, projects replay data off the UI thread, atomically replaces the conversation message source, uses stable variable-height conversation layout, exposes loading and retryable error states, and starts only the selected session's live subscription
- per-turn model selection, submission, queue status, and cancellation
- approval deny, allow-once, and allow-for-session decisions with readable action summaries, focused command or target details, and expandable raw requests; session Full Control is persistently warning-styled above Agent Processes and can be turned off directly
- structured question prompts with single-select, multi-select, custom-answer, submit, and skip controls; pending questions restore from snapshots and live events
- current-turn todo list restored from normalized `conversationTurns[*].todos` snapshots and updated by live `todo.updated` events
- conversation tool activity cards that show concise operation summaries and open selectable request/result/error details on demand
- touched paths, turn checkpoints, conflict-aware undo, and diagnostics
- runtime-owned Git status and structured per-file diff review
- a resizable session trace drawer with an expandable turn/call/content tree; each call lazily exposes user, assistant, thinking, and tool-use entries alongside call-level request/response, timing, token, and cache diagnostics
- provider credential status/store/remove, default model, project tool-call limit, and dark/light theme settings
- responsive navigation and review bays with a stable conversation composer
- in-window dialogs, application-modal settings, and the native macOS project menu
- project-window keyboard toggles for navigation (`Command+1`) and Git review (`Command+9`), with Control equivalents
- consistent colors, geometry, states, shortcuts, assets, and font fallback stacks

## Verification

The source build verifies the Rust `cdylib` integration and compiled Avalonia bindings. Focused startup checks exercise native runtime loading, project listing/selection, diagnostics, and Git projections. Release signing and installer production remain separate release-engineering work.
