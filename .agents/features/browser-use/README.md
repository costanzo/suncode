# Browser Use and Embedded Preview

**Status:** Browser Use is implemented and focused-tested, with packaged smoke tests only on macOS arm64. Embedded preview is partially implemented.

These are two separate capabilities. Each has its own profile, and they do not share state.

## Browser Use (CEF CDP)

The Rust `suncode-browser` client and core `BrowserManager` connect over WebSocket to the CEF remote debugging endpoint configured by the Avalonia client. Rust sends bounded CDP commands and owns policy, lifecycle, and artifacts; CEF owns Chromium and UI.

- `browser_use_enabled` is global and defaults to `false`. Enabling does not launch anything. The first browser call starts one project-scoped worker with a headed persistent Chromium context. Disabling retires the tools, cancels calls, and stops processes while keeping profiles. The host capability ceiling (for example, the CLI) can suppress Browser Use entirely.
- Each project has one persistent profile under the agent data directory. Profiles never live inside the project and never reuse the user's own browser profile. The worker receives a filtered environment, Rust-chosen profile and download paths, and the global proxy mode, and it never receives provider credentials.
- Model tools are available only to primary sessions: `browser_open`, `browser_navigate`, `browser_snapshot`, `browser_click`, `browser_fill`, `browser_press`, `browser_screenshot`, `browser_tabs`, and `browser_close_page`. Actions target a snapshot reference bound to the page revision, or a structured role, label, placeholder, test-id, or text locator. Raw CSS, XPath, and scripts are rejected. Only `http` and `https` navigation is allowed.
- Every browser call, including observation, has `BrowserAccess` risk and needs interactive approval even under Full Control. Non-interactive use is denied. Browser effects on remote systems and on profiles are never part of filesystem undo.
- Screenshots up to 5 MiB are stored as PNG artifacts. The model receives text metadata, not the image.
- `take_browser_control` gives the user an exclusive lease, and agent calls wait until it is released. Returning control invalidates earlier references. Desktop Settings shows runtime identity and health and offers show, return-control, restart, stop, verify, and clear-data actions. Clearing data requires a stopped runtime and a confirmation.

Not implemented: per-origin and per-action policy preflight with action-time confirmation categories, screenshot delivery to vision models, scroll/wait/select/console tools, packaged offline smoke tests on Windows x64 and Linux x64, and other browser engines.

## Embedded preview (CEF)

The desktop project window can show a CEF Chromium pane beside the chat. The pane renders a local development server that Rust starts and owns.

- Rust `PreviewManager` exposes `preview_state`, `start_preview`, and `stop_preview` through the Rust, C, and C# SDKs. Preview configuration lives in project-only settings: `preview_program`, `preview_args`, `preview_cwd`, and `preview_url`. The desktop defaults to `npm run dev` at `http://127.0.0.1:5173/`.
- The server starts as a structured background process with a filtered environment and `HOST=127.0.0.1`. Its working directory is project-scoped, and its process group is cleaned up on stop, project close, and SDK shutdown. The start is a user action, not a model tool call, so no approval prompt is involved.
- Only `http://localhost` and `http://127.0.0.1` URLs without credentials are accepted. Readiness means the port accepts connections within 30 seconds. If the process exits early or never listens, start fails with `preview_server_exited` or `preview_server_not_ready`.
- Live refresh works by polling. Rust bumps `reload_generation` when a bounded scan of project file sizes and modification times changes, ignoring `.git`, `node_modules`, `bin`, `obj`, and `target`. The pane reloads when the generation changes. The CEF pane blocks popups, uses an isolated per-project request context, and offers reload and close. It depends on a locally patched CefGlue Avalonia 12 adapter.

Not implemented: native HMR event transport, typed preview events (state is polled), Windows and Linux CEF packaging and smoke tests, and use of CEF as a Browser Use target.
