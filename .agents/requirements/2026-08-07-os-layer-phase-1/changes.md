# Changes

## Documentation changes

- Added `.agents/requirements/2026-08-07-os-layer-phase-1/README.md`.
- Added `.agents/requirements/2026-08-07-os-layer-phase-1/requirement.md`.
- Added `.agents/requirements/2026-08-07-os-layer-phase-1/architecture.md`.
- Added `.agents/requirements/2026-08-07-os-layer-phase-1/plan.md`.
- Added `.agents/requirements/2026-08-07-os-layer-phase-1/progress.md`.
- Added `.agents/requirements/2026-08-07-os-layer-phase-1/todo.md`.
- Added `.agents/requirements/2026-08-07-os-layer-phase-1/test-plan.md`.

## Product changes

- None. This is a documentation-only requirement package.

## Architecture changes proposed

- Clarify that built-in OS-related tools are runtime model-facing tools backed by core-owned operations.
- Add explicit core ownership for read, write, edit, patch, glob, search, process execution, sandbox, artifacts, and checkpoints.
- Add sandbox as a first-class core subsystem with a named profile catalog.
- Add operation lifecycle, capability enforcement, recovery, and resource-limit expectations.

### Revision, 2026-08-07

- Restated the boundary rationale as third-party containment and auditability rather than OS-enforced isolation, and derived the narrow-surface rule from it.
- Cut Phase 1 to a completable catalog. Watchers, file catalog, content-search index, diff engine, VCS, PTY, and toolchain inspection each moved to their own future package.
- Moved language-server orchestration to the runtime and removed the core LSP broker and its methods. A language server is now an ordinary sandboxed process operation.
- Split index tiers across the boundary: file catalog and content search are core; symbol, dependency, and embedding indexes are runtime.
- Added the operation journal, making the core authoritative for whether a keyed mutation completed, plus hash-based reconciliation with an explicit conflict outcome.
- Added the artifact mark-and-sweep protocol with the runtime marking and the core sweeping.
- Added secret delivery through scoped references resolved off the request path.
- Added checkpoint capture and restore with hash verification and conflict reporting.
- Added the two-channel transport so cancellation and heartbeats bypass bulk output.
- Reduced the method backlog from roughly thirty to about eighteen and dropped `process.writeStdin`.
- Renamed "workspace" to "project"; replaced generated-contract language with hand-written contracts and shared test vectors.

## Files intentionally not changed

- No source code, contracts, features, specs, or decisions were changed by this package. The decisions it depends on were recorded separately in `.agents/DECISIONS.md`.

