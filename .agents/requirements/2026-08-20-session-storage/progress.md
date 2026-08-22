# Progress

- Status: Complete
- Last updated: 2026-08-22

## Completed

- Consolidated turn admission and approval recovery into `session_turn`.
- Added call, tool-use, and message projections with call correlations.
- Removed legacy session table schemas and updated current documentation.
- Added thinking-message projection test.
- Removed the duplicate `session_content` event journal and `session_sequences`; subscriptions now use live events with snapshot resync.
- Removed persisted tool-role messages from `session_message` and made `session_tool_use` the sole durable owner of tool results, with transient context reconstruction for later turns.
- Removed duplicate `session_message.usage_json`; call and turn records remain the authoritative usage projections.
- Added provider HTTP request and response-object identifiers to model-call persistence and diagnostics.

## Verification

- Workspace tests and lint are run as part of closeout; results are recorded in the final response.
