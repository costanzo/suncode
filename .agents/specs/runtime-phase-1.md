# Runtime Phase 1

The runtime is one Rust process with one SQLite database under `~/.suncode/data/sqlite/runtime.sqlite3`.

Startup acquires the runtime lock, opens the database, performs recovery, and serves the SDK/API surface. Session context is built from retained `session_messages`; streaming deltas are not stored durably.
