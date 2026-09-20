# Changes

## Source

- Add public SDK open options and host capability types.
- Add async and blocking option-aware open methods.
- Pass immutable capability ceilings into core Browser and Computer managers.
- Gate backend initialization, catalogs, execution, permissions, controls, and SDK setting mutations.

## Contracts and generated artifacts

- Update Rust SDK and CLI contracts.
- Preserve C ABI 13 and managed contracts.
- No generated artifacts.

## Configuration and persistence

- No schema or setting changes.
- Persisted desired enablement remains untouched by a capability-disabled host.

## Tests

- Add persisted-enabled/host-disabled integration coverage.
- Run broad Rust, native, and desktop regressions.

## Documentation

- Add this delivery package and update architecture, features, specifications, SDK documentation, and decisions.
