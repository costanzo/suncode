# Test Plan

## Regression checks

- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj --no-restore`
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj --no-restore`
- `npm run build` in `design-system/`
- `git diff --check`

## Results

All commands passed on 2026-08-30. The .NET test suite passed 49 tests.
