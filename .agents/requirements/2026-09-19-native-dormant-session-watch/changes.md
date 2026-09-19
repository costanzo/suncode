# Changes

## Source

- Added C ABI `watch_session` and `subscription_start`, advanced the exact ABI to 10, and retained immediate `subscribe_session` compatibility.
- Added one native subscription handle implementation supporting dormant and started states, close-before-start, lag translation, double-start rejection, and callback self-disposal.
- Added C# typed `SessionWatch` and native marshalling.
- Migrated Avalonia primary-session loading to create a dormant watch, apply snapshot and auxiliary state, install the current handle, then start callbacks.

## Contracts and generated artifacts

- Advanced the C ABI to version 10; event envelope remains unchanged.
- Updated Rust/C/C# SDK documentation and the embedded SDK contract.

## Configuration and persistence

- No configuration or schema changes.

## Tests

- Added native dormant/start/close and ABI coverage.
- Added managed `SessionWatch` start-once and disposal coverage.
- Existing desktop latest-selection and snapshot-projection suites continue to pass.

## Documentation

- Added this delivery package.
- Updated architecture, feature/specification records, persistence ABI documentation, and the accepted decision index.
