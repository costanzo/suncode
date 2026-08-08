# Runtime Phase 1

Suncode's runtime is one local Rust process owning provider integration, agent turns, policy, SQLite state, approvals, checkpoints, credentials, and the client-facing SDK/API surface.

The runtime persists durable history in SQLite, keeps streaming deltas ephemeral, and exposes snapshots plus ordered events for clients. The first provider is DeepSeek V4 Flash.
