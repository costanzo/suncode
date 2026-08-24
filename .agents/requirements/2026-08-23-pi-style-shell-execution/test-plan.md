# Test Plan

## Scope

Timeout translation, process output draining and artifact preservation, cancellation, and existing cross-platform process behavior.

## Unit tests

- Verify timeout seconds convert to milliseconds and invalid values fail.
- Verify large output completes and produces an artifact.
- Verify an already-cancelled operation terminates its child and returns cancelled.

## Integration and conformance tests

- Run operations and runtime crate test suites.
- Run the complete Rust workspace test suite.

## Regression checks

- Verify structured argv translation remains unchanged.
- Verify platform shell selection and process start error classification remain unchanged.
- Verify no model tool schema advertises timeout_ms.

## Manual checks

- Review the final diff for temporary-file cleanup and platform-specific process termination.

## Commands and results

- cargo fmt --manifest-path agent/Cargo.toml --all: passed.
- cargo test --manifest-path agent/Cargo.toml -p suncode-tool -p suncode-agent: passed, 53 tests plus doc tests.
- cargo test --manifest-path agent/Cargo.toml --workspace --quiet: passed, 90 tests.
- dotnet test apps/desktop-avalonia-tests/SunCode.Desktop.Tests.csproj --no-restore: passed, 34 tests.
- git diff --check: passed.

## Residual risks

- Windows process-tree behavior requires a Windows host run for full validation.
- Streaming tool-output events remain a future client/API enhancement; the runner currently streams into capture threads rather than client events.
