# Test Plan

## Scope

Session configuration persistence, approval atomicity, policy behavior, session switching, and Full Control presentation.

## Unit tests

- Verify session Full Control defaults false and round-trips as a boolean.
- Verify Full Control permits known risky tools but not unknown tools.
- Verify resolving a stale approval with `allow_session` does not enable the grant.

## Integration and conformance tests

- Verify an approved session resumes and later risky calls skip approval.
- Verify the desktop loads and disables the selected session grant.

## Regression checks

- Existing allow-once and deny behavior remains unchanged.
- Project boundaries, argument validation, audit, and checkpoints remain enforced.
- Run Rust workspace and Avalonia tests.

## Manual checks

- Inspect the three approval actions at the minimum sidebar width.
- Inspect the Full Control warning in dark and light themes.

## Commands and results

- `cargo test --manifest-path runtime/Cargo.toml --workspace --quiet`: 35 + 3 + 35 + 24 tests passed.
- `dotnet test apps/desktop-avalonia-tests/SunCode.Desktop.Tests.csproj --no-restore`: 38 passed.
- `cargo fmt --manifest-path runtime/Cargo.toml --all -- --check`: passed.
- `git diff --check`: passed.
- `dotnet run --project apps/desktop-avalonia/SunCode.Desktop.csproj`: application started; the existing disabled-state sidebar was visually inspected at the current project-window width.

## Residual risks

- The enabled warning state was compile-verified through Avalonia bindings but was not forced into the user's live database for a screenshot, to avoid mutating an existing session grant.
