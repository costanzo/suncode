# Test Plan

## Scope

Fresh schema construction, logging setting defaults and validation, runtime reconfiguration compilation, and Avalonia settings parsing.

## Unit tests

- Verify the four global rows retain their JSON types and defaults.
- Verify the current schema/index manifest contains singular `project` names.
- Verify logging keys are global-only and reject invalid types, levels, sizes, and retention counts.

## Integration and conformance tests

- Run the complete Rust workspace test suite.
- Build the Avalonia application against the unchanged native settings contract.

## Regression checks

- Search production source and current logging documentation for `SUNCODE_LOG_*` and physical `projects` SQL references.
- Run Rust formatting and `git diff --check`.

## Manual checks

- With a fresh data directory, verify `runtime.log` and `desktop.log` are created under its `logs` directory.
- Open Settings > Logging, change each field, save, reopen the page, and verify the values persist.
- Enter an invalid size or retention count and verify saving is rejected with a visible message.
- Change global logging settings through the SDK and verify Rust uses them immediately and Avalonia uses them on its next settings load.
- Preserve any existing incompatible database; do not test by deleting or rewriting user data.

## Commands and results

- `cargo fmt --manifest-path agent/Cargo.toml --all -- --check`: passed.
- `cargo test --manifest-path agent/Cargo.toml --workspace`: passed.
- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj --no-restore`: passed with 0 warnings and 0 errors.
- `git diff --check`: passed.

## Residual risks

- Existing databases with `projects` require an explicit future migration or a user-selected fresh data directory.
- Avalonia applies a setting changed by another host on the next settings reload; Rust applies SDK writes immediately.
