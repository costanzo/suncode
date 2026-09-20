# SunCode CLI

Status: Foundation commands implemented; `run`, `chat`, session commands, approvals, questions, and signal-driven cancellation remain the next delivery.

This native Rust executable embeds `suncode_sdk::AsyncAgentSdk` directly. It never opens SQLite, contacts providers, invokes operation crates, or calls the C ABI. The normative behavior contract is [`../../contracts/cli.md`](../../contracts/cli.md).

Implemented commands:

```text
suncode doctor
suncode models
suncode auth list
suncode auth set PROVIDER
suncode auth remove PROVIDER
suncode config list
```

Global options currently include `--output text|jsonl`, `--color auto|always|never`, and `--user-id`. Their corresponding environment variables are `SUNCODE_OUTPUT`, `SUNCODE_COLOR`, and `SUNCODE_USER_ID`. SDK bootstrap continues to consume `SUNCODE_DATA_DIRECTORY`, `SUNCODE_DATABASE_PATH`, and `SUNCODE_NON_INTERACTIVE`.

The CLI opens the SDK with Browser Use and Computer Use disabled at the host capability boundary. Provider credentials are entered with no terminal echo and stored through the SDK; provider API-key environment variables are unsupported.

Build and test:

```text
cargo test --manifest-path apps/cli/Cargo.toml --all-targets
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
```
