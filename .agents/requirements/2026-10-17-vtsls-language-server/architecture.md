# Architecture

## Current state

The Rust-owned language-server catalog contains only jdtls. Its typed adapter owns Java language IDs, `-data`, JAVA_HOME handling, root matching, managed data paths, and validation. The shared LSP runtime and five semantic tools are language-neutral.

## Proposed design

Add an immutable `vtsls` catalog kind and typed project configuration. The adapter launches vtsls over stdio, supplies the JavaScript/TypeScript language IDs, maps the TypeScript SDK policy to supported vtsls settings, validates Node and SDK paths, and applies bounded tsserver memory and timeout settings.

## Boundaries and dependencies

Rust remains authoritative for detection, version validation, launch construction, initialization options, configuration mapping, persistence, and runtime reconciliation. SDK bindings expose a typed vtsls DTO and write request. Avalonia presents the approved server-specific page. The design-system implementation in this delivery is review tooling only.

## Data and control flow

Settings loads the project catalog, opens the vtsls detail by stable kind, edits a typed draft, and submits one optimistic mutation after design approval and production implementation. Rust persists the project record, reconciles only that project's vtsls process, and returns effective runtime state.

## Security and failure handling

vtsls runs without a shell in the project root using the filtered environment. Project files cannot choose the executable or Node runtime. Custom SDK paths are user-entered local configuration and must resolve to a TypeScript lib directory. Server requests that edit files or execute commands remain refused by the shared LSP client.

## Compatibility and migration

The existing version-2 project language-server record shape can continue to store built-in kind and common fields, but vtsls requires its own typed configuration fields. The production phase must decide whether persistence advances to a new config version or uses a kind-specific typed payload while preserving jdtls records.

## Risks and rollback

Npm shims obscure the Node executable actually used, and vtsls settings track the VS Code TypeScript extension. Keep the exposed settings narrow and validate the final resolved launch. The design-system change is independently revertible before contract changes.

## Open questions

- Direct npm bin versus direct server-script launch remains a production decision.
- Node runtime override semantics require confirmation before the typed SDK contract is finalized.
