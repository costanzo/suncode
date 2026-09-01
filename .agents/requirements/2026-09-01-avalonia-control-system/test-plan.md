# Test Plan

## Scope

- Reusable control helper logic.
- Settings adoption of file selectors, provider management, and shared selectors.
- Workspace adoption of modal overlays and grouped model selection.

## Unit tests

- Provider metadata lookup and filter normalization helpers used by the reusable controls.

## Integration and conformance tests

- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj`
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj`

## Regression checks

- `git diff --check`

## Manual checks

- Open Settings and verify defaults, appearance, logging, network, and providers render with the shared controls.
- Verify grouped model selection in the chat composer.
- Verify session create/rename modal dismissal and submit behavior.

## Commands and results

- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj` — passed
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj` — passed, 55 tests
- `git diff --check` — passed

## Residual risks

- Visual template overrides for Avalonia primitives remain lightly tested without automated screenshot coverage.
