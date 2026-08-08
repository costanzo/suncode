# Progress

- Status: Complete
- Last updated: 2026-08-09

## Completed

- Inspected current Rust and Qt boundaries and OpenCode's provider/tool layout.

## Completed

- Implemented the canonical LLM boundary, provider registry, per-tool modules, and Qt/QML component split.

## Blocked

- None.

## Log

### 2026-08-09

- Initialized the requirement package.

### 2026-08-09

- Moved DeepSeek HTTP/SSE code below `model_provider/deepseek`.
- Added the canonical `llm` contract and model catalog/registry.
- Added one-file tool declarations and explicit operations routing modules.
- Aligned the model-facing built-in tool names with OpenCode's `packages/core/src/tool/`
  names for implemented Phase 1 capabilities: `read`, `write`, `edit`, `apply_patch`,
  `bash`, `glob`, and `grep`.
- Split the Qt QML shell into focused panels and extracted SDK/event conversion helpers.
- Extracted operations implementations into `filesystem.rs`, `search.rs`, `write.rs`, `mutations.rs`, `process.rs`, `artifacts.rs`, and `checkpoint.rs`; the remaining `lib.rs` owns dispatch, path policy, journaling, and the public facade.
- Rust tests, clippy, Qt CMake build, and offscreen startup passed.
