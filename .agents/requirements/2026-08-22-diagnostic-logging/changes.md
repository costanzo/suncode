# Changes

## Source

- Added level-aware Avalonia logging to `desktop.log`.
- Added level-aware Rust logging to `runtime.log`.
- Added bounded size-based rotation and retention for both files.
- Classified session switch stages, events, subscription lifecycle, and runtime snapshot boundaries.

## Contracts and generated artifacts

- None.

## Configuration and persistence

- Added the shared `SUNCODE_LOG_LEVEL` and `SUNCODE_LOG_DIRECTORY` environment variables.
- Added `SUNCODE_LOG_MAX_BYTES` and `SUNCODE_LOG_RETENTION` for log growth control.
- No SQLite or durable product state changes.

## Tests

- Rust runtime tests, formatting, desktop build, and diff checks passed.

## Documentation

- Added this requirement package.
