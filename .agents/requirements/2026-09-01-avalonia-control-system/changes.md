# Changes

## Source

- Add reusable control components under `apps/desktop-avalonia/Controls/`.
- Add shared control styles under `apps/desktop-avalonia/Styles/Application/ControlStyles.axaml`.
- Update Settings, ProjectWorkspace, and ChatInput to adopt the reusable controls.

## Contracts and generated artifacts

- No contract or generated-artifact changes.

## Configuration and persistence

- No persistence schema changes.
- Existing settings, credential, and provider-endpoint save flows remain unchanged.

## Tests

- Add focused unit coverage for reusable helper logic where practical.
- Run desktop build, desktop tests, and diff checks.

- Added `ControlHelpersTests` for extension normalization and provider metadata fallback.
- Verified:
  - `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj`
  - `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj`
  - `git diff --check`

## Documentation

- Populate this requirement package for the new delivery.
