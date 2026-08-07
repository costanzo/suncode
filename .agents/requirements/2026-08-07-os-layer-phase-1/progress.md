# Progress

- Status: Draft
- Last updated: 2026-08-07

## Completed

- Reviewed approved product and architecture boundaries.
- Reviewed the Phase 1 runtime requirement and the OpenCode-informed additions.
- Defined the core as a trusted capability substrate rather than a second agent brain.
- Assigned ownership for built-in OS tools, artifacts, process execution, search, and filesystem mutation.
- Documented the runtime/core split for tool schemas, policy preflight, operation execution, and session persistence.
- Restated the boundary rationale: containment of third-party code, one auditable choke point, and performance — not OS-enforced isolation, since the runtime spawns the core as a child under the same user.
- Cut Phase 1 to a completable catalog. Watchers, file catalog, content-search index, diff engine, VCS, PTY, and toolchain inspection each moved to their own future package.
- Moved language-server orchestration and symbol/dependency indexing to the runtime, leaving the core with process lifetime and file primitives.
- Added the operation journal, making the core authoritative for whether a keyed mutation completed, with hash-based reconciliation for unknown outcomes.
- Added the artifact mark-and-sweep protocol, resolving who reclaims bytes whose references live in the runtime database.
- Added secret delivery through scoped references resolved off the request path.
- Added checkpoint capture and restore with hash verification and conflict reporting.
- Reduced the method backlog from roughly thirty to about eighteen.

## In progress

- Product, security, runtime, and contract review.

## Blocked

- Functional implementation is intentionally not started.
- The final method list, default approval policy, sandbox mechanisms per OS, journal durability, and the secret-reference mechanism must be decided first.

## Log

### 2026-08-07

- Requirement package initialized as documentation-only work.
- Recorded that read, write/edit/patch, glob, search, process execution, sandbox, artifacts, and checkpoints are core operations.
- Revised after architecture review. Two changes drove the rest: the boundary's justification changed from isolation to auditability, which argues for a narrow surface, and Phase 1 was cut to something a milestone can actually finish.
- Removed the LSP broker design. Placing per-language orchestration behind a hand-written protocol would make each new language a change in Rust and in both protocol implementations, for no security gain — the runtime can already read files through core reads.
- Added the three cross-boundary protocols that were previously undefined: operation journal and reconciliation, artifact collection, and secret delivery.
