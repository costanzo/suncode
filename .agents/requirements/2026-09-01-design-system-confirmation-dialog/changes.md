# Changes

## Source

- Added a universal confirmation dialog composed from the shared modal shell.
- Added unique modal accessibility identifiers and safe initial-focus support.
- Changed the session Archive specimen interaction to require confirmation.
- Added archive confirmation states to the universal overlays and Sessions review pages.

## Contracts and generated artifacts

- None.

## Configuration and persistence

- None.

## Tests

- Passed `npm run build` in `design-system/`.
- Passed `git diff --check`.
- Passed local browser interaction review for cancel, Escape, explicit confirmation, initial focus, dark theme, and narrow layout.
- `npm run format:check` could not run because Prettier is missing from the current install and the configured package mirror failed dependency restoration with `SELF_SIGNED_CERT_IN_CHAIN`.

## Documentation

- Updated `DESIGN.md` with reusable confirmation-dialog behavior.
