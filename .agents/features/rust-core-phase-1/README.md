# Rust Operations Phase 1

**Status:** Implemented and focused-tested

The `suncode-tool` crate is the agent's narrow audited operations boundary. It is an in-process auditability boundary, not an OS sandbox and not a second authority owner.

## Implemented operations

- Canonical, project-scoped reads, writes, edits, glob traversal, and regular-expression grep with bounded output and repository ignore rules.
- BOM- and line-ending-preserving edits with overlap and precondition checks; safe parent-directory creation; pre-image checkpoints and conflict-aware restore.
- Read-only Git status and per-file diff inspection through vendored `git2`/libgit2. Results are project-relative and bounded; Git mutation, remotes, and credentials are out of scope.
- Structured program-plus-argv execution and platform-native shell scripts. Output streams are continuously drained, previews are bounded, complete oversized output is retained as an artifact, and cancellation terminates the process group/tree.
- Approval-gated HTTP(S) text retrieval with URL credential rejection, same-origin redirect checks, declared-charset decoding, parser-backed HTML conversion, 5 MiB raw-response bounds, 64 KiB model previews, and managed artifacts for the remainder.

All operations return typed results and business errors. Retired operation names fail closed. The model tool definitions and their audited implementations are maintained together under `agent/crates/tools`.

## Verification

Focused tests cover path boundaries, ignore traversal, regex validation, edit preconditions, checkpoints, Git projections, process failure/cancellation, WebFetch bounds, and artifact handling. See [`contracts/agent-sdk/README.md`](../../../contracts/agent-sdk/README.md) for the client-visible method surface.
