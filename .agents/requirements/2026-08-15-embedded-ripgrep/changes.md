# Changes

## Source

- Added embedded ripgrep dependencies and replaced the content search implementation.
- Preserved the `search/find` response shape while adding regex matching and ripgrep standard filters.

## Contracts and generated artifacts

- No public response-schema change planned.

## Configuration and persistence

- None.

## Tests

- Added operations tests for regex, ignore/hidden filters, include globs, multiple matches, truncation, and invalid regex input.

## Documentation

- Updated the Rust core spec and feature record with the embedded-search behavior.
