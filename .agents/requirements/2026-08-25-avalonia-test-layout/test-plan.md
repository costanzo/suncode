# Test Plan

## Scope

Source-tree layout, project reference resolution, and existing Avalonia test behavior.

## Commands and results

- `dotnet restore apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj --ignore-failed-sources` - passed.
- `dotnet build apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj --no-restore -p:BuildProjectReferences=false` - passed with 0 warnings and 0 errors.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj --no-build --no-restore` - passed; existing test suite executed successfully.
- `git diff --check` - passed.

## Residual risks

The full test-project build with project references enabled also invokes the embedded Rust build target. That target depends on `cargo` being available in the calling shell.
