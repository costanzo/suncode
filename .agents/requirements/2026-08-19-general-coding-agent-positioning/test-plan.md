# Test Plan

## Scope

Verify terminology consistency and ensure the existing Avalonia application still compiles and tests after the About copy change.

## Unit tests

- Run `apps/desktop-avalonia-tests`.

## Integration and conformance tests

- Build `apps/desktop-avalonia` with its embedded Rust dependency.

## Regression checks

- Scan tracked source and knowledge files for the retired positioning phrase.
- Confirm precise technical uses of local execution and storage remain intact.

## Manual checks

- Confirm the About subtitle reads `General-purpose coding agent`.

## Commands and results

- Repository-wide case-insensitive terminology scan: passed with no matches.
- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj --no-restore`: passed with zero warnings and zero errors.
- `dotnet test apps/desktop-avalonia-tests/SunCode.Desktop.Tests.csproj --no-restore`: passed, four tests.
- `git diff --check`: passed.

## Residual risks

- None identified beyond copy consistency.
