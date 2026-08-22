# Changes

## Source

- Added level-aware Avalonia logging to `desktop.log`.
- Added level-aware Rust logging to `runtime.log`.
- Added bounded size-based rotation and retention for both files.
- Classified session switch stages, events, subscription lifecycle, and runtime snapshot boundaries.

## Contracts and generated artifacts

- None.

## Configuration and persistence

- Added shared global `log_level`, `log_directory`, `log_max_bytes`, and `log_retention` rows in `configuration`.
- Rust loads and validates the settings; Avalonia consumes them through the SDK.

## Tests

- Rust runtime tests, formatting, desktop build, and diff checks passed.

## Documentation

- Added this requirement package.
