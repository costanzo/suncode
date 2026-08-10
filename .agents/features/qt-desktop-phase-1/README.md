# Qt Desktop Phase 1

The Phase 1 desktop client is a Qt 6 Quick/QML application that talks to the Rust SDK facade.

It launches into a standalone project hub that lists recent projects without auto-selecting one. Opening a recent or newly chosen project creates a separate project window; closing the last project window returns to the hub, and closing the hub exits the app.

Inside a project window, it can list sessions, render conversation history, submit turns, handle approval prompts, inspect touched files, and trigger checkpoint undo. The left navigation lists sessions for the current project. The right process/review panel shows active turn state, runtime activity-derived process rows, approvals, checkpoints, touched files, and diagnostics. Activity events remain available in the runtime adapter but are not exposed as a separate central Activity page yet. It does not open SQLite or contact providers directly.

The desktop presentation uses the Quiet Control Desk visual system. The conversation canvas is the primary work surface; session navigation and process/review diagnostics are independent side bays that can be collapsed from the top bar. Presentation changes do not alter the Rust SDK boundary or runtime behavior.

Global settings are available from the hub and project windows. The current implementation exposes a left-side settings tree with Defaults, Appearance, and Model providers pages for DeepSeek, Zhipu GLM, and OpenAI. Project composers include a model selector for turn submission and disable sending when the selected model's provider is not configured.
