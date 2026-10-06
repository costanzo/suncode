# Changes

## Source

- Add `agent/crates/remote`.
- Move remote transport and protocol implementation out of `sdks/rust`.
- Add the agent-side remote host and lifecycle adapter.

## Contracts and generated artifacts

No wire contract changes.

## Configuration and persistence

Use the existing global `configuration` keys and SQLite ownership.

## Tests

Retain and move remote unit tests; add host/lifecycle delegation coverage.

## Documentation

Update architecture and remote-control feature ownership after implementation.
