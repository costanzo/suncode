# CLI Foundation

**Status:** Implemented and focused-tested

The native Rust CLI package lives under `apps/cli` and builds the `suncode` executable. It embeds `suncode_sdk::AsyncAgentSdk` directly on Tokio with Browser and Computer host capabilities disabled. It does not call the C ABI, open SQLite, contact providers, invoke operation crates, or implement policy.

Implemented commands are `doctor`, `models`, `auth list`, `auth set`, `auth remove`, and `config list`. `auth set` requires an interactive terminal and reads the credential without echo; credentials remain SQLite-owned through SDK methods. API-key arguments and environment variables are unsupported.

Global CLI options currently resolve explicit values over `SUNCODE_OUTPUT`, `SUNCODE_COLOR`, and `SUNCODE_USER_ID`. SDK bootstrap continues using `SUNCODE_DATA_DIRECTORY`, `SUNCODE_DATABASE_PATH`, and `SUNCODE_NON_INTERACTIVE`. Invalid values fail closed.

Text reports use stdout and errors use stderr. JSONL uses the versioned envelope in `contracts/cli.md`; SDK diagnostics remain on stderr. The CLI opens one embedded SDK, executes one administrative command, consumes explicit shutdown, and only then emits a success report. Stable exit mappings cover invalid input, unavailable providers/models/credentials, policy outcomes, data-directory contention, and ordinary failure.

`run`, `chat`, session commands, live event rendering, approvals, questions, turn cancellation, TUI, PTY, Browser/Computer UX, daemon/attach, distribution, and general CI policy profiles remain follow-up work.
