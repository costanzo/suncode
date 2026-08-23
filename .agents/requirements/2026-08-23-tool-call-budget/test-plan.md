# Test Plan

## Scope

Terminal turn persistence, project setting validation/resolution, agent batch enforcement, and Avalonia settings integration.

## Unit tests

- A projected failed turn accepts the later structured error write.
- Completed, cancelled, and interrupted turns are not overwritten.
- Project tool-call limit defaults to 64 and accepts only 1 through 256.
- SDK rejects the key outside project scope or with a non-integer/out-of-range value.
- An over-budget provider batch records all rejected calls as failed and executes none.

## Integration and conformance tests

- Existing settings C ABI carries the project value without a contract shape change.
- Avalonia builds with the bounded `NumberUpDown` and project-aware save path.

## Regression checks

- Existing approval continuation and successful tool execution remain green.

## Manual checks

- Open Settings with and without a selected project and inspect enabled/disabled states.

## Commands and results

- `cargo test -p suncode-db --lib`: passed, 40 tests.
- `cargo test -p suncode-runtime --lib`: passed, 34 tests.
- `cargo test --workspace`: passed, 113 tests plus doc tests.
- `cargo clippy -p suncode-runtime --lib -- -D warnings`: passed.
- `cargo clippy -p suncode-db --all-targets -- -D warnings`: passed.
- `dotnet build SunCode.Desktop.csproj`: passed with zero warnings and errors.
- `jq empty` for runtime SDK/provider vectors: passed.
- `git diff --check`: passed.
- Workspace all-target strict Clippy reached the existing unused `.enumerate()` warning in the core test helper.
- Workspace rustfmt check reports existing session-pinning formatting differences in `store.rs`; the new tool-budget code is formatted.
- Rebuilt Avalonia after binding Settings to the owning project window's view model: passed with zero warnings and errors.

## Residual risks

- Native Settings layout was build-verified but not exercised by an automated UI screenshot test.
