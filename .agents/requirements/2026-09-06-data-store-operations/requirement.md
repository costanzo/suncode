# Requirement

## Background

`agent/crates/data/src/store.rs` contains connection lifecycle plus operations for every persisted table. It is difficult to navigate and obscures table ownership even though `operations/` already contains one module per table.

## Goals

- Keep `store.rs` focused on database connection, initialization, locking, and transaction infrastructure.
- Place table-owned operations in the corresponding `operations/<table>.rs` module.
- Keep cross-table event projection and recovery orchestration explicitly separate.
- Preserve the existing public `Store` API and persistence behavior.

## Non-goals

- Changing the SQLite schema or stored data.
- Changing SDK contracts or DTO serialization.
- Exposing SQLite to clients.

## Requirements

- Every public persistence operation must be implemented outside `store.rs` unless it concerns store lifecycle or database health.
- Operations spanning multiple tables must live in a clearly named orchestration module.
- Existing agent and SDK callers must continue compiling without API migration.

## Acceptance criteria

- Data, agent, Rust SDK, C SDK, C# SDK, and Avalonia focused checks pass.
- `store.rs` no longer owns table-specific public operations.
- `git diff --check` passes.
