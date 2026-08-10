# Runtime Phase 1

Suncode's runtime is one local Rust process owning provider integration, agent turns, policy, SQLite state, approvals, checkpoints, credentials, and the client-facing SDK/API surface.

The runtime persists durable history in SQLite, keeps streaming deltas ephemeral, and exposes snapshots plus ordered events for clients.

The built-in provider catalog includes DeepSeek, Zhipu GLM, and OpenAI. Their stable Suncode model identities are `deepseek-v4-flash`, `glm-5.2`, and `gpt-5.6-sol`. Provider credentials are provider-keyed and stored as plaintext values in the Rust-owned SQLite `secret_records` table.
