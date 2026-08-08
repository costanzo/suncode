# Changes

## Source

- Add canonical `llm` module.
- Add `model_provider` registry and DeepSeek adapter modules.
- Add one-file-per-tool schema modules and registry.
- Rename model-facing implemented tools to match OpenCode built-ins: `read`, `write`,
  `edit`, `apply_patch`, `bash`, `glob`, and `grep`; the agent translates these
  names and argument shapes into the existing audited operations methods.
- Split the operations implementation by responsibility: filesystem reads, search, writes, mutations, processes, artifacts, and checkpoint capture/restore.
- Remove the obsolete monolithic `provider.rs` entry point after updating all in-tree callers.

## Contracts and generated artifacts

No wire contract changes. Tool schemas are still hand-written JSON values.

## Configuration and persistence

No changes.

## Tests

Move provider parser tests to the adapter module, retain agent round-trip coverage, and preserve checkpoint/journal regression coverage during operations extraction.

## Documentation

This requirement package records the module boundaries and migration status.
