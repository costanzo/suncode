# Changes

## Source

- Added `suncode-lsp` for bounded Content-Length JSON-RPC framing, initialization, document synchronization, semantic requests, cancellation, server-request handling, and local stdio process lifecycle.
- Added a Rust-owned project runtime manager and the five read-only agent tools: diagnostics, definition, references, hover, and document symbols.
- Added Rust/C/C# management methods and the Avalonia Language servers Settings page and editor window.

## Contracts and generated artifacts

- Advanced the hand-written C ABI contract to version 8 and documented the additive SDK methods and runtime semantics.
- No protocol or SDK contract is generated.

## Configuration and persistence

- Added the eighteenth SQLite table, `language_server`, with exact prior 17-table additive compatibility.
- Added validated global definitions, optimistic revisions, write-only environment patches, redacted read DTOs, and project-scoped runtime state.

## Tests

- Added protocol duplex tests, process-launch failure coverage, persistence compatibility and validation tests, SDK redaction/revision tests, failed-runtime save coverage, C ABI checks, and C# serialization coverage.
- Added core coverage for position conversion, language inference, and root-marker-aware server selection.

## Documentation

- Updated current product, architecture, decisions, feature, specification, persistence, schema, and SDK contract records.
- Kept the approved design-system Language Server Settings specification aligned with the production Avalonia surface.
