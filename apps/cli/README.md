# SunCode CLI

Status: Administrative commands plus one-shot new-session and resumed-session turns are implemented. Interactive multi-turn chat remains deferred.

This native Rust executable embeds `suncode_sdk::AsyncAgentSdk` directly. It never opens SQLite, contacts providers, invokes operation crates, or calls the C ABI. The normative behavior contract is [`../../contracts/cli.md`](../../contracts/cli.md).

Implemented commands:

```text
suncode doctor
suncode run [PATH] (--prompt TEXT | --stdin) [--model MODEL] [--reasoning-effort EFFORT]
suncode session list [PATH]
suncode session resume SESSION_ID (--prompt TEXT | --stdin) [--model MODEL] [--reasoning-effort EFFORT]
suncode session archive SESSION_ID
suncode models
suncode auth list
suncode auth set PROVIDER
suncode auth remove PROVIDER
suncode config list
```

Global options currently include `--output text|jsonl`, `--color auto|always|never`, and `--user-id`. Their corresponding environment variables are `SUNCODE_OUTPUT`, `SUNCODE_COLOR`, and `SUNCODE_USER_ID`. `run` and `session resume` also accept `SUNCODE_MODEL` and `SUNCODE_REASONING_EFFORT`. SDK bootstrap continues to consume `SUNCODE_DATA_DIRECTORY`, `SUNCODE_DATABASE_PATH`, and `SUNCODE_NON_INTERACTIVE`.

`run` opens the selected project and creates a primary session. `session resume` reopens an existing primary session and submits one new turn with its durable context. Both establish an atomic SDK watch, stream typed events, and print final assistant text to stdout. Text-mode progress stays on stderr; JSONL finishes with `run.result` or `session.resume.result`. The first interrupt requests SDK turn cancellation, while a second exits with status 130 after attempting explicit shutdown.

`session list` returns active and archived primary sessions for the selected project. `session archive` performs the SDK-owned lifecycle transition. Resume fails with status 4 if the session already has a pending approval or structured question; it does not resolve interactive state.

The CLI opens the SDK with Browser Use and Computer Use disabled at the host capability boundary. Provider credentials are entered with no terminal echo and stored through the SDK; provider API-key environment variables are unsupported.

Build and test:

```text
cargo test --manifest-path apps/cli/Cargo.toml --all-targets
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
```
