# Changes

- Added the public Rust `EventType` enum and stable string mapping.
- Added `EventPayload` plus typed turn-state, tool-state, and assistant-delta payload structures.
- Changed core emission helpers to accept a single typed event value while preserving the existing serialized SDK event envelope.
- Migrated all agent `emit` and `emit_live` call sites; arbitrary string/JSON pairs are no longer accepted.
