# Architecture

## Current state

The embedded Rust agent owns tools, policy, persistence, MCP, process execution, and the SDK boundary. Avalonia Settings already manages MCP definitions through named C#/C/Rust methods. No LSP protocol adapter, lifecycle manager, persistence row, semantic tool, or production UI exists.

## Proposed design

Add `agent/crates/lsp` as a protocol adapter for local stdio LSP processes. It owns Content-Length framing, initialization, capability negotiation, document synchronization, bounded requests, notification handling, cancellation, and shutdown. It owns no SQLite, SDK DTOs, policy, project identity, or UI state.

Core adds a project-scoped `LanguageServerManager`. It loads desired definitions from `suncode-data`, starts matching enabled definitions, tracks runtime generations and state, resolves one server for a file/language, normalizes locations, and executes the five allowlisted semantic operations. The agent loop treats those operations as built-in read-only tools after ordinary validation and records them in `session_tool_use`.

## Boundaries and dependencies

- `suncode-lsp` depends only on common contracts, Tokio, Serde, JSON, URL handling, and the Rust standard library.
- `suncode-data` and `suncode-database` own durable definitions.
- Core owns authority, project/dependency path mapping, tool orchestration, and runtime composition.
- `suncode-tool` owns the five static model-facing schemas and validation entry points, but not protocol state.
- Rust SDK, C ABI, and C# SDK expose named management methods and typed DTOs.
- Avalonia owns only presentation and transient editor-window state.

## Data and control flow

1. A definition is validated and persisted before runtime reconciliation.
2. Project activation starts enabled matching definitions without blocking the project open path.
3. The manager initializes the server with the canonical project root and caches negotiated capabilities.
4. A semantic tool validates path and position, reads the current file through Rust authority, synchronizes it with a monotonically increasing document version, issues the allowlisted LSP request, normalizes and bounds the result, and records the ordinary tool result.
5. Disable, update, or delete retires the old runtime before subsequent requests resolve it.

## Security and failure handling

- Commands are structured and launched without a shell using a filtered environment derived from the existing local-process baseline.
- Repository files never supply executable commands automatically.
- LSP-returned URIs are canonicalized and mapped only to the current project or registered read-only dependencies.
- Server stderr, errors, initialization options, and environment secrets are bounded/redacted before diagnostics.
- Server requests that would mutate files or execute commands receive an explicit unsupported response.
- Runtime failure removes semantic availability but does not make the agent or project unavailable.

## Compatibility and migration

Add one `language_server` table and a narrowly validated additive upgrade from the exact prior 17-table schema. Existing sessions and tool rows require no migration. New SDK methods are additive and the C ABI version advances.

## Risks and rollback

The main risks are protocol edge cases, heavy indexing, subprocess authority, provider tool-count growth, and stale semantic results. Bounded timeouts, one process per project/definition, fixed tool names, document versions, and explicit runtime states limit these risks. Rollback disables/removes definitions and semantic tools; the additive table remains recognized.

## Open questions

- None for the first delivery.
