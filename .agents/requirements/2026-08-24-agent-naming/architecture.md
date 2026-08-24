# Architecture

The Rust package under `agent/crates/core` is the embedded agent SDK. Avalonia calls its method-oriented C ABI and does not access durable state directly. The `agent` boundary owns provider integration, agent turns, policy, persistence, credentials, recovery, undo, and audited operations.

The ABI reports version 3 and exports `suncode_agent_sdk_*`. New hosts use the current symbols directly. Startup uses only `agent.sqlite3`; there is no database fallback or legacy credential import path.
