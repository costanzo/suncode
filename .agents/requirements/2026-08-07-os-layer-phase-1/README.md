# Rust OS Layer Phase 1

- Date: 2026-08-07
- Status: Draft
- Related features: None; the OS layer is not implemented.
- Related specifications: `../../ARCHITECTURE.md`
- Related decisions: `../../DECISIONS.md` — `ADR-20260807-rust-boundary-rationale`, `ADR-20260807-runtime-owns-durable-state`, `ADR-20260807-hand-written-protocol-contracts`, `ADR-20260807-local-first-scope`, `ADR-20260807-domain-vocabulary`
- Related requirements: `../2026-08-07-agent-runtime-phase-1/README.md`

## Summary

Define the Rust OS layer: the audited execution point for machine-affecting work. It owns path canonicalization, file primitives, search, process execution, sandbox profiles, artifacts, checkpoints, and an operation journal. This is a requirements and architecture delivery only; no OS-layer behavior is implemented here.

## Scope note

Phase 1 is deliberately small — roughly eighteen methods. The boundary's value is auditability rather than OS-enforced isolation, and audit coverage degrades as the surface grows, so the core owns primitives and the runtime owns semantics.

Watchers, file catalog, content-search index, diff engine, VCS, PTY, and toolchain inspection are each deferred to their own package. Language-server orchestration and symbol indexing belong to the runtime permanently.

## Documents

- `requirement.md` - scope, ownership rules, and acceptance criteria
- `architecture.md` - operation categories, journal, sandbox, checkpoints, method backlog
- `changes.md` - affected documentation
- `plan.md` - later implementation sequence
- `progress.md` - documentation status
- `todo.md` - decisions and future work
- `test-plan.md` - verification strategy
