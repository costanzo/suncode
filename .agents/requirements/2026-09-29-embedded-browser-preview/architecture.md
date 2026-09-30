# Architecture

## Current state

The desktop uses Avalonia 12.1.1. Existing Browser Use uses a bundled Node.js/Playwright/Chromium worker owned by Rust, with no embedded rendering surface. Project Workspace has navigation, a fluid conversation center, a narrow review bay, and a bottom drawer.

## Proposed design

Add a CEF-backed `BrowserPreviewPane` in the desktop. A Rust `PreviewManager` owns local development-server lifecycle, project identity, URL validation, readiness, and status events. Existing `BrowserManager` continues to own Playwright Browser Use. They use separate profiles and do not claim synchronized state.

## Boundaries and dependencies

`Avalonia pane -> C# SDK -> C ABI -> Rust SDK -> PreviewManager -> audited process dispatcher`. The pane displays an accepted URL and handles rendering; it does not inspect project files, choose commands, or launch processes. CEF is a UI runtime, not an agent or policy engine.

## Data and control flow

The user starts a configured preview command. Rust validates project scope, evaluates policy, launches the server, confirms its loopback endpoint responds, then publishes ready state. Avalonia loads the URL into CEF. File changes flow through the development server's HMR connection; manual reload remains available. On server exit, the pane shows a stopped state.

## Security and failure handling

Allow `http://localhost` and `http://127.0.0.1` preview URLs. Reject credentials and non-loopback URLs. Deny unrequested popups, downloads, browser-internal schemes, and permission prompts. The CEF profile stays outside the project and never contains provider credentials. Browser failure does not block chat.

## Compatibility and migration

This adds an optional UI and additive SDK methods; it does not alter Browser Use tools or session history. CEF and its native helpers must be version-locked and verified before release.

## Risks and rollback

Main risks are Avalonia 12 binding compatibility, CEF event-loop integration, macOS signing, Linux shared libraries, and input focus. Keep preview opt-in until all target smoke tests pass. Rollback hides the pane without changing Playwright profiles.

## Open questions

- Complete native startup and packaging verification for macOS arm64, Windows x64, and Linux x64.
