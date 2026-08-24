# SQLite Schema v14 Optimization

- Date: 2026-08-19
- Status: Superseded by `../2026-08-19-db-module-layout/`
- Related features: `features/agent-phase-1/`
- Related specifications: `contracts/sqlite-schema.md`, `contracts/persistence.md`
- Related decisions: `ADR-20260819-sqlite-schema-v14`, `ADR-20260808-rust-unified-runtime`

## Documents

- `requirement.md`
- `architecture.md`
- `changes.md`
- `plan.md`
- `progress.md`
- `todo.md`
- `test-plan.md`
- `table-analysis.md`

The later requirement removes schema versions and all database migration behavior because SunCode is treated as a new system. The useful table/index analysis and terminal snapshot cleanup were carried forward; the v14 compatibility and data-conversion conclusions are no longer current.
