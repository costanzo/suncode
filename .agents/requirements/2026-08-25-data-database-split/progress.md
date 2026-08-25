# Progress

- Status: Complete
- Last updated: 2026-08-25

## Completed

- Reviewed the current `suncode-db` layout and established the dependency direction.
- Moved persistence code to `suncode-data` and added `suncode-database::sqlite`.
- Updated core dependencies, initialization, manifests, and current project documentation.

## In progress

- None.

## Blocked

- None.

## Verification

- `cargo test --workspace --lib`: passed.
- `cargo clippy --workspace --lib -- -D warnings`: passed.
- `git diff --check`: passed.
