# Architecture

## Current state

The Rust catalog contains jdtls and vtsls. Shared lifecycle, document synchronization, workspace configuration, runtime status, and the five semantic tools are language-neutral.

## Proposed design

Add an immutable `basedpyright` catalog kind with a typed project adapter. The adapter launches `basedpyright-langserver --stdio`, supplies the Python language id, maps the selected interpreter and analysis policy to supported settings, and preserves committed project configuration precedence.

## Boundaries and dependencies

Rust remains authoritative for detection, launch construction, configuration mapping, persistence, and runtime reconciliation. The later production phase will add typed SDK and Avalonia fields. The design-system implementation in this delivery remains review tooling only.

## Data and control flow

Settings loads the catalog, opens the basedpyright detail by stable kind, edits a typed draft, and eventually submits one optimistic project mutation. Runtime settings use `python.pythonPath`, `basedpyright.analysis.diagnosticMode`, `basedpyright.analysis.typeCheckingMode`, and optional `basedpyright.analysis.configFilePath`.

## Security and failure handling

The language server runs without a shell in the project root using the filtered environment. Repository files cannot choose the executable or interpreter. Server requests that edit files or execute arbitrary commands remain refused by the shared LSP client.

## Compatibility and migration

Production persistence will require optional basedpyright-specific fields while retaining existing jdtls and vtsls records. No persistence change is part of design review.

## Risks and rollback

Interpreter discovery varies across virtualenv, uv, conda, Homebrew, and system installations. The design keeps detection visible and editable while treating project configuration files as the durable team-owned source for detailed checking rules.

## Open questions

- Final interpreter discovery priority.
- Final runtime behavior when configuration-file settings conflict with Settings values.
