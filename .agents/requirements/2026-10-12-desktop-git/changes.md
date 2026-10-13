# Changes

## Source

- Added desktop Git command runner, service, local DTOs, and NUL/patch parsers.
- Updated availability, cancellation, stale-result handling, window activation, and saved drawer restoration.
- Removed Rust Git implementation and SDK/binding methods/DTOs; removed git2/libgit2 from four lockfiles.

## Contracts and configuration

- Native ABI is 18. No database schema/settings migration.
- Git is optional; no installer, Git mutation, or remote Git features.

## Tests

- Real repository integration, absence/re-detection, index immutability, helper suppression, output/cancellation limits, and view-model races.
- Retired Rust Git dispatch names fail closed.

## Documentation

- Updated architecture, SDK contract, features/specifications, design authority/specimens, and ADR-20261012-desktop-system-git.
