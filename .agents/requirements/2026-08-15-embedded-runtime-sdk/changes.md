# Changes

## Source

- Added `sdk.rs` with the embedded runtime lifecycle, public typed DTOs, named SDK methods, typed errors, direct subscriptions, and method-oriented C ABI.
- Reduced `lib.rs` to the runtime module boundary and public SDK exports.
- Replaced endpoint discovery with a restrictive data-directory runtime lock.
- Removed the standalone runtime binary, Axum router, synthetic HTTP requests, SSE adapter, authentication token, host/port configuration, and generic `request_json` export.
- Migrated all Qt project, session, model, credential, setting, snapshot, checkpoint, turn, cancellation, approval, health, and diagnostic calls to named C ABI functions.
- Added ABI version validation and shared ownership so asynchronous Qt calls keep the runtime alive safely.
- Changed direct event subscription to register live delivery before replay, de-duplicate by durable sequence, and recover lag from SQLite.

## Contracts and generated artifacts

- Replaced the HTTP client-runtime contract with a hand-written embedded runtime SDK contract.
- Renamed the shared client vector to `contracts/vectors/runtime-sdk.json` and converted method/path/status cases to named SDK calls and domain outcomes.
- No generated protocol artifacts are introduced.

## Configuration and persistence

- Removed client-facing host, port, token, and endpoint discovery configuration.
- Preserve the runtime data directory, SQLite location, provider endpoint configuration, and non-interactive credential rules.

## Tests

- Added focused SDK method, credential, C ABI version/envelope, runtime lock, and replay/live subscription tests.
- Passed 35 Rust workspace tests, the Qt desktop build, QML lint target, offscreen startup, Rust formatting, JSON validation, SDK symbol inspection, normal dependency inspection, and `git diff --check`.
- Clippy completed with one pre-existing `manual_clamp` warning in `runtime/crates/core/src/context.rs`.

## Documentation

- Added this requirement package and accepted embedded-runtime ADR.
- Updated durable architecture, product, feature, specification, SQLite contract, Qt README, and future TypeScript/Python SDK packaging documentation.
