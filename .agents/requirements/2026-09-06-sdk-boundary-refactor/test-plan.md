# Test Plan

## Scope

Validate the extracted typed facade, C ABI, Cargo workspace, and Avalonia native-library packaging.
The Rust facade module split must preserve its crate-root exports, method signatures, DTO serialization, and runtime behavior.

## Unit tests

- Rust facade tests.
- C ABI envelope/version/handle tests.

## Regression checks

- Existing core agent tests.
- Avalonia build.

## Commands and results

Results:

- `cargo test --manifest-path agent/Cargo.toml --workspace --all-targets` — passed (85 tests).
- `cargo test --manifest-path sdks/rust/Cargo.toml --lib` — passed (13 tests).
- `cargo clippy --manifest-path sdks/rust/Cargo.toml --all-targets -- -D warnings` — passed.
- `cargo test --manifest-path sdks/c/Cargo.toml --lib` — passed (1 test).
- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj --no-restore` — passed.
- `dotnet build sdks/csharp/SunCode.Sdk.csproj --no-restore` — passed.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj --no-restore` — passed (58 tests).
- `git diff --check` — passed.
