# Progress

- Status: Complete
- Last updated: 2026-08-20

## Completed

- Consolidated turn admission and approval recovery into `session_turn`.
- Added call, tool-use, and message projections with call correlations.
- Removed legacy session table schemas and updated current documentation.
- Added thinking-message projection test.
- Removed the duplicate `session_content` event journal and `session_sequences`; subscriptions now use live events with snapshot resync.

## Verification

- Workspace tests and lint are run as part of closeout; results are recorded in the final response.
