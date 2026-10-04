# Language Servers

**Status:** Implemented and focused-tested

The Rust agent gives the model language-aware diagnostics, definitions, references, hover, and document symbols through user-configured local stdio language servers. `suncode-lsp` owns framing, initialization, capability negotiation, document sync, and cancellation. Core owns the project-scoped manager and path authority. Desktop Settings has a Language servers page and an editor window.

## What users can rely on

- Definitions are global rows in `language_server`. Each one has a display name, structured command and arguments, language IDs, root markers, initialization options, write-only environment changes, startup and request timeouts, enabled state, ordering, and an optimistic revision. Named SDK methods create, update, enable, delete, and retry definitions and start the project's servers.
- Each project and definition pair gets its own process. Separate projects never share a process or document state. Runtime state, capabilities, and document versions live only in memory. States are `not_started`, `disabled`, `starting`, `indexing`, `ready`, and `failed`. Servers are chosen by language ID and project root marker. A save succeeds even when startup fails, and the row shows the failed state so the user can retry.
- Servers run in the project root with no shell and a filtered environment. Repository files never supply a command. Language servers are not bundled.
- The model sees exactly five read-only tools: `lsp_diagnostics`, `lsp_definition`, `lsp_references`, `lsp_hover`, and `lsp_symbols`. Positions are one-based. Documents are synced through the audited project or dependency read path. Returned locations are project-relative paths or dependency aliases, and external absolute paths are never exposed.
- Bounds: 4 MiB per protocol message, 1,000 diagnostics per document, and at most 1 MiB of synced text per call (see contract). Missing servers or capabilities, timeouts, crashes, and malformed responses become recoverable `lsp_*` tool failures. They never make the agent or project unavailable.
- Server requests to edit files (`workspace/applyEdit`) or run commands (`workspace/executeCommand`) are refused. Language-server processes run with the user's OS authority, and their cache and toolchain effects are outside undo.

Not implemented: rename, code actions, formatting, completion, remote LSP transports, and arbitrary LSP methods.

Contract: [`contracts/agent-sdk/README.md`](../../../contracts/agent-sdk/README.md). Decision: `ADR-20260919-rust-owned-language-server-support`.
