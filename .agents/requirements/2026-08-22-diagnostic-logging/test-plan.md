# Test Plan

## Scope

Logger initialization, level filtering, file separation, and session-switch diagnostics.

## Unit tests

- Existing Rust runtime unit tests cover the modified runtime crate.

## Integration and conformance tests

- No SDK contract changes; no shared vectors required.

## Regression checks

- Rust logger compiles with the runtime core and does not change the C ABI.
- Avalonia logger compiles with the desktop client and preserves stderr diagnostics.

## Manual checks

- Run with `SUNCODE_LOG_LEVEL=TRACE` and `SUNCODE_LOG_DIRECTORY=/tmp/suncode-logs`.
- Switch sessions and verify `desktop.log` and `runtime.log` contain separate records.

## Commands and results

- `cargo fmt --manifest-path runtime/Cargo.toml --all`: passed.
- `cargo test --manifest-path runtime/Cargo.toml -p suncode-runtime`: 21 passed.
- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj --no-restore`: passed.
- `git diff --check`: passed before final documentation-only additions.

## Residual risks

- There is no rotation or retention policy yet.
