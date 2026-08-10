# Runtime Phase 1

The runtime is one Rust process with one SQLite database under `~/.suncode/data/sqlite/runtime.sqlite3`.

Startup acquires the runtime lock, opens the database, performs recovery, and serves the SDK/API surface. Session context is built from retained `session_messages`; streaming deltas are not stored durably.

Provider integration is Rust-owned. The built-in provider registry resolves `deepseek-v4-flash`, `glm-5.2`, and `gpt-5.6-sol` to trusted runtime adapters. `/credentials` returns redacted provider-keyed credential state, `/models` returns registered models with credential-derived availability, and provider API keys are loaded from plaintext SQLite `secret_records` or explicit non-interactive environment overrides only.
