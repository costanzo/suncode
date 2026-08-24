# Test Plan

## Scope

Tool-card summaries, detail data projection, shell argument compatibility, and failed-state persistence.

## Unit tests

- Verify tool names map to concise operation summaries.
- Verify invalid shell input returns `invalid_arguments`.
- Verify non-empty legacy `command` input translates to the platform shell.

## Integration and conformance tests

- Run the focused Rust runtime tests.
- Run the Avalonia desktop test project.

## Regression checks

- Verify tool request/result data survives snapshot projection.
- Verify read-only JSON details preserve shell operators such as `&&` without `\\u0026` escapes.
- Verify no model-facing shell schema change.
- Verify `git diff --check`.

## Manual checks

- Review the tool card and detail overlay layout at the existing conversation width.
- Confirm the referenced turn's final request and failure code from SQLite.

## Commands and results

- `cargo test --manifest-path agent/Cargo.toml -p suncode-agent --quiet`: passed, 31 tests.
- `dotnet test apps/desktop-avalonia-tests/SunCode.Desktop.Tests.csproj --no-restore`: passed, 37 tests.
- `git diff --check`: passed.

## Residual risks

- The overlay has not been exercised on a Windows desktop host in this environment.
- The already-persisted malformed turn remains historical data; the new failed-state fix applies to future runs.
