# Avalonia Desktop Phase 1

**Status:** Implemented and focused-tested

SunCode's Phase 1 production client is a .NET 10 Avalonia desktop application under `apps/desktop-avalonia/`. The original Qt implementation under `apps/desktop-qt/` is retained as its parity reference. CLI, TUI, Web, mobile, and IDE clients remain deferred.

## Boundary

Avalonia XAML owns layout and semantic styles. C# view models own navigation and transient presentation state. The client embeds the Rust runtime through a hand-written P/Invoke adapter over C ABI version 1. It does not open SQLite, contact model providers, invoke Git, or read project files directly.

The Rust crate emits a `cdylib`; the desktop build invokes Cargo and copies the platform library beside the managed executable. Native calls run off the UI thread. All desktop windows share one reference-counted native runtime handle while keeping independent project/session state. SDK subscription callbacks are copied and marshalled through `Dispatcher.UIThread`; subscriptions close before the runtime handle.

## Implemented workflows

- project hub, local folder opening, independent project windows, duplicate-window activation, and recent project selection
- project-scoped session create, select, rename, and archive
- snapshot replay and ordered live conversation/activity events
- per-turn model selection, submission, queue status, and cancellation
- approval allow-once/deny decisions
- touched paths, turn checkpoints, conflict-aware undo, and diagnostics
- runtime-owned Git status and structured per-file diff review
- provider credential status/store/remove, default model, and dark/light theme settings
- responsive navigation and review bays with a stable conversation composer
- Qt-style in-window dialogs, application-modal settings, and the native macOS project menu
- one-to-one Qt-derived colors, geometry, states, shortcuts, assets, and font fallback stacks

## Verification

The source build verifies the Rust `cdylib` integration and compiled Avalonia bindings. Focused macOS startup checks compare the Qt and Avalonia project hub, workbench, compact layout, settings, and Git drawer while exercising native runtime loading, project listing/selection, diagnostics, and Git projections. Release signing and installer production remain separate release-engineering work.
