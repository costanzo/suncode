# Changes

## Source

- Added Rust persistence, SDK/C ABI, directory listing, dependency-aware agent routing, and read-only enforcement.
- Added C# SDK wrappers, Explorer view/model logic, gutter switching, lazy tree loading, and add/remove actions.
- Added a files gutter icon.

## Contracts and generated artifacts

- Updated runtime SDK prose and shared vectors with dependency methods and DTOs.
- Updated the SQLite contract to the 14-table manifest and documented the additive bootstrap behavior.
- No generated contracts or types are used.

## Configuration and persistence

- Added `project_dependency` with project cascade ownership and per-project canonical-root uniqueness.
- Dependency canonical roots stay inside Rust-owned persistence.

## Tests

- Added store uniqueness/removal and additive bootstrap tests.
- Added SDK browsing and overlap rejection coverage.
- Added stable alias result normalization coverage.

## Documentation

- Updated architecture, Phase 1 feature/spec records, and the decision index.
