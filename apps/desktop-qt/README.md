# Suncode Qt Desktop

Phase 1 contains the Qt client only. CLI, TUI, and Web clients are deferred.

The client is a Qt 6 Quick/QML application with a hand-written C++ adapter for the Rust SDK facade. It does not access SQLite, provider endpoints, or project files directly.

## QML layout

QML is organized by ownership rather than kept in one flat directory:

```text
qml/
├── app/                    # Application entry points and window shells
├── features/
│   ├── project/             # Project and session navigation
│   ├── conversation/        # Conversation and composer surface
│   ├── review/              # Agent activity, approvals, and undo review
│   └── settings/            # Global provider, model, and appearance settings
└── shared/
    ├── components/          # Reusable controls, labels, and themed SVG icons
    ├── navigation/          # Sidebar affordances
    ├── theme/               # Semantic design tokens
    └── window/              # Frameless window state and resize behavior
```

Feature modules may depend on `shared/`. The `app/` layer composes features and owns window lifecycle. Shared modules must remain independent of feature modules. Keep this as one `Suncode.Desktop` QML module; directory imports make ownership explicit without introducing runtime or protocol boundaries.

UI icons live as monochrome SVG resources under `assets/icons/` and are rendered through `shared/components/ThemeIcon.qml` so QML owns semantic theme and interaction colors. Do not add hand-drawn `Canvas` icons to feature files.

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
