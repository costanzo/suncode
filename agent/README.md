# Agent Workspace

## Dependency Ownership

`agent/Cargo.toml` owns dependency management for the entire agent workspace. The `[workspace.dependencies]` table is the single source of truth for dependency versions, sources, and default-feature decisions.

Every member under `agent/crates/` must reference dependencies with `workspace = true`. Member manifests may select crate-specific features at their dependency boundary, but they must not declare dependency versions, registry sources, git revisions, paths, or default-feature policy locally.

When adding or upgrading a dependency, update `agent/Cargo.toml` first and then regenerate `agent/Cargo.lock`. Do not add a second version declaration in a child crate.
