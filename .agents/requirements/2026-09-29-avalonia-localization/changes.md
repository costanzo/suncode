# Changes

## Source

- Add Avalonia localization service and embedded English/Chinese resource dictionaries.
- Add the Appearance language selector and global locale persistence.
- Replace affected presentation literals and generated status copy with localization lookups.

## Contracts and generated artifacts

- No Rust protocol or C ABI changes.
- `ui_locale` uses the existing generic global settings contract.

## Configuration and persistence

- New global setting: `ui_locale`.
- Default: `en-US`.

## Tests

- Add locale fallback and resource-parity tests.
- Add Settings language persistence and runtime-refresh coverage where practical.

## Documentation

- Update `DESIGN.md`, design-system Settings specimen, and Avalonia feature documentation after implementation.
