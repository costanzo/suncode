# Changes

## Source

- Simplified the model schemas for read, glob, grep, write, and edit.
- Implemented read offsets and limits, ignore-aware glob traversal, safe parent creation, multi-edit normalization, and failed bash state projection.
- Consolidated model-facing tool definitions with audited execution in `agent/crates/tools`; removed the duplicate `agent/crates/core/src/tools` module.

## Contracts and generated artifacts

- Preserved internal `max_results`, `query`, `replacements`, `program`, `args`, `cwd`, and `timeout_ms` compatibility fields.

## Tests

- Add focused schema, range-read, ignore-filter, multi-edit, and non-zero process status coverage.
