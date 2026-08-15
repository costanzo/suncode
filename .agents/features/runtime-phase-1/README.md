# Runtime Phase 1

SunCode's runtime is an embedded Rust SDK owning provider integration, agent turns, policy, SQLite state, approvals, checkpoints, credentials, and the client-facing native SDK surface. It has no standalone server or client-facing HTTP/SSE adapter.

The runtime persists durable history and per-turn provider-reported token usage in SQLite, keeps streaming deltas ephemeral, and exposes named methods, session usage aggregates, snapshots, and ordered direct subscriptions to its host process.

The SDK exposes bounded, read-only Git status and per-file diff methods backed by the audited operations crate. Repository discovery, project-relative path filtering, diff parsing, and output bounds remain Rust-owned; the client receives typed DTOs only.

The built-in provider catalog includes six providers with two static models each: DeepSeek (`deepseek-v4-flash`, `deepseek-v4-pro`), Zhipu GLM (`glm-5.2`, `glm-5.3`), OpenAI (`gpt-5.5`, `gpt-5.6-sol`), Kimi (`kimi-k2.7-code`, `kimi-k3`), Claude (`claude-sonnet-5`, `claude-opus-5`), and Gemini (`gemini-3.5`, `gemini-3.6-flash`). Provider credentials are provider-keyed and stored as plaintext values in the Rust-owned SQLite `secret_records` table. Provider endpoint/model environment overrides are not used.
