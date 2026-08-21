# Progress

- Status: Complete
- Last updated: 2026-08-17

## Completed

- Audited the desktop feature surface and Rust SDK C ABI.
- Chosen the additive `cdylib` plus P/Invoke integration.
- Implemented the .NET 10 Avalonia project hub, independent multi-project windows, settings, native SDK adapter, and ordered event projection.
- Reworked Avalonia against captured desktop hub, workbench, settings, Git drawer, and compact-layout baselines.
- Updated product, architecture, feature, specification, contract, design, contributor, SDK, and decision records.
- Matched in-window dialogs, the macOS project menu, shared runtime-handle lifetime, multi-window behavior, focus states, composer gating, and settings modality.
- Passed Debug and Release Avalonia builds, C# and Rust formatting checks, all 43 Rust tests, native startup checks, multi-window interaction checks, compact layout checks, screenshot comparisons, and `git diff --check`.

## In progress

- None.

## Blocked

- None.

## Log

### 2026-08-17

- Restored and verified native macOS shadows for the custom-chrome hub, project, and settings windows without changing other platforms, then softened their theme-specific outer hairlines to 0.5 DIP without altering internal borders or content geometry.
- Corrected the border-only macOS full-screen regression by using Avalonia's native-backed window-state transition instead of an untracked Objective-C toggle, retained the 4-by-6-DIP internal chrome spacing in full screen, removed manual frame and composition-surface changes that caused exit artifacts, and replaced the no-session composer with a centered empty state.

### 2026-08-16

- Requirement initialized and implementation started.
- Avalonia parity pass started.
- Avalonia final verification completed.
