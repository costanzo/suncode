# Changes

## Source

- Renamed `runtime/` to `agent/` and the core crate to `suncode-agent`.
- Renamed Rust SDK types, lock/log surfaces, health/error identifiers, and C ABI symbols to harness terminology.
- Renamed the Avalonia binding directory and `RuntimeSdk` wrapper to `AgentSdk`.

## Contracts and generated artifacts

- Renamed current harness contract directories and shared vector filenames.
- Bumped the native ABI version from 1 to 2.

## Configuration and persistence

- New default database filename is `harness.sqlite3`.
- Existing `runtime.sqlite3` remains readable as a fallback.

## Tests

- Rust workspace tests passed.
- Avalonia desktop build passed with the renamed native library.

## Documentation

- Updated current product, architecture, feature, specification, contract, and startup documentation.
- Updated historical path references without changing historical requirement directory identities.
