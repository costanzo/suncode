# Changes

## Source

- Pending implementation.

## Contracts and generated artifacts

- No contract or generated artifact changes.

## Configuration and persistence

- No schema or data migration.

## Tests

- Preserve and relocate existing Store tests as needed.

## Documentation

- Record the table-owned operation layout.
# Implementation notes

- Added table-owned operation modules for project, project dependency, session, configuration, LLM provider/model, session messages/calls/images/tool uses/turns, approvals, checkpoints, and manifests.
- Kept `append_content` and `session_conversation_turns` in `operations/projection.rs` because they read/write multiple normalized tables; kept startup reconciliation in `operations/recovery.rs`.
- Preserved the existing inherent `Store` method API so core and SDK callers do not need a migration.
- Moved regression tests from `store.rs` to `src/tests.rs`.
- `store.rs` is reduced to 673 lines and no longer contains the former CRUD/query method implementations.
- Follow-up cleanup moved all remaining table-specific row conversion/query helpers out of `store.rs`; it is now 154 lines and contains no `QueryableByName` declarations or table operation helpers.
- `session_image_from_row` is owned by `operations/session_image.rs`; shared Diesel row declarations live in `rows.rs`, while table behavior remains in its matching operation module.
