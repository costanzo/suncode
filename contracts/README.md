# Contracts

These are the hand-written Phase 1 contracts shared by the Rust agent and native clients. They are normative current behavior, not generated schemas.

## Active contracts

- [`agent-sdk/README.md`](agent-sdk/README.md): embedded SDK lifecycle, methods, DTOs, errors, events, authority, and binding rules.
- [`persistence.md`](persistence.md): ownership, retention, recovery, configuration, and secret handling.
- [`sqlite-schema.md`](sqlite-schema.md): the current 15-table SQLite schema and projection rules.
Contract behavior is verified by focused Rust and Avalonia tests in the owning implementation packages. There is no generated schema or fixture directory; changes are made directly to the hand-written contract and its tests.
