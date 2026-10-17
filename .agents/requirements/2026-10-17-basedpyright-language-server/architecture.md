# Architecture

## Implemented state

The Rust catalog contains jdtls, vtsls, and basedpyright. The basedpyright adapter launches `basedpyright-langserver --stdio`, supplies the Python language id, maps the selected interpreter and analysis policy to supported settings, and preserves committed project configuration precedence. Shared lifecycle, document synchronization, workspace configuration, runtime status, and the five semantic tools remain language-neutral.

## Boundaries and dependencies

Rust remains authoritative for detection, launch construction, configuration mapping, persistence, and runtime reconciliation. Rust, C ABI 21, C#, and Avalonia expose typed basedpyright configuration. The design-system implementation remains review tooling only.

## Data and control flow

Settings loads the catalog, opens the basedpyright detail by stable kind, edits a typed draft, and submits one optimistic project mutation. Runtime settings use `python.pythonPath`, `basedpyright.analysis.diagnosticMode`, `basedpyright.analysis.typeCheckingMode`, and optional `basedpyright.analysis.configFilePath`.

## Security and failure handling

The language server runs without a shell in the project root using the filtered environment. Repository files cannot choose the executable or interpreter. Server requests that edit files or execute arbitrary commands remain refused by the shared LSP client.

## Compatibility and migration

basedpyright records use configuration version 4 with optional typed fields in the existing JSON column. Existing jdtls and vtsls records remain readable through serde defaults; no database table migration is required.

## Risks and rollback

Interpreter discovery varies across virtualenv, uv, conda, Homebrew, and system installations. The design keeps detection visible and editable while treating project configuration files as the durable team-owned source for detailed checking rules.

## Decisions

- Interpreter discovery prefers project `.venv`/`venv`, active conda/virtualenv state, host PATH and standard paths, then user pyenv/local paths.
- basedpyright project configuration files remain authoritative and may override Settings type-checking defaults.
