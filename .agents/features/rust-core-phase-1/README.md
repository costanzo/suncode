# Rust Operations Phase 1

**Status:** Implemented and focused-tested

The `suncode-tool` crate is the agent's narrow audited operations boundary. It is an in-process auditability boundary, not an OS sandbox and not a second authority owner.

## Implemented operations

- Canonical, project-scoped reads, writes, edits, glob traversal, and regular-expression grep with bounded output and repository ignore rules.
- BOM- and line-ending-preserving edits with overlap and precondition checks; safe parent-directory creation; pre-image checkpoints and conflict-aware restore.
- Read-only Git status and per-file diff inspection through vendored `git2`/libgit2. Results are project-relative and bounded; Git mutation, remotes, and credentials are out of scope.
- Structured program-plus-argv execution and platform-native shell scripts. Output streams are continuously drained, previews are bounded, complete oversized output is retained as an artifact, and cancellation terminates the process group/tree. Child processes receive the filtered baseline plus the six standard proxy variable spellings (`http_proxy`, `https_proxy`, `no_proxy`, `HTTP_PROXY`, `HTTPS_PROXY`, `NO_PROXY`) when present.
- Turn checkpoint paths are projected into `session_snapshot` and `watch_session` as `conversationTurns[*].changedPaths`. This array is always present. It lists each non-empty project-relative `session_checkpoint.relative_path` once, ordered by first checkpoint ordinal, then creation time, then path, and it includes restored and invalidated rows. Changes made by `bash` and Computer Use are not recorded. Older clients ignore the field and treat a missing value as empty. Reopened desktop sessions restore their changed-file list from it.
- Approval-gated HTTP(S) text retrieval with URL credential rejection, same-origin redirect checks, declared-charset decoding, parser-backed HTML conversion, 5 MiB raw-response bounds, 64 KiB model previews, and managed artifacts for the remainder.
- Project-scoped local stdio language-server clients with bounded JSON-RPC framing, filtered process environments, document synchronization, cancellation, and five normalized read-only semantic operations.

All operations return typed results and business errors. Retired operation names fail closed. The model tool definitions and their audited implementations are maintained together under `agent/crates/tools`.

## Verification

Focused tests cover path boundaries, ignore traversal, regex validation, edit preconditions, checkpoints, Git projections, process failure/cancellation, WebFetch bounds, and artifact handling. See [`contracts/agent-sdk/README.md`](../../../contracts/agent-sdk/README.md) for the client-visible method surface.
