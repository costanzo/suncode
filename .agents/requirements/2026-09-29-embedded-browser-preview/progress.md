# Progress

- Status: In progress
- Last updated: 2026-09-29

## Completed

- Added the design-system browser preview specimen and responsive right-pane composition.
- Added a CEF preview control with loopback URL validation, isolated project request context, popup blocking, and reload/close controls.
- Added a local Avalonia 12 compatibility build of CefGlue.Avalonia and wired CEF initialization/shutdown into the desktop lifetime.
- Added focused URL and responsive layout tests.
- Desktop test suite passes (135 tests) and the design-system production build passes.
- Added Rust-owned project preview process lifecycle with filtered environment and process-group cleanup.
- Added typed Rust SDK, C ABI, and C# methods for preview state, start, and stop; the Avalonia pane starts `npm run dev` through that path.
- Added a 30-second loopback port readiness gate and deterministic background-process cleanup.
- Added project-scoped preview configuration for program, args, working directory, and URL.
- Added reload generation tracking from project file fingerprints; Avalonia polls state and reloads CEF when the generation changes.
- Added explicit CEF native runtime package references and a macOS bundle deployment target; a Release `osx-arm64` publish now contains the CEF framework and four helper apps.

## In progress

- Native HMR event transport and Windows/Linux publish smoke tests remain to be implemented.

## Blocked

- None.

## Log

### 2026-09-29

- Requirement initialized and compatibility spike completed.
- Published CefGlue Avalonia adapter failed at runtime under Avalonia 12; local adapter patches compile and a packaged macOS spike loaded a localhost page.
