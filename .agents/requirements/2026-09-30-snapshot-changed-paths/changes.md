# Changes

## Source

- `agent/crates/data/src/domain.rs`: `SessionConversationTurn.changed_paths`.
- `agent/crates/data/src/operations/checkpoint.rs`: `load_changed_paths`.
- `agent/crates/data/src/operations/projection.rs`: per-turn call in `session_conversation_turns`.
- `sdks/csharp/src/Models/SdkModels.cs`: optional trailing `SessionConversationTurn.ChangedPaths`.
- `apps/desktop-avalonia/ViewModels/DesktopViewModel.Persistence.cs`: `ProjectSnapshot` folds turn paths into the projection.

## Contracts and generated artifacts

- `contracts/agent-sdk/README.md`: `conversationTurns[*].changedPaths` semantics.
- No C ABI symbol or Rust SDK type change; the snapshot is serde JSON end to end.

## Configuration and persistence

- `agent/crates/database/src/sqlite/schema/session_checkpoint.sql`: idempotent `session_checkpoint_turn_ordinal_idx`.
- `contracts/sqlite-schema.md`: index note. No data migration; existing rows already carry `relative_path`.

## Tests

- `agent/crates/data/src/tests.rs`: `conversation_turns_list_checkpointed_paths_once_in_first_touched_order`; empty assertion in the existing projection test.
- `sdks/rust/src/facade/tests.rs`: snapshot JSON `changedPaths` assertion.
- `apps/desktop-avalonia/tests/SdkTypedModelTests.cs`: current and older payload deserialization.
- `apps/desktop-avalonia/tests/SessionSnapshotProjectionTests.cs`: cross-turn fold and review presentation after reopen.

## Documentation

- `.agents/specs/agent-phase-1.md`: snapshot restore sentence.
