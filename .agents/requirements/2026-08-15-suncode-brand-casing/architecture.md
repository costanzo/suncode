# Architecture

## Current state

The canonical brand casing is inconsistent across presentation text and PascalCase integration identifiers.

## Proposed design

Use `SunCode` for the brand and PascalCase names. Retain lowercase machine-facing names as stable compatibility surfaces.

## Boundaries and dependencies

The change touches Qt/QML presentation, application metadata, opaque SDK handle type names, the icon resource name, and documentation. Runtime behavior and persistent data formats do not change.

## Data and control flow

No data or control flow changes.

## Security and failure handling

No security changes. Build and startup checks catch mismatched QML URI or resource casing.

## Compatibility and migration

The `suncode_*` C ABI, Cargo package names, environment variables, executable name, and `~/.suncode` directory remain compatible. No migration is required.

## Risks and rollback

The primary risk is a case mismatch between QML imports, generated resource paths, or the macOS icon. Reverting the casing-only changes restores the previous names.

## Open questions

- None.
