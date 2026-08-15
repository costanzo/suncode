# Changes

## Source

- Updated Qt application metadata and user-visible copy to `SunCode`.
- Renamed QML module namespaces and resource URLs to `SunCode`.
- Renamed PascalCase SDK opaque handle types to `SunCode...`.
- Renamed the macOS icon resource to the stable lowercase technical name `suncode-desktop.icns`.

## Contracts and generated artifacts

- Updated SDK contract prose while retaining all lowercase ABI symbols.

## Configuration and persistence

- No configuration keys or persistence paths changed.

## Tests

- Built the Rust SDK and Qt desktop application successfully.
- Passed all 35 Rust workspace tests and Rust formatting checks.
- Passed `all_qmllint` with the repository's existing warnings.
- Reached the Qt event loop in an offscreen startup smoke test with no QML runtime output.
- Passed brand consistency and `git diff --check` checks.

## Documentation

- Updated project, architecture, product, feature, SDK, design, and historical requirement documentation.
