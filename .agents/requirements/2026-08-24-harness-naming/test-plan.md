# Test Plan

## Scope

Verify that the renamed Rust package and native ABI still build and that the Avalonia client links the new library.

## Unit tests

- Run the complete Rust workspace test suite.
- Verify the ABI version assertion and harness lock tests.

## Integration and conformance tests

- Build the Avalonia project, which invokes Cargo and copies the native harness library.
- Inspect current contracts and shared vector paths for the new names.

## Regression checks

- Search production files for old package, library, ABI, SDK type, and path identifiers.
- Run `git diff --check`.

## Commands and results

- `cargo test --manifest-path agent/Cargo.toml --workspace --quiet`: passed, 113 tests.
- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj --no-restore`: passed with zero warnings and errors.

## Residual risks

External native hosts compiled against ABI version 1 need to be rebuilt. Historical records intentionally retain runtime terminology where it describes the old architecture.
