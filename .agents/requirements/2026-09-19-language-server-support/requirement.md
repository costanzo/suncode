# Requirement

## Background

SunCode currently understands code through bounded file reads, text search, Git inspection, and model reasoning. It lacks language-aware diagnostics, symbol, definition, reference, and hover queries. Language Server Protocol support should provide these semantics without turning the desktop into a general-purpose IDE or bypassing Rust ownership.

## Goals

- Add Rust-owned local stdio language-server lifecycle management scoped per project.
- Persist global language-server definitions and expose project runtime status through the SDK.
- Add bounded read-only semantic tools for diagnostics, definitions, references, hover, and symbols.
- Implement the approved Avalonia Settings surface through the existing Rust/C/C# SDK chain.
- Preserve path authority, cancellation, auditability, and honest process-isolation language.

## Non-goals

- Editable desktop editor, completion UI, rename, code actions, formatting, or server-requested edits.
- Remote LSP transports, repository-controlled auto-execution, bundled language servers, or Node/Bun runtime dependencies.
- Treating a language-server process as an OS sandbox or covering its external cache/toolchain effects with SunCode undo.
- Exposing arbitrary LSP methods to the model.

## Requirements

- Language servers launch only from explicit structured executable and argument configuration, never through a shell.
- Runtime instances are keyed by project and server definition; separate projects never share a process or document state.
- Configuration supports display name, command, arguments, language IDs, root markers, initialization options, write-only environment changes, startup/request timeouts, enabled state, ordering, and optimistic revision.
- Runtime states are `not_started`, `disabled`, `starting`, `indexing`, `ready`, and `failed`.
- Model-facing tools are fixed and bounded: `lsp_diagnostics`, `lsp_definition`, `lsp_references`, `lsp_hover`, and `lsp_symbols`.
- Tool paths remain project-relative or registered dependency aliases. Arbitrary absolute paths returned by servers are not exposed.
- Position input is one-based; the Rust adapter converts it to the negotiated LSP encoding.
- `workspace/applyEdit` and `workspace/executeCommand` are not supported by the first delivery.
- Avalonia accesses definitions and runtime state only through the managed SDK.

## Edge cases

- Missing executable, startup timeout, handshake failure, server crash, request timeout, cancellation, malformed response, and stale document version.
- Multiple configured servers matching one language.
- Project without a matching root marker or active runtime.
- Definitions/references outside the opened project or registered dependencies.
- Non-UTF-8, binary, too-large, deleted, and concurrently changed documents.
- Existing databases with the exact prior 17-table schema.

## Acceptance criteria

- A fake duplex stdio transport verifies framing, initialization, semantic requests, notifications, cancellation, and server-request handling; process launch failures are covered separately.
- Rust data, core, SDK, C ABI, C# SDK, and Avalonia focused tests pass.
- Settings implements the approved design-system page without direct database or process access.
- Agent tool contract tests cover all five semantic tools and path/position validation.
- `git diff --check` passes and unavailable checks are reported.

## Open questions

- None for this delivery.
