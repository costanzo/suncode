# CLI Session Administration

**Status:** Implemented and focused-tested

`suncode session list [PATH]` opens the canonical project through `AsyncAgentSdk` and returns all primary sessions, including archived records, together with SDK-projected UI states. Text mode prints stable session ID, projected state, title, and model. JSONL mode emits `session.list` with the typed SDK result.

`suncode session archive SESSION_ID` calls the SDK lifecycle method and emits `session.archived` with the returned session record. User ownership, primary/child restrictions, status mutation, timestamps, and persistence remain Rust SDK/core responsibilities; the CLI never opens SQLite or rewrites records.

One-shot session resume is implemented separately in `features/cli-session-resume/`. It reuses the existing session context but does not add interactive chat or approval/question resolution.
