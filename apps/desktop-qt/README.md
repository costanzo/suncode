# Suncode Qt Desktop

Phase 1 contains the Qt client only. CLI, TUI, and Web clients are deferred.

The client is a Qt 6 Quick/QML application with a hand-written C++ adapter for the Rust SDK facade. It does not access SQLite, provider endpoints, or project files directly.

## Build

Requirements:

- Qt 6.5 or newer with `Quick` and `Concurrent`
- CMake 3.21 or newer
- A C++17 compiler
- Rust stable and `cargo`

```sh
cmake -S apps/desktop-qt -B apps/desktop-qt/build
cmake --build apps/desktop-qt/build
```

The Phase 1 client covers runtime connection, health/models, project opening, session create/rename/archive, conversation and activity replay, turn submission/cancellation, allow-once/deny approval decisions, file-touch inspection, diagnostics, and turn-level checkpoint undo with expiry/conflict status. It deliberately does not access SQLite, provider endpoints, or local project files directly.

By default the runtime stores its local data under `~/.suncode`, with SQLite at `~/.suncode/data/sqlite/runtime.sqlite3`.
