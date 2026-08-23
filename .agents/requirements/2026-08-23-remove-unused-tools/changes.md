# Changes

## Source

- Reduced agent dispatch and policy to the seven canonical model tools.
- Removed patch, unused filesystem, capability-wrapper, asynchronous process, and recovery dispatcher paths.
- Kept the private synchronous process runner used by `bash`, including cancellation and output artifacts.
- Removed stale desktop aliases and updated persisted test fixtures to canonical names.

## Contracts and generated artifacts

- Removed retired operation rows from the runtime-core contract and updated shared vectors to canonical methods.

## Configuration and persistence

- No changes planned.

## Tests

- Rust workspace and Avalonia tests pass.
- Formatting, production-library clippy, source scans, and diff checks pass.

## Documentation

- Updated runtime feature/specification and operations contract documentation.
