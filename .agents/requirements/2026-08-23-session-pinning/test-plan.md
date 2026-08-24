# Test Plan

## Scope

Session `pin_at` persistence, ordering, archive behavior, runtime DTO shape, and Avalonia integration.

## Unit tests

- `suncode-db`: pin ordering, archive cleanup, and archived rejection.

## Integration and conformance tests

- `suncode-agent` unit suite and shared runtime vector JSON validation.

## Regression checks

- Avalonia desktop test project build and test suite.

## Manual checks

- Open a project, use the session overflow menu, pin/unpin a session, and confirm the row moves while the active conversation remains selected.

## Commands and results

- `PATH=... cargo test -p suncode-db`: passed, 37 tests.
- `PATH=... cargo test -p suncode-agent`: passed, 32 tests.
- `PATH=... cargo test -p suncode-agent`: passed, 32 tests.
- `dotnet build apps/desktop-avalonia-tests/SunCode.Desktop.Tests.csproj --no-restore`: passed.
- `dotnet test apps/desktop-avalonia-tests/SunCode.Desktop.Tests.csproj --no-build --no-restore`: passed, 39 tests.
- `git diff --check`: passed.
- `cargo fmt --all -- --check`: unavailable; rustfmt is not installed.

## Residual risks

- The UI was compile-verified but not screenshot-tested in a running desktop window.
