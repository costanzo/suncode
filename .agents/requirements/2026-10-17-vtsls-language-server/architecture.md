# Architecture

## Implemented state

The Rust-owned language-server catalog contains jdtls and vtsls. The vtsls adapter owns JavaScript, JSX, TypeScript, and TSX language IDs, Node.js launch construction, `--stdio`, root matching, TypeScript SDK settings, bounded tsserver memory, and validation. The shared LSP runtime and five semantic tools remain language-neutral.

## Boundaries and dependencies

Rust remains authoritative for detection, version validation, launch construction, initialization options, configuration mapping, persistence, and runtime reconciliation. SDK bindings expose a typed vtsls DTO and write request. Avalonia presents the approved server-specific page. The design-system implementation in this delivery is review tooling only.

## Data and control flow

Settings loads the project catalog, opens the vtsls detail by stable kind, edits a typed draft, and submits one optimistic mutation. Rust persists the project record, reconciles only that project's vtsls process, and returns effective runtime state.

## Security and failure handling

vtsls runs without a shell in the project root using the filtered environment. Project files cannot choose the executable or Node runtime. Custom SDK paths are user-entered local configuration and must resolve to a TypeScript lib directory. Server requests that edit files or execute commands remain refused by the shared LSP client.

## Compatibility and migration

vtsls records use configuration version 3 with optional typed vtsls fields and a runtime settings object. Version-2 jdtls JSON remains readable through serde defaults and continues to be written unchanged apart from the empty shared settings object.

## Risks and rollback

Npm shims obscure the Node executable actually used, and vtsls settings track the VS Code TypeScript extension. The implementation resolves Unix symlinks and common Unix/Windows npm layouts to `vtsls.js`, uses the configured Node.js runtime directly, and surfaces unresolved launches as recoverable runtime failures.

## Decisions

- Persist the user-facing vtsls executable and launch the resolved server script through the configured Node.js executable.
- Require Node.js 18 or newer and expose the runtime override as project configuration.
