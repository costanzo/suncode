# Rust Operations Phase 1

**Status:** Implemented and focused-tested

The `suncode-tool` crate is the agent's narrow audited operations boundary. It is an in-process auditability boundary, not an OS sandbox and not a second authority owner.

## Implemented operations

- Canonical, project-scoped reads, writes, edits, glob traversal, and regular-expression grep with bounded output and repository ignore rules.
- Registered dependencies are additional working roots under the existing session policy. Built-in file tools accept their real absolute paths and legacy aliases; `bash.workdir` selects a registered root before process dispatch. Dependency results use real paths, and retained file-change keys use dependency-qualified identities. See [editable dependencies](../editable-dependencies/README.md).
- Read-only PDF, Word, presentation, and spreadsheet tools delegate parsing to `suncode-documents`; see [`office-document-read`](../office-document-read/README.md) for supported formats and verification gaps.
- Strict single/batch exact-text edits with unique-match and overlap checks, preservation of untouched mixed LF/CRLF/CR, UTF-8/BOM and UTF-16LE round trips, and a 16 MiB source/result bound. Conflicts identify failing items and bounded candidate positions; successful calls return counts, changed ranges and bounded before/after previews. Conversation-derived read-state notices flag changed or unknown observations without blocking a still-valid exact match.
- SHA-256 preconditions for file overwrites and deletions, safe parent-directory creation, staged atomic file publication with permission preservation and an in-process mutation gate, and content-addressed before/after versions for turn diff review. The final baseline check is optimistic with respect to arbitrary external writers; there is no OS-level compare-and-swap guarantee. Full-source reads, writes, and edits return a fixed-size digest for subsequent overwrites or deletions.
- Structured program-plus-argv execution and platform-native shell scripts. Output streams are continuously drained, previews are bounded, complete oversized output is retained as an artifact, and cancellation terminates the process group/tree. Child processes receive the filtered baseline plus the six standard proxy variable spellings (`http_proxy`, `https_proxy`, `no_proxy`, `HTTP_PROXY`, `HTTPS_PROXY`, `NO_PROXY`) when present.
- Net file changes from `write`, `edit`, and `delete` are stored in `session_turn_change` and projected into `session_snapshot` and `watch_session` as `conversationTurns[*].changes`. This always-present array lists unexpired paths in first-touch order with status, line counts, and binary state. Repeated mutations collapse into one baseline-to-latest change, and net-zero paths disappear. SDK `turn_diff` renders hunks from content-addressed gzip versions stored by UTC day, with 30-day retention. Each child turn has its own projection.
- Approval-gated HTTP(S) text retrieval with URL credential rejection, same-origin redirect checks, declared-charset decoding, parser-backed HTML conversion, 5 MiB raw-response bounds, 64 KiB model previews, and managed artifacts for the remainder.
- Project-scoped local stdio language-server queries use the separate `suncode-lsp` runtime and tool provider, with bounded framing, filtered environments, document synchronization, cancellation, and five read-only semantic operations under core policy. These are not implemented by `suncode-tool` itself.

All operations return typed results and business errors. Retired operation names fail closed. Built-in filesystem/process/document/WebFetch tool definitions and their audited implementations are maintained under `agent/crates/tools`; LSP, MCP, Browser Use, and Computer Use have their own Rust tool providers governed by core policy.

File-change review covers the versions captured by the built-in filesystem tools. Shell processes, interactive terminals, MCP side effects, browser actions, and Computer Use have separate execution paths. The current SDK exposes retained net changes and diffs.

## Verification

Focused tests cover path boundaries, ignore traversal, regex validation, edit preconditions, checkpoints, rejection of retired Git operations, process failure/cancellation, WebFetch bounds, and artifact handling. See [`contracts/agent-sdk/README.md`](../../../contracts/agent-sdk/README.md) for the client-visible method surface.

The edit reliability changes pass all 72 tool tests on Windows, including concurrent readers, competing writers, failed checkpoint cleanup, protected DACL preservation, mixed endings, bounded diagnostics/results, larger-file reads/edits, and UTF-16LE retained diffs. Core regression passes 72 tests with the independent `agent_lock::tests::prevents_a_second_agent_without_publishing_an_endpoint` test excluded: that unchanged test also fails when compiled alone on this Windows host because the lock error is `Uncategorized` rather than `AlreadyExists`. Unix rename/mode preservation has source and a platform-gated test but still needs native platform execution. See the operation specification for metadata and external-writer limits.
