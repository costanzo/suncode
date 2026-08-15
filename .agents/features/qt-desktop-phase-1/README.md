# Qt Desktop Phase 1 (Parity Reference)

**Status:** Superseded for production on 2026-08-16 by `.agents/features/avalonia-desktop-phase-1/README.md` and `ADR-20260816-avalonia-desktop-client`; retained intact as the authoritative parity reference.

The Qt/QML/CMake source under `apps/desktop-qt/` remains buildable so the Avalonia client can be checked against its pages, responsive geometry, states, interactions, assets, and font-selection behavior. Qt is not part of the Avalonia production build or runtime dependency graph.

The Phase 1 desktop client is a Qt 6 Quick/QML application that embeds the Rust SDK static library and calls its method-oriented C ABI. It does not construct REST paths or connect to a client-facing runtime server.

It launches into a standalone project hub that lists recent projects without auto-selecting one. Opening a recent or newly chosen project creates a separate project window; closing the last project window returns to the hub, and closing the hub exits the app.

Inside a project window, it can list sessions, render conversation history, submit turns, handle approval prompts, inspect touched files, and trigger checkpoint undo. The left navigation lists sessions for the current project. The right process/review panel shows active turn state, runtime activity-derived process rows, approvals, checkpoints, touched files, and diagnostics. A Git gutter control opens a docked, resizable bottom drawer with all, staged, and unstaged changed-file scopes and an on-demand unified file diff. The footer shows a colored Git summary alongside the selected model and SDK-provided cumulative token usage for the selected session. Activity events remain available in the runtime adapter but are not exposed as a separate central Activity page yet. It does not open SQLite, read project files, inspect `.git`, or contact providers directly.

The desktop presentation uses the Quiet Control Desk visual system. The conversation canvas is the primary work surface; session navigation and process/review diagnostics are independent side bays that can be collapsed from the top bar. Presentation changes do not alter the Rust SDK boundary or runtime behavior.

On macOS, the project window keeps its custom title bar visible in fullscreen, with minimize disabled and close/fullscreen controls still available.

Global settings are available from the hub and project windows. The current implementation exposes a left-side settings tree with Defaults, Appearance, and Model providers pages for DeepSeek, Zhipu GLM, OpenAI, Kimi, Claude, and Gemini. The Defaults model selector is populated from the runtime's complete multi-model catalog and stores the selected stable model ID. Project composers include a model selector for turn submission and disable sending when the selected model's provider is not configured.
