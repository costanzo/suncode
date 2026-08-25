# Progress

- Status: Complete
- Last updated: 2026-08-25

## Completed

- Added `suncode-common`.
- Unified core, LLM, and data business errors.
- Unified tool execution errors under `BusinessError` and removed `CoreFailure`.
- Removed Diesel dependencies from `suncode-common`; database conversion now belongs to `suncode-data`.
- Removed the `AgentError`, `SdkError`, and `PersistenceError` compatibility names.
- Preserved provider retry metadata and SDK serialization.
- Verified the workspace.
