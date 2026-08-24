# Requirement

## Background

The agent core was renamed from `runtime` to `harness`, but `harness` was not a useful product-facing name for the core that owns agent behavior, policy, persistence, and audited operations.

## Goals

- Name the production Rust core and embedded SDK `agent` consistently.
- Keep lower-level executor terminology such as Tokio runtime when it describes the async library.
- Treat the project as new and do not preserve data or native integrations from prior names.

## Non-goals

- Changing agent behavior, persistence schema, provider behavior, or operation policy.
- Rewriting historical requirement prose or decision identifiers.

## Requirements

- Rename current production directories, crates, native symbols, desktop bindings, contracts, feature/specification paths, and user-facing diagnostics from `harness` to `agent`.
- Bump the C ABI from 2 to 3 because the exported symbol family and DTO names changed.
- Use `agent.sqlite3` as the only default database path.
- Store credentials only through the current agent-owned SQLite credential surface.

## Acceptance criteria

- No production build or current contract uses the old harness core package, native library, ABI symbols, SDK type, or health field.
- Rust workspace tests pass and the Avalonia desktop project builds against the renamed native library.
- The new database path and current credential surface are covered by focused tests without compatibility branches.
