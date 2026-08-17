# Changes

## Source

- Reformatted `schema.sql` into reviewable table and index definitions.
- Added schema version 14 migration preparation and application.
- Released terminal suspended-turn continuation snapshots during migration and future terminal transitions.
- Removed unused legacy tables and redundant indexes.
- Added focused partial/global indexes and one-active-secret uniqueness.

## Contracts and generated artifacts

- Updated the SQLite and persistence contracts to version 14.
- Updated the durable architecture and decision index for caller-owned reconnect cursors and schema normalization.

## Configuration and persistence

- Migrates legacy user settings with newest-timestamp-wins semantics.
- Reconciles duplicate active provider secrets before applying the unique partial index.
- Preserves pending/resuming continuation snapshots and all live runtime projections.

## Tests

- Added fresh version 14 shape coverage.
- Added version 13 upgrade coverage for settings, snapshots, secret reconciliation, removed objects, integrity, and foreign keys.
- Added terminal snapshot lifecycle coverage.

## Documentation

- Added a table-by-table design and residual-risk analysis.
- Updated the runtime Phase 1 feature record.
