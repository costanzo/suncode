# Test Plan

## Scope

Validate QML resource registration, cross-directory imports, and application startup after the source move.

## Regression checks

- Build `suncode-desktop` through CMake.
- Inspect the generated QML cache paths for every moved source.
- Run the Impeccable layout detector against the reorganized QML tree.
- Run `git diff --check`.

## Residual risks

Manual interaction and screenshot review are unchanged from the existing Qt UI delivery; this change only relocates sources and resource references.
