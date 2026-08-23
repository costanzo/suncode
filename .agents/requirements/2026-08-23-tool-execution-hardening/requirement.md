# Requirement

SunCode's model-facing file and shell tools must have one stable contract whose advertised parameters match audited execution behavior. File inspection must support bounded continuation, search must respect repository ignore rules, mutations must preserve source encoding details and concurrency preconditions, and process failures must be visible in tool state.

## Requirements

- `read` applies 1-indexed `offset` and optional line `limit`, while retaining a bounded byte cap.
- `glob` respects standard ignore rules and reports bounded results.
- `grep` keeps the compact `pattern/path/include` model contract while retaining legacy internal aliases.
- `write` creates safe missing parent directories and retains precondition, checkpoint, and journal protection.
- `edit` accepts OpenCode-shaped replacements, supports multiple disjoint edits, preserves BOM and line endings, and rejects overlap.
- `bash` marks non-zero exits, timeouts, and cancellation as failed process states while preserving structured output and artifacts.
- WebFetch security behavior remains unchanged.
