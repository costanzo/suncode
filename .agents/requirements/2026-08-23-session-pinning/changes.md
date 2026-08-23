# Changes

## Source

- Added `pin_at` to the session DTO and database projections.
- Added project-local ordering and archive cleanup.
- Added Rust SDK/C ABI and Avalonia Pin/Unpin flow.
- Added the sidebar pin icon.

## Contracts and generated artifacts

- Updated the runtime SDK method table and shared vectors.
- No generated artifacts are used.

## Configuration and persistence

- Uses nullable `session.pin_at`; no new configuration key is created.

## Tests

- Added a database test for ordering, archive cleanup, and archived pin rejection.

## Documentation

- Added this requirement package.
