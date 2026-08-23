# Test Plan

## Scope

New bash schema, millisecond timeout translation, legacy shell compatibility, and desktop projection.

## Unit tests

- Assert the advertised tool is `bash` with required `command`.
- Assert `timeout=120000` becomes `timeout_ms=120000`.
- Assert invalid new timeout values fail closed.
- Assert legacy shell command input remains translatable.
- Assert invalid tool arguments are returned as a correlated tool result and the turn continues.

## Integration and conformance tests

- Run the runtime workspace tests.
- Run the Avalonia desktop test project.

## Regression checks

- Confirm no C ABI or SQLite schema changes.
- Run `cargo fmt --check` and `git diff --check`.

## Manual checks

- Inspect a new bash approval and tool detail request.

## Commands and results

- `cargo test --manifest-path runtime/Cargo.toml --workspace --quiet`: 33 + 3 + 33 + 24 tests passed.
- `dotnet test apps/desktop-avalonia-tests/SunCode.Desktop.Tests.csproj --no-restore`: 38 passed.
- `cargo fmt --manifest-path runtime/Cargo.toml --all -- --check`: passed.
- `git diff --check`: passed.

## Residual risks

- Provider-specific strict tool validation remains outside the runtime; compatibility translation prevents known historical aliases from failing.
