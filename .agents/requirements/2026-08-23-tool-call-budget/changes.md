# Changes

## Source

- Made terminal failure persistence idempotently enrich already failed turns while protecting completed, cancelled, and interrupted rows.
- Added typed project `tool_call_limit` resolution, a 64 default, continuation snapshot compatibility, and whole-batch overflow rejection.
- Added project-aware SDK wrapper methods and a bounded Avalonia numeric setting.
- Bound Settings to the owning window's view model so a project window can read and save its own limit; hub-opened Settings remains intentionally disabled for this project-only value.

## Contracts and generated artifacts

- Documented setting scope, range, default, turn snapshot semantics, overflow behavior, and terminal error persistence.
- Added the project setting to the shared runtime SDK vectors. No generated artifacts are used.

## Configuration and persistence

- Reused the existing `configuration` table; no schema migration was required.
- Persisted complete `error_json` and `error_code` after a failed lifecycle projection.

## Tests

- Added database coverage for typed limits, failed-row enrichment, and protected terminal states.
- Added agent coverage proving an overflowing batch executes none of its calls.
- Extended SDK setting validation coverage.

## Documentation

- Updated runtime SDK, persistence, SQLite, runtime feature/specification, and Avalonia feature records.
