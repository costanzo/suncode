# Architecture

## Implemented state

The immutable Rust-owned catalog contains jdtls, vtsls, basedpyright, and rust-analyzer. Shared lifecycle, document synchronization, runtime status, and five bounded semantic tools remain language-neutral. The rust-analyzer adapter launches the configured executable with no arguments over its default stdio transport and supplies typed workspace settings.

## Proposed design

The fixed `rust-analyzer` catalog kind supplies the Rust language id and maps typed project settings to supported rust-analyzer workspace settings.

## Boundaries and dependencies

Rust remains authoritative for detection, launch construction, configuration mapping, persistence, and runtime reconciliation. The design-system implementation is review tooling only and does not become an Avalonia runtime dependency.

## Data and control flow

Settings loads the catalog, opens the rust-analyzer detail by stable kind, edits a typed draft, and submits one optimistic project mutation. The mapping covers Cargo features, all targets, build scripts, procedural macros, check-on-save command, and an optional target triple.

## Security and failure handling

The language server must run without a shell in the project root using the filtered environment. Repository files cannot select the executable. Server requests that edit files or execute arbitrary commands remain refused by the shared LSP client. Detection and initialization failures remain visible without discarding saved configuration.

## Compatibility and migration

rust-analyzer records use configuration version 5 and a nested typed settings object in the existing JSON column. Existing version 1-4 records remain readable through serde defaults, so no database table migration is required. ABI 22 adds the typed rust-analyzer mutation.

## Risks and rollback

rustup shims, standalone packages, custom toolchains, cross-compilation targets, build scripts, and procedural macros vary across systems. The design keeps detection visible and makes expensive analysis choices explicit. Rollback can remove the catalog entry and typed clients while leaving version-5 rows inert; older records remain compatible.

## Decisions

- Detection accepts only an executable that returns a valid `rust-analyzer --version` response.
- SunCode sends stable `rust-analyzer.cargo`, `procMacro`, `checkOnSave`, and `check` settings and exposes no raw JSON.
