# Test Plan

## Scope

Verify latest-selection-wins behavior, bounded collection notifications, virtualized conversation rendering, loading/error states, and unchanged runtime behavior.

## Unit tests

- Bulk replacement raises one reset notification and preserves item order.
- Snapshot projection preserves normalized session messages.
- Applying a snapshot replaces the message source, raises its binding property notification once, and requests one conversation scroll.

## Integration and conformance tests

- Build the Avalonia client against the existing Rust C ABI.
- Keep runtime workspace tests and SDK subscription tests passing.

## Regression checks

- Debug and Release desktop builds.
- C# formatting and design detection.
- `git diff --check`.

## Manual checks

- Open a project and immediately switch sessions.
- Rapidly switch A-B-A and confirm only A is rendered.
- Confirm the loading animation remains active while the snapshot is pending.
- Confirm the composer accepts drafting while Send remains disabled.
- Confirm light and dark themes retain the composer background.

## Commands and results

- `dotnet test apps/desktop-avalonia-tests/SunCode.Desktop.Tests.csproj --no-restore`: passed, 3 tests.
- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj -c Release --no-restore`: passed with 0 warnings and 0 errors.
- `dotnet format` for the desktop and test projects: passed.
- `cargo test --manifest-path runtime/Cargo.toml --workspace`: passed, 44 tests.
- `node .agents/skills/impeccable/scripts/detect.mjs --json apps/desktop-avalonia DESIGN.md`: returned `[]`.
- `git diff --check`: passed.
- `cargo fmt --manifest-path runtime/Cargo.toml --all -- --check`: did not pass because untouched `persistence.rs` and `sdk.rs` already differ from the current rustfmt output.

## Residual risks

- Live computer-use timing and visual confirmation of variable-height Markdown virtualization remain manual verification steps.
- Existing Rust formatting drift remains outside this client-only change.
