# Rust Operations Phase 1

**Status:** Implemented and focused-tested

The `suncode-tool` crate is the agent's narrow audited operations boundary. It is an in-process auditability boundary, not an OS sandbox and not a second authority owner.

## Implemented operations

- Canonical, project-scoped reads, writes, edits, glob traversal, and regular-expression grep with bounded output and repository ignore rules.
- Read-only PDF, Word, presentation, and spreadsheet tools delegate parsing to `suncode-documents`; see [`office-document-read`](../office-document-read/README.md) for supported formats and verification gaps.
- BOM- and line-ending-preserving edits with overlap and precondition checks, safe parent-directory creation, and content-addressed before/after file versions for turn diff review.
- Structured program-plus-argv execution and platform-native shell scripts. Output streams are continuously drained, previews are bounded, complete oversized output is retained as an artifact, and cancellation terminates the process group/tree. Child processes receive the filtered baseline plus the six standard proxy variable spellings (`http_proxy`, `https_proxy`, `no_proxy`, `HTTP_PROXY`, `HTTPS_PROXY`, `NO_PROXY`) when present.
- Net file changes from `write`, `edit`, and `delete` are stored in `session_turn_change` and projected into `session_snapshot` and `watch_session` as `conversationTurns[*].changes`. This always-present array lists unexpired paths in first-touch order with status, line counts, and binary state. Repeated mutations collapse into one baseline-to-latest change, and net-zero paths disappear. SDK `turn_diff` renders hunks from content-addressed gzip versions stored by UTC day, with 30-day retention. Each child turn has its own projection.
- Approval-gated HTTP(S) text retrieval with URL credential rejection, same-origin redirect checks, declared-charset decoding, parser-backed HTML conversion, 5 MiB raw-response bounds, 64 KiB model previews, and managed artifacts for the remainder.
- Project-scoped local stdio language-server queries use the separate `suncode-lsp` runtime and tool provider, with bounded framing, filtered environments, document synchronization, cancellation, and five read-only semantic operations under core policy. These are not implemented by `suncode-tool` itself.

All operations return typed results and business errors. Retired operation names fail closed. Built-in filesystem/process/document/WebFetch tool definitions and their audited implementations are maintained under `agent/crates/tools`; LSP, MCP, Browser Use, and Computer Use have their own Rust tool providers governed by core policy.

File-change review covers the versions captured by the built-in filesystem tools. Shell processes, interactive terminals, MCP side effects, browser actions, and Computer Use have separate execution paths. The current SDK exposes retained net changes and diffs.

## Verification

Focused tests cover path boundaries, ignore traversal, regex validation, edit preconditions, checkpoints, rejection of retired Git operations, process failure/cancellation, WebFetch bounds, and artifact handling. See [`contracts/agent-sdk/README.md`](../../../contracts/agent-sdk/README.md) for the client-visible method surface.
