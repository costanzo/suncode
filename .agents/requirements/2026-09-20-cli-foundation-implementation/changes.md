# Changes

## Source

- Add the independent `apps/cli` Rust package and `suncode` binary.
- Add modular arguments, configuration, command, output, error, and lifecycle handling.
- Re-export the SDK business error type so Rust clients do not depend on `suncode-common` directly.

## Contracts and generated artifacts

- Update CLI status and implemented command subset.
- Add CLI Cargo lockfile.
- No generated protocol code.

## Configuration and persistence

- Implement `SUNCODE_OUTPUT`, `SUNCODE_COLOR`, and `SUNCODE_USER_ID` resolution.
- Reuse existing SDK bootstrap variables and SQLite credential persistence.
- No schema changes.

## Tests

- Add six binary unit tests and five executable integration tests.
- Run broad regression and Clippy.

## Documentation

- Add CLI README, durable feature record, and this delivery package.
- Update product, architecture, repository guidance, contracts, and decisions to distinguish foundation implementation from conversational CLI completion.
