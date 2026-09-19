# Changes

## Source

- Added per-session gates and synchronized event projection/publication to `SessionEventHub`.
- Added atomic `subscribe_with_snapshot` establishment and cleanup on snapshot failure.
- Added `SessionWatch` and `AgentSdk::watch_session` to the Rust facade.
- Kept standalone snapshot and subscribe methods unchanged for compatibility.

## Contracts and generated artifacts

- Additive Rust SDK contract only; C ABI remains unchanged.

## Configuration and persistence

- No configuration or persistence schema changes.

## Tests

- Added nine focused event-hub tests covering isolation, lag, close, before/after watch ordering, live publication waiting, cleanup, and cross-session concurrency.
- Added a Rust SDK integration test for the returned snapshot and typed stream.
- Passed the agent workspace, Rust SDK, C binding, and desktop suites.

## Documentation

- Added this delivery package.
- Updated the embedded SDK contract, package README, architecture, durable feature/specification records, and decision index.
