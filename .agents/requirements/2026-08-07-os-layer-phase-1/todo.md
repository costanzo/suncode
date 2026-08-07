# Todo

## Documentation

- [x] Create OS-layer requirement package.
- [x] Define core/runtime ownership split.
- [x] Classify built-in OS-related tools.
- [x] Define operation lifecycle, artifacts, recovery, and security expectations.
- [x] Add sandbox scope and profile catalog.
- [x] Restate the boundary rationale as auditability and third-party containment rather than OS-enforced isolation.
- [x] Cut Phase 1 to a completable catalog and move watchers, indexes, diff, VCS, PTY, and toolchain inspection to their own packages.
- [x] Move language-server orchestration and symbol/dependency indexes to the runtime.
- [x] Add the operation journal and hash-based reconciliation.
- [x] Add the artifact mark-and-sweep protocol.
- [x] Add secret delivery without secrets in protocol bodies.
- [x] Add checkpoint capture and restore.
- [ ] Review with product/security/runtime architecture.
- [ ] Consolidate approved stable facts into specs or decisions before implementation.

## Decisions needed

- [ ] Final Phase 1 method list, kept as small as the capability list allows.
- [ ] Default approval policy per operation class for read, search, write, process, artifact, and checkpoint.
- [ ] Command execution sandbox strategy per supported OS, and what each profile actually enforces where primitives differ.
- [ ] Initial sandbox profile catalog and enforcement guarantees.
- [ ] Journal durability across core restart, and pre-image retention duration.
- [ ] Journal storage form: file or embedded store, given the core must not open the runtime database.
- [ ] Scoped secret reference mechanism and its resolution channel.
- [ ] Artifact and checkpoint retention defaults.
- [ ] Trigger and ordering for each deferred package.

## Implementation

- [ ] Not started by design.
