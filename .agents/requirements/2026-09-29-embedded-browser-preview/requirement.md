# Requirement

## Background

Developers want to chat while watching a local web application update in a wider embedded browser to the right. Existing Browser Use automates a separate Playwright Chromium and does not provide an embedded preview.

## Goals

- Embed CEF Chromium in the Avalonia project window on macOS arm64, Windows x64, and Linux x64.
- Keep chat visible in a narrower left column and preview in a wider resizable right column.
- Launch project development servers through Rust's audited process path and use their HMR for live updates.
- Keep preview state, profile, and lifecycle separate from Playwright Browser Use.
- Align the design-system specimen and production Avalonia UI.

## Non-goals

- Using CEF as the Agent's Browser Use target or synchronizing it with Playwright.
- Arbitrary remote or `file:` navigation in preview.
- Automatically guessing and executing project commands.

## Requirements

1. Pin and package CEF, its subprocess, and native dependencies for all supported targets.
2. Use a separate project profile under application data.
3. Accept only loopback HTTP preview URLs; reject credentials, popups, downloads, and browser-internal schemes by default.
4. Run servers through Rust policy and the audited process dispatcher; Avalonia never starts them directly.
5. Expose typed SDK preview state and events, including starting, ready, failed, and exited.
6. Keep chat, preview, approval access, and navigation usable at all supported window sizes and themes.
7. Release preview and server resources on project or application close without interrupting the agent.

## Edge cases

- Server advertises a URL before it is listening, exits, hangs, or changes port.
- User changes project during startup.
- CEF crashes while chat remains open.
- HMR is unavailable, so manual reload remains possible.

## Acceptance criteria

- A configured local web project launches and visibly updates after an agent file edit when HMR is available.
- Chat and preview are visible side by side in a wide window with useful minimum widths.
- Each supported target passes packaged offline startup and input checks.
- Focused Rust, SDK, design-system, and Avalonia checks pass.

## Open questions

- Verify which CEF binding works with Avalonia 12.1.1 on all three release targets.
